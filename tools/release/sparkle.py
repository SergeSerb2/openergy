#!/usr/bin/env python3
"""Local Sparkle bundle support and signed universal update archives. Never publishes."""
from __future__ import annotations

import argparse
import base64
import binascii
import hashlib
import json
import os
from pathlib import Path
import plistlib
import re
import signal
import stat
import subprocess
import sys
import tempfile
from urllib.parse import unquote, urlsplit
import xml.etree.ElementTree as ET
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SPARKLE_VERSION = "2.10.0"
SPARKLE_REVISION = "eef1a539a373c1f1a320624b1130fc5de7b2e100"
SPARKLE_CHECKSUM = "17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959"
NS = "http://www.andymatuschak.org/xml-namespaces/sparkle"
FRAMEWORK = Path("Sparkle.xcframework/macos-arm64_x86_64/Sparkle.framework")


def run(*args, sensitive=False):
    # Tool diagnostics can include malformed private-key contents. Never relay them.
    result = subprocess.run(["nice", "-n", "15", *map(str, args)], capture_output=True,
                            text=True, timeout=600, stdin=subprocess.DEVNULL)
    if result.returncode:
        detail = "" if sensitive else ": " + (result.stderr or result.stdout).strip()[:2000]
        raise ValueError(f"{Path(args[0]).name} failed ({result.returncode}){detail}")
    return result.stdout


