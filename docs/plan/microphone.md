# Share a microphone between two computers

You can use a physical microphone on the computer it plugs into, and use the
same audio as a microphone on another Openergy computer. Either the keyboard
host or the client can send audio. Both directions can run at the same time
with separate devices. Moving the pointer between screens leaves the audio
stream running, so a call can continue while you work on the other computer.

## Set up the devices

1. Update Openergy on both computers. Connect them with TLS and verify both
   certificate fingerprints using the normal approval flow. Keep client
   authentication enabled on the keyboard host.
2. On the receiving computer, set up a virtual audio cable. It provides a
   playback device for Openergy and a matching recording device for the app
   that needs the microphone. On macOS, [BlackHole 2ch][blackhole] exposes
   both sides as **BlackHole 2ch**. On Windows, [VB-CABLE][vb-cable] exposes
   **CABLE Input** for playback and **CABLE Output** for recording. On Linux,
   use an ALSA loopback device or a virtual sink and its monitor through
   PulseAudio or PipeWire’s PulseAudio service.
3. On each computer, run `openergy microphone devices` and copy the device
   `id` from the JSON list. On macOS, the bundled CLI is at
   `Openergy.app/Contents/Helpers/openergy`. Select the physical microphone
   as the sender's input and the virtual cable's playback endpoint as the
   receiver's output. Keep your regular system speaker and microphone
   defaults. Set the virtual cable as the microphone inside the receiving app.

## Enable one direction

Use the peer fingerprint from `openergy status` on each machine. Use the
fingerprint under the connected peer, which you already verified. The commands
below show POSIX-style quoting. On Windows you can pass the same JSON using
your shell's quoting rules or edit the daemon TOML while it is stopped.

On the computer that **has the microphone**, set the other computer's
fingerprint and this computer's capture device:

```sh
openergy ctl config.patch --json '{"patch":{"microphone":{"peer_fingerprint":"RECEIVER_FINGERPRINT","input_device":"PHYSICAL_MIC_ID"}}}'
```

On the computer that **needs the microphone**, set the sender's fingerprint
and this computer's virtual cable playback device:

```sh
openergy ctl config.patch --json '{"patch":{"microphone":{"peer_fingerprint":"SENDER_FINGERPRINT","output_device":"VIRTUAL_CABLE_PLAYBACK_ID"}}}'
```

The equivalent `[microphone]` TOML fields are `peer_fingerprint`,
`input_device`, and `output_device`. Use the full colon-separated SHA-256
fingerprint. Setting or changing a fingerprint reconnects the input session to
negotiate audio. Changing a device on the same peer updates the audio route
live. To add the reverse direction, set the remaining device on each computer.
Keep any virtual cable playback device separate from the capture device to
avoid audio feedback.

For example, with a Blue Yeti plugged into the MacBook client, configure
the Blue Yeti as the MacBook's `input_device`. On the host, configure the
virtual cable as `output_device`. Select the Blue Yeti in MacBook apps and
the cable's recording endpoint in host apps.

## Start, check, and stop sharing

On the Mac app, open the Openergy menu and choose **Start Sharing**. Review
the device and peer fingerprint, then confirm. The menu bar shows a microphone
icon while sending audio.

If you run the sender as a standalone daemon, save the microphone route first,
then start the daemon with `--enable-microphone-capture` and the same config and
socket options you normally use. The flag authorizes only the input device and
peer in that startup config. Receive-only daemons need no flag. The local
control socket can stop capture but cannot start standalone capture.

On macOS, grant microphone access when prompted. If access was denied,
check **System Settings → Privacy & Security → Microphone**, then choose **Start Sharing** again.
On Windows, allow desktop apps to use the microphone if capture is blocked.

Run `openergy status` and look under `microphone`. The sender reports
`input_state: sharing` when it has opened the mic and the receiver is ready.
The receiver reports `output_state: ready` when it has opened the selected
playback device. The status also reports device errors. Reconnect after
restoring a missing device, or select an available device ID. Check the
recording meter in your receiving app.

To stop sending, choose **Stop Sharing** in the macOS menu or run:

```sh
openergy ctl microphone.stop
```

Starting again after a stop, an explicit disconnect, a lock or sleep notice,
a peer revocation, or a change to the capture device or peer requires a new
menu confirmation or a new standalone daemon launch.

Clear `output_device` to stop receiving. You can clear either device without
changing the keyboard and mouse connection.

## Privacy and compatibility

Sharing is off by default. A microphone opens only when the designated peer
has a verified TLS connection and has opened its selected playback device.
The receiver can hear the microphone while the pointer is on either screen.
Disconnect, suspend or lock when the platform reports it, or remove the input
device from the sharing configuration to stop capture. Openergy forwards
mono audio in memory. Apps on the receiving computer can record it through
the virtual microphone.

Deskflow, Barrier, Synergy, and older Openergy releases can still share
keyboard and mouse input. Microphone sharing requires compatible Openergy
releases at both ends. Virtual audio drivers are installed separately.
This feature is verified with isolated audio on Linux. Physical device,
macOS permission, Windows driver, and call-app acceptance still need testing.

[blackhole]: https://github.com/ExistentialAudio/BlackHole/wiki/Use-BlackHole-as-a-Virtual-Microphone
[vb-cable]: https://vb-audio.com/Cable/
