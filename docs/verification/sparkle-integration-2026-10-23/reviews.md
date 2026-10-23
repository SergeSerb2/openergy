# Independent Sparkle source reviews

All reviews used GPT-6.1-sol, xhigh effort, standard/default service tier and
isolated clean worktrees. Reviewers ran no builds, binaries, traffic, tests,
desktop automation or private-key operations. Saved evidence was context, not
independent runtime reproduction.

| Review | Candidate | Finding and response |
| --- | --- | --- |
| App R1, `oe-sparkle-app-independent-review-r1-8aab70f-20261023` | `8aab70fc01e813c18dd82ba159ad8d662666065a` | P2: manual checks could not refocus an active update UI. Removed the extra session restriction from production action/readiness in `6ab225e`; retained the fixture guard. |
| App R2, `oe-sparkle-app-combined-review-r2-6ab225e-20261023` | `6ab225e099f1e524d4cf104efbe7db041d943f12` | No concrete P1/P2. R1 closed. Controller ownership, launch gates, preference bindings, strict bundled configuration, private E2E isolation and candidate veto remain intact. |
| Package/hosting R2, `oe-sparkle-package-host-review-r2-6ab225e-20261023` | `6ab225e099f1e524d4cf104efbe7db041d943f12` | P2: generator used `Openergy updates` while hosting verification requires `Openergy Updates`. Corrected the generator in `d647a01` before signing. No other concrete P1/P2; ad-hoc options `0` and named Hardened Runtime paths are consistent. |
| Final R3, `oe-sparkle-final-source-evidence-review-r3-2d89aef-20261023` | Source delta `6ab225e` → `d647a01`; evidence `c7c805a` + `2d89aef` | No concrete P1/P2. Title P2 closed. All 456 source hashes, 26 raw records and private originals, and four artifact hashes/sizes agree. Command logs, manifests, feed fields and archived app/daemon binaries are consistent. |

R3 completed read-only at clean HEAD
`2d89aef951ddf74b51c1752e0c6a8a228c89b117`. It reviewed source
`d647a01a7b83f79187d1a15c067a763647f01a31` and evidence
`c7c805a72ac5409f4dcfd465c9229c0f5e6fbc72` plus that HEAD.
`results.json` retains its original collection-time pending-review status;
this record supplies the final disposition without rewriting the audited snapshot.
R3 read no private key and repeated no runtime check.

Visual UI, Sparkle archive download, installation, daemon handoff/relaunch,
Developer ID signing, notarization, Gatekeeper, native input/clipboard, Windows,
strict DNS, original security/provenance and distribution gates remain open.
