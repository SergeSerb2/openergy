# Current packaged app and daemon flow

The current worktree at `043757a` passed the existing real `--e2e` app command:
**25/25 checks, exit 0, 3.650 seconds**. The universal app and daemon were built
from `d647a01`. All 456 inputs in the prior combined build record still match
the current tree exactly. Later commits changed only documentation.

The command used the real `AppModel`, IPC client and bundled daemon with a
separate owned virtual peer. It required actual peer connection, disconnect and
rejoin. Other checks cover safe off-role startup, clipboard setting changes,
layout/explicit-link preservation, live logs, rejected layout rollback, ordered
Join/Off intent, a stopped owned daemon during a large IPC write, supervised
crash recovery, an ordinary legacy import warning and an empty import error.
The stalled-request case remained responsive and failed within its bound.

Both daemons used private config, TLS and socket paths, discovery disabled,
the virtual platform and explicit loopback ports 25530/25531. The harness creates
the app's initial off-role config before its model can connect. The peer was
running with the expected PID and platform before any app Join command. No
unrelated endpoint was contacted. The final inventory found no owned daemon or
assigned listener.

The app used `OPENERGY_E2E_NO_RENDER=1`. This proves model/IPC flows, not rendered
UI, native input/clipboard, login integration, Sparkle installation or relaunch.
It adds no speedup or native performance claim. No unit test, mock, desktop
automation, production config, service or installed app was used.

[results.json](results.json) binds the exact command, report, controller and
postflight. The six raw records are byte-exact copies from the retained private
fixture. The app SHA-256 is
`99f49c30c9e30c82899bd8cd5e2cf4a9e4293a4e8614010edfeb19589b6264ab`;
the bundled universal daemon SHA-256 is
`350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572`.
