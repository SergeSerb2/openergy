# Historical refusal-control evidence audit R15

No concrete P1/P2 was found in the read-only audit of evidence
`1d9b6a6ff0f234bd663647b325adf3d47f1743ca`, integration
`c85b78fdefa010e0e314253b65c990b41ed3f569` and reviewed HEAD
`270106002490ef3e135d4c6500df1c8ff452b45b`. The evidence trees match.

The reviewer verified the 600-file export, 599 unchanged historical files, sole
exact probe overlay, artifact hashes, private bundle, 61 copy transformations,
timing/cardinality and cleanup bindings. The historical source is
`615c16e1df71bc0b350aeee4f1f34bceba5a59e0`; the overlay SHA-256 is
`530c880eb74f6c95d24a0612c4fa8773fa21d256765fc3f9f6260801fa303a30`.

The saved run exits 0 in 70.420 seconds. Both ports qualify and all ten refusal
cases pass, each with one correct event before five seconds and a complete
six-second observation window. There is no false Connected, extra event or
observer trace. The historical fault was **not reproduced**. Its negative-control
gate therefore stays **unverified**, as stated in the original report.

The executable hash is
`3276f67ca621d9efdb1327eda59b85cdcfebe89384fd4cdcbb854fdd2ca0ee52`;
the private bundle hash is
`1dbaff1ab824320ee13047547089b8e928cff58cca7fe9a61d012bf26f358271`.
The worktree remained clean. The audit repeated no runtime command or R12 run.
It adds no independent execution proof. Strict TCP/fallback/TLS timing, full
acceptance and broader native gates remain open.
