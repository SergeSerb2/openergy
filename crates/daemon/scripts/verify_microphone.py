#!/usr/bin/env python3
"""Exercise microphone sharing with real daemons, private Xvfb and PulseAudio.

Linux only. Requires Xvfb, PulseAudio, and pacat.
No process connects to the desktop, the system audio server, or the LAN.
"""

import argparse
from contextlib import ExitStack
import json
import math
import os
from pathlib import Path
import select
import socket
import struct
import subprocess
import tempfile
import time


RATE = 48000
SINKS = ("host_source", "client_source", "host_cable", "client_cable")


def wait_for(predicate, label):
    end = time.monotonic() + 10
    while time.monotonic() < end:
        try:
            if predicate():
                return
        except (OSError, subprocess.CalledProcessError):
            pass
        time.sleep(0.1)
    raise RuntimeError(f"Timed out: {label}")


def stop(process):
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)


def route(root, env, stack, exe):
    def spawn(args, subprocess_env=env, **kwargs):
        process = subprocess.Popen(args, env=subprocess_env, **kwargs)
        stack.callback(stop, process)
        return process

    def run(args, subprocess_env=env):
        return subprocess.check_output(args, env=subprocess_env, text=True, timeout=5)

    pulse_socket = root / "pulse.sock"
    env["PULSE_SERVER"] = f"unix:{pulse_socket}"
    cookie = root / "pulse-cookie"
    cookie.write_bytes(os.urandom(256))
    cookie.chmod(0o600)
    env["PULSE_COOKIE"] = str(cookie)
    pulse_config = root / "pulse.conf"
    pulse_config.write_text(
        f"load-module module-native-protocol-unix socket={pulse_socket} auth-cookie={cookie}\n"
        + "".join(f"load-module module-null-sink sink_name={sink} rate={RATE} channels=1\n" for sink in SINKS),
        encoding="utf-8",
    )
    spawn(["pulseaudio", "-nF", pulse_config, "--daemonize=no", "--exit-idle-time=-1",
           "--disable-shm=yes", "--use-pid-file=no"])
    wait_for(lambda: pulse_socket.exists(), "private PulseAudio server")
    # An empty ALSA configuration keeps enumeration away from hardware devices.
    alsa_config = root / "alsa.conf"
    alsa_config.write_text("# No ALSA devices configured for this run\n", encoding="utf-8")
    env["ALSA_CONFIG_PATH"] = str(alsa_config)
    devices = json.loads(run([exe, "microphone", "devices"]))
    ids = {row["id"] for row in devices}
    required = {"pulseaudio:host_source.monitor", "pulseaudio:client_source.monitor",
                "pulseaudio:host_cable", "pulseaudio:client_cable"}
    if not required.issubset(ids):
        raise RuntimeError(f"Missing isolated devices: {devices}")

    with socket.socket() as reservation:
        # This range is reserved for this isolated test. Never use production 24800.
        for candidate in range(25680, 25700):
            try:
                reservation.bind(("127.0.0.1", candidate))
            except OSError:
                continue
            port = candidate
            break
        else:
            raise RuntimeError("No available port in 25680-25699")

    endpoints = {}
    for name in ("host", "client"):
        work = root / name
        work.mkdir(mode=0o700)
        read, write = os.pipe()
        with os.fdopen(read, "r") as display_file, os.fdopen(write, "w") as fd:
            xvfb = spawn(["Xvfb", "-displayfd", str(fd.fileno()), "-screen", "0",
                          "640x480x24", "-nolisten", "tcp"], pass_fds=(fd.fileno(),))
            fd.close()
            if not select.select([display_file], [], [], 5)[0]:
                raise RuntimeError("Private X server did not provide a display")
            display = display_file.readline().strip()
        if not display.isdecimal() or xvfb.poll() is not None:
            raise RuntimeError("Private X server failed")
        endpoints[name] = (work, {**env, "DISPLAY": f":{display}"})
    fingerprints = {name: run([exe, "fingerprint", "--dir", work]).splitlines()[0]
                    for name, (work, _) in endpoints.items()}
    daemons = {}

    def start(name, enable_capture):
        work, node_env = endpoints[name]
        args = [exe, "daemon", "--platform", "x11", "--config", work / "config.toml",
                "--socket", work / "ctl.sock"]
        if enable_capture:
            args.append("--enable-microphone-capture")
        daemons[name] = spawn(args, node_env, stdout=subprocess.DEVNULL)

    for name, other in (("host", "client"), ("client", "host")):
        work, _ = endpoints[name]
        run([exe, "trust", "approve", fingerprints[other], "--dir", work])
        role = "server" if name == "host" else "client"
        config = work / "config.toml"
        config.write_text(
            f'role = "{role}"\nname = "{name}"\n'
            f'[server]\nlisten = "127.0.0.1:{port}"\nport = {port}\n'
            f'[client]\nremote = "127.0.0.1:{port}"\n'
            f'[microphone]\npeer_fingerprint = "{fingerprints[other]}"\n'
            f'input_device = "pulseaudio:{name}_source.monitor"\n'
            + ('output_device = "pulseaudio:host_cable"\n' if name == "host" else "")
            + '[[screen]]\nname = "host"\nrect = {x=0,y=0,w=640,h=480}\n'
            + '[[screen]]\nname = "client"\nrect = {x=640,y=0,w=640,h=480}\n',
            encoding="utf-8",
        )
        start(name, enable_capture=name == "host")

    def ctl(name, method="status.get", **params):
        return json.loads(run([exe, "ctl", method, "--socket", endpoints[name][0] / "ctl.sock",
                               "--json", json.dumps(params), "--raw"]))

    def state(name, field, expected):
        return ctl(name)["microphone"][field] == expected

    def patch(name, **fields):
        ctl(name, "config.patch", patch={"microphone": fields})

    def sink_audio(sink, require_data=True):
        with tempfile.TemporaryFile() as capture:
            process = spawn(["pacat", "--record", "--raw", f"--device={sink}.monitor",
                             "--rate=48000", "--channels=1", "--format=s16le", "--latency-msec=20"],
                            stdout=capture)
            time.sleep(1.0)
            stop(process)
            capture.seek(0)
            raw = capture.read()
        values = [sample for (sample,) in struct.iter_unpack("<h", raw)]
        if require_data and len(values) < RATE * 0.3:
            raise RuntimeError("Insufficient audio from the private virtual cable")
        return values

    def tone(sink, hz):
        path = root / f"{sink}.pcm"
        second = b"".join(struct.pack("<h", int(12_000 * math.sin(2 * math.pi * hz * n / RATE)))
                          for n in range(RATE))
        path.write_bytes(second * 30)
        with path.open("rb") as source:
            spawn(["pacat", "--playback", "--raw", f"--device={sink}", "--rate=48000",
                   "--channels=1", "--format=s16le", "--latency-msec=20"], stdin=source)

    def power(audio, hz):
        in_phase = sum(s * math.cos(2 * math.pi * hz * n / RATE) for n, s in enumerate(audio))
        quadrature = sum(s * math.sin(2 * math.pi * hz * n / RATE) for n, s in enumerate(audio))
        return math.hypot(in_phase, quadrature) / len(audio)

    def expect_tone(sink, expected, other):
        def hears_tone():
            samples = sink_audio(sink, require_data=False)
            if len(samples) < RATE * 0.3:
                return False
            level = power(samples, expected)
            return level >= 150 and level >= 3 * power(samples, other)

        wait_for(hears_tone, f"{expected} Hz at {sink}")

    wait_for(lambda: ctl("client")["state"] == "connected", "initial input connection")
    assert state("client", "input_state", "needs_consent")
    unprivileged = subprocess.run(
        [exe, "ctl", "microphone.start", "--socket", endpoints["client"][0] / "ctl.sock",
         "--json", json.dumps({"peer_fingerprint": fingerprints["host"],
                              "input_device": "pulseaudio:client_source.monitor"}), "--raw"],
        env=env, capture_output=True, text=True, timeout=5)
    assert unprivileged.returncode == 1 and json.loads(unprivileged.stdout)["error"]["code"] == "permission_denied"
    stop(daemons["client"])
    start("client", enable_capture=True)
    wait_for(lambda: state("client", "input_state", "sharing"), "client microphone starts")
    assert state("host", "input_state", "waiting_for_receiver")
    tone("client_source", 440)
    tone("host_source", 733)
    expect_tone("host_cable", 440, 733)
    expect_tone("client_source", 440, 733)

    since = ctl("host")["peers"][0]["since_unix_ms"]
    patch("client", output_device="pulseaudio:client_cable")
    wait_for(lambda: state("host", "input_state", "sharing"), "reverse audio starts")
    expect_tone("client_cable", 733, 440)
    expect_tone("host_cable", 440, 733)

    patch("client", output_device="pulseaudio:openergy_missing")
    wait_for(lambda: state("client", "output_state", "error"), "device error")
    wait_for(lambda: state("host", "input_state", "waiting_for_receiver"), "sender stops after receiver error")
    assert state("client", "input_state", "sharing")

    patch("client", output_device="pulseaudio:client_cable")
    wait_for(lambda: state("host", "input_state", "sharing"), "reverse audio recovers")
    ctl("host", "microphone.stop")
    wait_for(lambda: state("host", "input_state", "needs_consent"), "stop capture")
    assert ctl("host")["peers"][0]["since_unix_ms"] == since
    time.sleep(0.35)
    assert state("client", "output_state", "ready")
    assert max(map(abs, sink_audio("client_cable", require_data=False)), default=0) < 50
    expect_tone("host_cable", 440, 733)

    patch("host", peer_fingerprint=":".join(["00"] * 32))
    wait_for(lambda: ctl("client")["state"] == "connected"
             and state("client", "input_state", "waiting_for_peer")
             and state("host", "output_state", "waiting_for_peer"),
             "TLS connection with unmatched audio peer")

    patch("host", peer_fingerprint=fingerprints["client"])
    wait_for(lambda: state("client", "input_state", "sharing"), "audio restores with correct peer")
    client_dir = endpoints["client"][0]
    run([exe, "trust", "revoke", fingerprints["host"], "--dir", client_dir])
    wait_for(lambda: state("client", "input_state", "needs_consent"), "offline revocation disarms capture")
    stop(daemons["client"])
    start("client", enable_capture=True)
    wait_for(lambda: state("client", "input_state", "needs_consent"), "startup requires an already trusted peer")
    wait_for(lambda: any(peer["fingerprint"] == fingerprints["host"]
                         for peer in ctl("client")["pending_trust"]), "client asks for trust")
    ctl("client", "trust.approve", fingerprint=fingerprints["host"])
    wait_for(lambda: ctl("client")["microphone"]["peer_connected"], "trust approval restores audio route")
    assert state("client", "input_state", "needs_consent")

    stop(daemons["client"])
    start("client", enable_capture=True)
    wait_for(lambda: state("client", "input_state", "sharing"), "explicit consent restores capture")
    ctl("host", "config.patch", patch={"tls": {"enabled": False}})
    ctl("client", "config.patch", patch={"tls": {"enabled": False}})
    wait_for(lambda: ctl("client")["state"] == "connected", "plaintext input connection")
    assert state("client", "input_state", "needs_consent")
    assert state("host", "output_state", "waiting_for_peer")
    print("PASS: explicit consent, bidirectional audio, device recovery, live stop, pinning, revocation, and TLS-only audio")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("openergy", type=Path)
    exe = parser.parse_args().openergy.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="openergy-mic-") as directory, ExitStack() as stack:
        root = Path(directory)
        env = {**os.environ, "OPENERGY_DISABLE_DISCOVERY": "1", "OPENERGY_ALLOW_PORT_24800": "0",
               "XDG_RUNTIME_DIR": directory, "XDG_CONFIG_HOME": directory,
               "PULSE_RUNTIME_PATH": directory, "PULSE_STATE_PATH": directory,
               "XAUTHORITY": str(root / "Xauthority")}
        route(root, env, stack, exe)


if __name__ == "__main__":
    main()