def https_url(value, label):
    if not isinstance(value, str) or not 1 <= len(value) <= 2048:
        raise ValueError(f"{label} must be an HTTPS URL of at most 2048 characters")
    if any(ord(c) <= 32 or ord(c) >= 127 for c in value) or "\\" in value:
        raise ValueError(f"{label} must use printable ASCII with escaped URL characters")
    parts = urlsplit(value)
    if (parts.scheme != "https" or not parts.hostname or parts.username is not None
            or parts.password is not None or "#" in value or parts.port == 0):
        raise ValueError(f"{label} requires HTTPS, a host, no userinfo and no fragment")
    # Force hostname and port parsing; reject percent escapes in the authority.
    if "%" in parts.netloc:
        raise ValueError(f"{label} has an invalid host")
    if re.search(r"%(?![0-9a-fA-F]{2})", value):
        raise ValueError(f"{label} has an invalid percent escape")
    if ":" not in parts.hostname and (len(parts.hostname) > 253 or any(
            not re.fullmatch(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?", part)
            for part in parts.hostname.rstrip(".").split("."))):
        raise ValueError(f"{label} has an invalid host")
    return value


def public_key(value):
    try:
        if not isinstance(value, str) or len(value) != 44:
            raise ValueError()
        decoded = base64.b64decode(value, validate=True)
        if len(decoded) != 32 or base64.b64encode(decoded).decode() != value:
            raise ValueError()
    except (ValueError, binascii.Error):
        raise ValueError("SUPublicEDKey must be canonical base64 for exactly 32 Ed25519 bytes") from None
    return decoded


def configuration():
    required = os.environ.get("OPENERGY_REQUIRE_UPDATES", "0")
    if required not in ("0", "1"):
        raise ValueError("OPENERGY_REQUIRE_UPDATES must be 0 or 1")
    disabled = os.environ.get("OPENERGY_DISABLE_UPDATES", "0")
    if disabled not in ("0", "1"):
        raise ValueError("OPENERGY_DISABLE_UPDATES must be 0 or 1")
    feed = os.environ.get("OPENERGY_UPDATE_FEED_URL")
    key = os.environ.get("OPENERGY_UPDATE_PUBLIC_KEY")
    if disabled == "1":
        if required == "1" or feed is not None or key is not None:
            raise ValueError("OPENERGY_DISABLE_UPDATES=1 cannot combine with require-updates or update overrides")
        return None
    if feed is None and key is None:
        # Release-owner PUBLIC defaults apply when overrides are absent.
        # An explicit environment override always needs both values of its own.
        defaults = plistlib.loads((ROOT / "apps/macos/Support/Info.plist").read_bytes())
        feed, key = defaults.get("SUFeedURL"), defaults.get("SUPublicEDKey")
    if feed is None and key is None and required == "0":
        return None
    if not feed or not key:
        raise ValueError("set BOTH OPENERGY_UPDATE_FEED_URL and OPENERGY_UPDATE_PUBLIC_KEY "
                         "or a complete source Info.plist pair; updates are required or configuration is partial/empty")
    https_url(feed, "OPENERGY_UPDATE_FEED_URL")
    public_key(key)
    return feed, key


def configure_plist(args):
    config = configuration()
    data = plistlib.loads(args.template.read_bytes())
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", args.version):
        raise ValueError("version must have three numeric parts")
    if not re.fullmatch(r"[1-9][0-9]{0,8}", args.build):
        raise ValueError("build must be a positive integer of at most 9 digits")
    data.update(CFBundleShortVersionString=args.version, CFBundleVersion=args.build,
                SUEnableAutomaticChecks=config is not None)
    data.pop("SUFeedURL", None)
    data.pop("SUPublicEDKey", None)
    if config:
        data.update(SUFeedURL=config[0], SUPublicEDKey=config[1])
    args.output.write_bytes(plistlib.dumps(data, sort_keys=False))


def artifact(scratch):
    # This is the binary artifact of the committed exact SwiftPM resolution.
    pins = [p for p in json.loads((ROOT / "apps/macos/Package.resolved").read_text())["pins"]
            if p["identity"] == "sparkle"]
    if (len(pins) != 1 or pins[0]["location"] != "https://github.com/sparkle-project/Sparkle"
            or pins[0]["kind"] != "remoteSourceControl" or pins[0]["state"] != {
                "revision": SPARKLE_REVISION, "version": SPARKLE_VERSION}):
        raise ValueError("Sparkle resolution differs from the pinned official release")
    root = scratch.resolve() / "artifacts/sparkle/Sparkle"
    framework = root / FRAMEWORK
    data = plistlib.loads((framework / "Resources/Info.plist").read_bytes())
    if data["CFBundleShortVersionString"] != SPARKLE_VERSION:
        raise ValueError("Sparkle artifact version mismatch")
    run("codesign", "--verify", "--all-architectures", "--deep", "--strict", framework)
    if not (root / "LICENSE").is_file():
        raise ValueError("Sparkle artifact license is missing")
    return root


def nested_code(app):
    framework = app / "Contents/Frameworks/Sparkle.framework"
    version = framework / "Versions/B"
    # Inside-out signing, using real paths only (no duplicate symlink aliases).
    return [version / "Autoupdate", version / "Updater.app",
            version / "XPCServices/Downloader.xpc", version / "XPCServices/Installer.xpc",
            framework]


def binaries(app):
    version = app / "Contents/Frameworks/Sparkle.framework/Versions/B"
    return [app / "Contents/MacOS/Openergy", app / "Contents/Helpers/openergy",
            version / "Sparkle", version / "Autoupdate",
            version / "Updater.app/Contents/MacOS/Updater",
            version / "XPCServices/Downloader.xpc/Contents/MacOS/Downloader",
            version / "XPCServices/Installer.xpc/Contents/MacOS/Installer"]


def verify_app(app, archs):
    framework = app / "Contents/Frameworks/Sparkle.framework"
    for path in binaries(app):
        if not path.is_file() or not os.access(path, os.X_OK):
            raise ValueError("missing executable in app/framework")
        for arch in archs:
            run("lipo", "-verify_arch", arch, path)
        # Universal otool output has a separate file header for each architecture.
        links = [line.strip() for line in run("otool", "-L", path).splitlines() if line.startswith("\t")]
        if any(not line.startswith(("/System/Library/", "/usr/lib/", "@rpath/", "@loader_path/", "@executable_path/"))
               for line in links):
            raise ValueError("binary links to a non-system absolute dependency")
        commands = run("otool", "-l", path)
        minimums = re.findall(r"cmd LC_BUILD_VERSION.*?\n\s+minos ([0-9.]+)|"
                              r"cmd LC_VERSION_MIN_MACOSX.*?\n\s+version ([0-9.]+)", commands, re.S)
        if not minimums or any(tuple(map(int, (a or b).split("."))) > (15, 0, 0)
                               for a, b in minimums):
            raise ValueError("bundle executable requires newer than macOS 15.0")
    links = run("otool", "-L", app / "Contents/MacOS/Openergy")
    if "@rpath/Sparkle.framework/Versions/B/Sparkle" not in links:
        raise ValueError("app does not link to the bundled Sparkle framework")
    load_commands = run("otool", "-l", app / "Contents/MacOS/Openergy")
    if "path @executable_path/../Frameworks (offset" not in load_commands:
        raise ValueError("app lacks the bundle Frameworks runpath")
    for relative in ("Sparkle", "Resources", "Versions/Current"):
        if not (framework / relative).is_symlink():
            raise ValueError("Sparkle framework symlink was not preserved")
    for path in nested_code(app) + [app / "Contents/Helpers/openergy", app]:
        run("codesign", "--verify", "--all-architectures", "--strict", path)
    run("codesign", "--verify", "--all-architectures", "--deep", "--strict", app)
    entitlements = run("codesign", "-d", "--entitlements", ":-", app)
    if "com.apple.security.cs.disable-library-validation" in entitlements:
        raise ValueError("release bundle must retain library validation")


def embed(args):
    root = artifact(args.scratch)
    destination = args.app / "Contents/Frameworks/Sparkle.framework"
    if destination.exists():
        raise ValueError("framework staging destination already exists")
    destination.parent.mkdir(parents=True, exist_ok=True)
    run("ditto", root / FRAMEWORK, destination)
    run("ditto", root / "LICENSE", args.app / "Contents/Resources/Sparkle-LICENSE.txt")
    run("codesign", "--verify", "--all-architectures", "--deep", "--strict", destination)


def sign(args):
    identity = os.environ.get("OPENERGY_SIGN_IDENTITY") or "-"
    if identity != "-" and os.environ.get("OPENERGY_ALLOW_SIGNING") != "1":
        raise ValueError("named signing requires OPENERGY_ALLOW_SIGNING=1")
    timestamp = "--timestamp" if identity.startswith("Developer ID Application:") else "--timestamp=none"
    flags = ["--force", "--options", "runtime", timestamp, "--sign", identity]
    for path in nested_code(args.app):
        # Downloader's sandbox/network entitlements must survive re-signing.
        run("codesign", *flags, "--preserve-metadata=identifier,entitlements", path)
    run("codesign", *flags, "--identifier", "dev.openergy.daemon", args.app / "Contents/Helpers/openergy")
    run("codesign", *flags, args.app)
    verify_app(args.app, args.arch)


def tree_identity(root):
    result = {}
    for path in sorted(root.rglob("*")):
        info = path.lstat()
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            if not path.resolve().is_relative_to(root.resolve()):
                raise ValueError("bundle symlink escapes the app")
            result[relative] = ("link", os.readlink(path))
        elif path.is_file():
            result[relative] = ("file", stat.S_IMODE(info.st_mode), hashlib.sha256(path.read_bytes()).hexdigest())
        elif path.is_dir():
            result[relative] = ("dir", stat.S_IMODE(info.st_mode))
        else:
            raise ValueError("bundle contains a non-file entry")
    return result


def stable_zip(source, destination):
    # ditto retains resource forks and Unix symlinks but emits current access
    # times. Normalize only ZIP container metadata before signing the archive.
    with zipfile.ZipFile(source) as original, zipfile.ZipFile(
            destination, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as output:
        for entry in sorted(original.infolist(), key=lambda item: item.filename):
            info = zipfile.ZipInfo(entry.filename, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = entry.external_attr
            info.compress_type = zipfile.ZIP_DEFLATED
            output.writestr(info, original.read(entry))


def archive(args):
    if sys.platform != "darwin":
        raise ValueError("Sparkle packaging requires macOS")
    output = args.output.absolute()
    if output.exists() or output.is_symlink():
        raise ValueError("output already exists; choose a fresh output directory")
    app = args.app.resolve()
    if app.name != "Openergy.app":
        raise ValueError("update payload must be Openergy.app")
    data = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    if (data["CFBundleIdentifier"] != "dev.openergy.app"
            or data["CFBundleShortVersionString"] != args.version or data["CFBundleVersion"] != args.build):
        raise ValueError("bundle identifier/version/build differs from the requested update")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", args.version) or not re.fullmatch(r"[1-9][0-9]{0,8}", args.build):
        raise ValueError("invalid update version/build")
    feed = https_url(data.get("SUFeedURL"), "bundle SUFeedURL")
    public = public_key(data.get("SUPublicEDKey"))
    download = https_url(args.download_url, "download URL")
    name = f"openergy-{args.version}-{args.build}-macos-universal.zip"
    if unquote(urlsplit(download).path.rsplit("/", 1)[-1]) != name:
        raise ValueError(f"download URL filename must be {name}")
    if urlsplit(download).query:
        raise ValueError("use a stable download URL without query parameters")
    if data.get("SUEnableAutomaticChecks") is not True or data.get("LSMinimumSystemVersion") != "15.0":
        raise ValueError("update requires configured automatic checks and macOS 15.0 minimum")
    # A shared feed must never replace a universal app with a native-only payload.
    verify_app(app, ["arm64", "x86_64"])
    before = tree_identity(app)
    key = args.private_key_file.absolute()
    info = key.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_mode & 0o077 or not 1 <= info.st_size <= 256:
        raise ValueError("private key must be a regular file, mode 0600 or stricter, at most 256 bytes")
    try:
        secret = base64.b64decode(key.read_bytes().strip(), validate=True)
        if len(secret) not in (32, 96):
            raise ValueError()
    except (ValueError, binascii.Error):
        raise ValueError("private key file must contain a Sparkle base64 seed (32 bytes) or legacy key (96 bytes)") from None
    del secret
    tool = artifact(args.scratch) / "bin/sign_update"
    run("codesign", "--verify", "--all-architectures", "--strict", tool)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".openergy-update-", dir=output.parent) as temporary:
        work = Path(temporary)
        pending = work / "output"
        pending.mkdir()
        payload = pending / name
        original_zip = work / "ditto.zip"
        run("ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", app, original_zip)
        stable_zip(original_zip, payload)
        extracted = work / "extracted"
        run("ditto", "-x", "-k", payload, extracted)
        unpacked = extracted / "Openergy.app"
        if tree_identity(unpacked) != before or tree_identity(app) != before:
            raise ValueError("archive round trip or source bundle changed")
        verify_app(unpacked, ["arm64", "x86_64"])
        signature = run(tool, "--ed-key-file", key, "-p", payload, sensitive=True).strip()
        if len(base64.b64decode(signature, validate=True)) != 64:
            raise ValueError("Sparkle tool returned an invalid signature")
        # sign_update has no public-key option. Its documented legacy 96-byte
        # format is private[64]+public[32]. Verification uses only the public half.
        # This PUBLIC verification container binds both signatures to SUPublicEDKey.
        verifier = work / "public-verification-container"
        verifier.write_bytes(base64.b64encode(bytes(64) + public))
        verifier.chmod(0o600)
        run(tool, "--verify", "--ed-key-file", verifier, payload, signature, sensitive=True)
        ET.register_namespace("sparkle", NS)
        rss = ET.Element("rss", {"version": "2.0"})
        channel = ET.SubElement(rss, "channel")
        ET.SubElement(channel, "title").text = "Openergy updates"
        ET.SubElement(channel, "link").text = feed
        ET.SubElement(channel, "description").text = "Universal macOS updates for Openergy"
        item = ET.SubElement(channel, "item")
        ET.SubElement(item, "title").text = f"Openergy {args.version} ({args.build})"
        ET.SubElement(item, f"{{{NS}}}version").text = args.build
        ET.SubElement(item, f"{{{NS}}}shortVersionString").text = args.version
        ET.SubElement(item, f"{{{NS}}}minimumSystemVersion").text = "15.0"
        ET.SubElement(item, "enclosure", {"url": download, "length": str(payload.stat().st_size),
                      "type": "application/octet-stream", f"{{{NS}}}edSignature": signature})
        ET.indent(rss)
        appcast = pending / "appcast.xml"
        appcast.write_bytes(ET.tostring(rss, encoding="utf-8", xml_declaration=True) + b"\n")
        run(tool, "--ed-key-file", key, appcast, sensitive=True)
        run(tool, "--verify", "--ed-key-file", verifier, appcast, sensitive=True)
        parsed = ET.parse(appcast).getroot()
        enclosure = parsed.find("channel/item/enclosure")
        if enclosure is None or enclosure.attrib != {
                "url": download, "length": str(payload.stat().st_size), "type": "application/octet-stream",
                f"{{{NS}}}edSignature": signature} or parsed.findtext("channel/link") != feed:
            raise ValueError("signed appcast differs from the validated archive/feed")
        manifest = {"version": args.version, "build": args.build, "architectures": ["arm64", "x86_64"],
                    "feed_url": feed, "download_url": download, "public_key": data["SUPublicEDKey"],
                    "sparkle_version": SPARKLE_VERSION, "sparkle_revision": SPARKLE_REVISION,
                    "sparkle_artifact_sha256": SPARKLE_CHECKSUM,
                    "sign_update_sha256": hashlib.sha256(tool.read_bytes()).hexdigest(),
                    "publication": "local only; signing/notarization/publication trust gates remain separate"}
        manifest["files"] = {p.name: {"sha256": hashlib.sha256(p.read_bytes()).hexdigest(), "length": p.stat().st_size}
                             for p in (payload, appcast)}
        (pending / "update.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
        if output.exists() or output.is_symlink():
            raise ValueError("output appeared during packaging")
        pending.rename(output)
    print(f"Created {output.name}/{name} and signed appcast.xml; local only, no publication or trust clearance")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("config")
    command = commands.add_parser("configure-plist")
    for name in ("template", "output"):
        command.add_argument("--" + name, type=Path, required=True)
    command.add_argument("--version", required=True)
    command.add_argument("--build", required=True)
    for name in ("embed", "sign", "verify"):
        command = commands.add_parser(name)
        command.add_argument("--app", type=Path, required=True)
        if name == "embed":
            command.add_argument("--scratch", type=Path, required=True)
        else:
            command.add_argument("--arch", action="append", choices=("arm64", "x86_64"), required=True)
    command = commands.add_parser("archive")
    command.add_argument("--app", type=Path, required=True)
    command.add_argument("--output", type=Path, required=True)
    command.add_argument("--scratch", type=Path, default=ROOT / "apps/macos/.build")
    command.add_argument("--private-key-file", type=Path, required=True)
    command.add_argument("--version", required=True)
    command.add_argument("--build", required=True)
    command.add_argument("--download-url", required=True)
    args = parser.parse_args()
    if args.command == "config":
        print("Sparkle: configured" if configuration() else "Sparkle: disabled (unconfigured local build)")
    elif args.command == "verify":
        verify_app(args.app, args.arch)
        print("PASS: bundle architectures, relocation linkage, nested signatures and framework symlinks")
    else:
        {"configure-plist": configure_plist, "embed": embed, "sign": sign, "archive": archive}[args.command](args)


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(130)
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        sys.exit(f"sparkle: {error}")
