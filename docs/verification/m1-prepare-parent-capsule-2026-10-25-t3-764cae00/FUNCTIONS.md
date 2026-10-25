# Whole function inventory

Full physical ranges, lengths and SHA-256 values are in [function-inventory.json](function-inventory.json). Hashes include decorators and LF. Nested functions are listed separately and also belong to their enclosing function bytes. The Stop class is a separate unchanged declaration.

## Changed

- `closed_json` (FunctionDef, lines 1152..1165), SHA-256 `019219040de9c1e8fb305ae0ee2e61a3335591568ac79d70de9011d53b77b2eb`.
- `bind_config` (FunctionDef, lines 1168..1208), SHA-256 `b04c520e2b268acf889f27957d315eb381a7a128a1ff8140fd17b05a583a79ec`.
- `collect` (FunctionDef, lines 1580..1714), SHA-256 `e2de320545b457197932daa6a7933be5351b13d2b3b2599fb2e6ac3a7ac01245`.
- `emit_once` (FunctionDef, lines 1717..1727), SHA-256 `548d547e0d17682cc006964f4d75fb3e6f7aa3132576600fc87e1727c9349cc7`.
- `main` (FunctionDef, lines 1730..1856), SHA-256 `efdcbcba517ab3d7842435e2ac5e8d595f2c4e8cc36ba0e95c20296e50734c3e`.

## Added

- `closed_json.nonfinite` (FunctionDef, lines 1159..1160), SHA-256 `f32a3a64e87b6448d7ef0b65c9c39395f23789cd334acc2948a07eb81d192925`.
- `closed_json.finite_float` (FunctionDef, lines 1161..1164), SHA-256 `580f75ecd092de39c675e9c790c854f42fe72decd96690c24b15ad302f5d7b85`.
- `fail` (FunctionDef, lines 1211..1214), SHA-256 `de1bd3450d76eb0036a1f40c2d29b91d542d1715469a2bb1256f8f309997adcc`.
- `absent` (FunctionDef, lines 1217..1222), SHA-256 `947f6b50c9344014a2e2d2a3cc77b865affc8ba26d46a98c5b8bbcc62d600b0f`.
- `console_admission` (FunctionDef, lines 1225..1235), SHA-256 `a5447dfce8fef6cac5259f0315182e0d0c206e8a9cfe50b8ffe280ef029e41fe`.
- `read_new_leaf` (FunctionDef, lines 1238..1257), SHA-256 `2b9c6970ae56804639e26a755b436528d84ae41d4ae0df09f3cea42d86b5b46c`.
- `read_build_record` (FunctionDef, lines 1260..1273), SHA-256 `6ea479ece901fb742b3e22a71d973af62d23dc35897fa04bb33f256f2fc4a8fa`.
- `reference` (FunctionDef, lines 1276..1281), SHA-256 `fd11ab41a612c579f21d048250603a916429d50a7022e540a71a0bb87f698fe3`.
- `inspect_artifact` (FunctionDef, lines 1284..1314), SHA-256 `1824e62a3616f2a40b03f3d9316fa1c96120a00626ab308ffff2f14150185c30`.
- `inspect_frozen` (FunctionDef, lines 1317..1363), SHA-256 `884c872a746f513558473b04be39ad4fa04013a88e66a73c4dff3eb0fc6e2504`.
- `initial_put` (FunctionDef, lines 1366..1403), SHA-256 `7b3fdc887bb007e6ae1e7378f0599cd602d7dfec450a1f2e7fcd3fa7ff493249`.
- `inspect_entry` (FunctionDef, lines 1406..1426), SHA-256 `d89e380c42166abef964b867a3e431cecb96cda8e5bad6919424a27dd594cc51`.
- `import_common` (FunctionDef, lines 1429..1447), SHA-256 `abc3f54bb7a317994224f575475a35192f6c7f8fc72b880af135c4dcd09025c7`.
- `common_deadline` (FunctionDef, lines 1450..1465), SHA-256 `1d8dcb38b35c9dd3390e5275b12940e0d8b5662ccf01a52aa230ad8a87da6b72`.
- `existing_ledger` (FunctionDef, lines 1468..1479), SHA-256 `f14b8a32e7ffa286b368e1e3fb4e870b85200b59a20fac15b6a87e1b94869d81`.
- `reserved_put` (FunctionDef, lines 1482..1513), SHA-256 `2af9e40a0c20195b3e4cb9fbd2073581c70d05a502cccba443a7eee67c945ae0`.
- `canonical_prepared_directory` (FunctionDef, lines 1516..1532), SHA-256 `0eba03c23e2c12eafe1e1a13d91721ed671ead77aaaaadd2f4725d575ff6bf1d`.
- `inspect_prepared` (FunctionDef, lines 1535..1579), SHA-256 `474b6b43175cd5c662109f02ac24cf1e9148a82d1047b785cbb1c355e13b7636`.

