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

The title correction and final saved evidence require the final focused review.
Native installation/relaunch and trusted distribution are not approved by these
source reviews.
