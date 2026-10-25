# Prepared record retention bound

STATIC DATA ONLY. execution=false. No preparation or remote read ran.

The unchanged preparer emits prepared.json with 14 fields. Its bound artifact receipt and bundle inventory of 167 rows are already retained. The actual new private directory is the only variable input. A single leaf of at most 255 bytes needs at most 1,530 ASCII JSON bytes after escaping. The model uses control characters to reach that bound. It is not an admitted or real path.

Using the exact pinned common encoder, all 21 selected limits, all 13 environment keys and the seven private-path occurrences, the encoded record is bounded by 72,870 bytes. Base64 needs at most 97,160 bytes. A separate reader envelope of 16,384 bytes leaves the total at 113,544, within the unchanged transport stdout cap of 131,072 bytes. A single read of the future prepared.json can therefore fit without raising caps, subject to the stated exact inputs and actual path/data checks.

This proves a serialized-data size bound. It does not implement or qualify a reader. It does not prove that prepared.json exists, that preparation succeeded, or that any copy is ready to launch. The future parent still needs actual exits/EOF/D4, the full remote receipt and all 12 fixed boundary/ledger leaves. Large helper/primary output and the full receipt keep their existing separate caps; this bound must not be applied to those files.

All five inputs were read with bounded nofollow regular-file checks and exact SHA-256. The preparer was parsed as AST data only to extract its literal limit_names tuple. No supplied source was imported, evaluated, compiled as executable code or run. No unit tests or mocks were added. A prior read-only lookup at source/limits.json failed with exit 1; the correct packet-root limits.json was verified and that miss remains in bound.json.

The prepare capsule is still under source repair, and the local observer and COPY capsule remain separate work. Before use, confirm the repaired capsule retains these exact unchanged prepare/common/limits/artifact/tree inputs. All historical failures, unknown port cause and runtime/whole-flow NI15 states remain unchanged. This calculation grants no runtime or artifact acceptance.
