# Preparation review R1 and response

The read-only review of `d47f6a3f414b138b58d497e943113be40117f5d1` found one P2
and no P1. The proposed helper/cache-only scope omitted timestamp updates that
notify LaunchServices and conditional Gatekeeper scanning. Pinned
`Autoupdate/SUPlainInstaller.m:82-115,354-383` permits these effects for the
prepared same-volume ad-hoc candidate. No target relaunch is needed for either.

The parent confirmed those pinned source paths and added both effects, their
persistent-state evidence limits and the requirement to keep normal validation
enabled to `operation-v2-DO-NOT-RUN.json`. The current README points to v2.
Original proposal, metadata, artifacts, signatures and private originals remain
unchanged. Execution remains unauthorized pending the corrected-scope review
and an execution-scope decision.

Other R1 checks passed: retained hashes/originals, bundle inventories, source
bindings, offline signatures and permitted resource-seal verification. The
controller preservation limits were clearly disclosed. No CLI, installer,
target, build, server, network, registry command, test or key read occurred in
the review. It supplies no runtime, trust or installation acceptance.