## Unchanged

- `Stop` (ClassDef, lines 929..930), SHA-256 `fd5f0e9e870ae08920c1d407fc73b253c407fdd0aef25c908048f181025ae7f5`.
- `require` (FunctionDef, lines 933..935), SHA-256 `fa57bb7c8de0a6f63c620d5d56849a58849c546317137baa3463c4ce8f7d4766`.
- `clock_ns` (FunctionDef, lines 938..943), SHA-256 `0a708be90d476ccafb3f658a6edeae5a969cd1fed47fc9e4568632c78144e7bf`.
- `remaining` (FunctionDef, lines 946..949), SHA-256 `b723e8885f1273197aa575682bee963f2cefbec7b0df8c8b6a051a0e26ac7c78`.
- `until` (FunctionDef, lines 952..968), SHA-256 `00ff5b79c8d3acd8bbcd3e298fb472fab7427f91611e93116c199224033746ff`.
- `until.expired` (FunctionDef, lines 959..960), SHA-256 `bc1ec102edf0eae776d4cdfa698bb0528d7764f751efecc51a5cd16bd1bd3505`.
- `sha` (FunctionDef, lines 971..972), SHA-256 `4e627faf4b903521e935f7c6f3ae93545856f821d0e2565ba7ea6cf97d3d0e6c`.
- `encoded` (FunctionDef, lines 975..977), SHA-256 `47546d7a82bbb5e109d0a86a34e3046f3adab68837513ddf922600b33ce202d2`.
- `hex_string` (FunctionDef, lines 980..982), SHA-256 `fd9f19e03ed808aeded45d465053a19b34fd34724e58ee0d2c21641f1df07d78`.
- `same_data` (FunctionDef, lines 985..993), SHA-256 `b03e25037d583929f50a0cc97c4069c4e211552782a8f14037cba370a49734fe`.
- `stamp` (FunctionDef, lines 996..998), SHA-256 `5ad54262f0589d8245e2a4e1d0fc970a2264bcd8ad3c9eee89e66022b017e1c0`.
- `open_root` (FunctionDef, lines 1001..1017), SHA-256 `c7aee36ba2ba05b59397e13a30160ffc77a94a2360d705ff22a54dfe69ae163f`.
- `read_leaf` (FunctionDef, lines 1020..1039), SHA-256 `963beb17880dcd6f322370c6df7a7d2a0c1cf573b049cbcf990faf02e4af389f`.
- `inspect_remote` (FunctionDef, lines 1042..1076), SHA-256 `1def34c6cbdaffd2021c2ff805c428fcef4a11d3a2ce2bebb4fe512149a4c47b`.
- `inventory` (FunctionDef, lines 1079..1114), SHA-256 `12c46f1c5b191b7fdd68455b26c33fc259c4119b9b3b5b9647a93a5737ebb22f`.
- `inventory.walk` (FunctionDef, lines 1082..1112), SHA-256 `14509512ab33bd3066c48331714922123e25b543cae89ca20eb2ed0ffe8845aa`.
- `startup` (FunctionDef, lines 1117..1122), SHA-256 `77f6e5efc6f48392d70f305e7f8be86b6d20f82574df1cff818d28389c088bd9`.
- `latch` (FunctionDef, lines 1125..1127), SHA-256 `1fc81f469b2139c30fcccfcc0ba115eec20e8deb4cd190d363f56bf31fe80640`.
- `observe_exit` (FunctionDef, lines 1130..1137), SHA-256 `94a6fafecbef10b6509533a0d792cce1fc61b4fcb99d9183118da1916b83610c`.
- `collection_complete` (FunctionDef, lines 1140..1149), SHA-256 `1e392aa0f5b155a79c614f7f8b967bf8edeefba7fb7729ccaa3ceacc25c6a793`.
- `closed_json.pairs` (FunctionDef, lines 1153..1158), SHA-256 `0631fc06468cfa7e32fdd726ce494a810b7000bdc90d5bc09083833869579093`.

## Removed baseline functions

- `inspect_initial`: merged into the fixed `inspect_frozen` admission.
- `parent_record`: metadata SOURCE parent-record generation does not apply to prepare.
- `publication`: replaced by pre-primary `initial_put` and post-primary `reserved_put`; no old INITIAL publisher survives primary entry.
