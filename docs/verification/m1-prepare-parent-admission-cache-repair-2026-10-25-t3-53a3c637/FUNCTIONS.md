# Whole-function and entry/global inventory

SOURCE/DATA only. execution=false. Exact baseline: `06634b58f1b4795684b9139de716b4fe8f6430e0`.

Changed functions: `import_common`, `collect`, `main`. No function was added or removed. All 41 other declaration ranges (40 functions plus Stop) retain their exact bytes and hashes.

Full old and new changed declarations are in [changed-functions.before.txt](changed-functions.before.txt) and [changed-functions.after.txt](changed-functions.after.txt). Ranges include decorators and the terminating newline. Nested functions have separate entries.

Entry code, imports, all globals/literals, main guard and every inter-declaration byte are unchanged. [module-layout.json](module-layout.json) binds every such physical segment before and after. No entry/global change is omitted.

| Declaration | State | Old lines | New lines | New SHA-256 |
|---|---|---|---|---|

| Stop | unchanged | 929..930 | 929..930 | fd5f0e9e870ae08920c1d407fc73b253c407fdd0aef25c908048f181025ae7f5 |

| absent | unchanged | 1217..1222 | 1217..1222 | 947f6b50c9344014a2e2d2a3cc77b865affc8ba26d46a98c5b8bbcc62d600b0f |

| bind_config | unchanged | 1168..1208 | 1168..1208 | b04c520e2b268acf889f27957d315eb381a7a128a1ff8140fd17b05a583a79ec |

| canonical_prepared_directory | unchanged | 1516..1532 | 1519..1535 | 0eba03c23e2c12eafe1e1a13d91721ed671ead77aaaaadd2f4725d575ff6bf1d |

| clock_ns | unchanged | 938..943 | 938..943 | 0a708be90d476ccafb3f658a6edeae5a969cd1fed47fc9e4568632c78144e7bf |

| closed_json | unchanged | 1152..1165 | 1152..1165 | 019219040de9c1e8fb305ae0ee2e61a3335591568ac79d70de9011d53b77b2eb |

| closed_json.finite_float | unchanged | 1161..1164 | 1161..1164 | 580f75ecd092de39c675e9c790c854f42fe72decd96690c24b15ad302f5d7b85 |

| closed_json.nonfinite | unchanged | 1159..1160 | 1159..1160 | f32a3a64e87b6448d7ef0b65c9c39395f23789cd334acc2948a07eb81d192925 |

| closed_json.pairs | unchanged | 1153..1158 | 1153..1158 | 0631fc06468cfa7e32fdd726ce494a810b7000bdc90d5bc09083833869579093 |

| collect | changed | 1580..1714 | 1583..1717 | 3608cc825e086f1fb0b7f901d7a759a456bdcde375bcff0e6b155facf493bf47 |

| collection_complete | unchanged | 1140..1149 | 1140..1149 | 1e392aa0f5b155a79c614f7f8b967bf8edeefba7fb7729ccaa3ceacc25c6a793 |

| common_deadline | unchanged | 1450..1465 | 1453..1468 | 1d8dcb38b35c9dd3390e5275b12940e0d8b5662ccf01a52aa230ad8a87da6b72 |

| console_admission | unchanged | 1225..1235 | 1225..1235 | a5447dfce8fef6cac5259f0315182e0d0c206e8a9cfe50b8ffe280ef029e41fe |

| emit_once | unchanged | 1717..1727 | 1720..1730 | 548d547e0d17682cc006964f4d75fb3e6f7aa3132576600fc87e1727c9349cc7 |

| encoded | unchanged | 975..977 | 975..977 | 47546d7a82bbb5e109d0a86a34e3046f3adab68837513ddf922600b33ce202d2 |

| existing_ledger | unchanged | 1468..1479 | 1471..1482 | f14b8a32e7ffa286b368e1e3fb4e870b85200b59a20fac15b6a87e1b94869d81 |

| fail | unchanged | 1211..1214 | 1211..1214 | de1bd3450d76eb0036a1f40c2d29b91d542d1715469a2bb1256f8f309997adcc |

| hex_string | unchanged | 980..982 | 980..982 | fd9f19e03ed808aeded45d465053a19b34fd34724e58ee0d2c21641f1df07d78 |

| import_common | changed | 1429..1447 | 1429..1450 | 30e74482054209866fee6e7ff0673a5b847a4ce27f457fddfbb931ba5d3afbd8 |

| initial_put | unchanged | 1366..1403 | 1366..1403 | 7b3fdc887bb007e6ae1e7378f0599cd602d7dfec450a1f2e7fcd3fa7ff493249 |

| inspect_artifact | unchanged | 1284..1314 | 1284..1314 | 1824e62a3616f2a40b03f3d9316fa1c96120a00626ab308ffff2f14150185c30 |

| inspect_entry | unchanged | 1406..1426 | 1406..1426 | d89e380c42166abef964b867a3e431cecb96cda8e5bad6919424a27dd594cc51 |

| inspect_frozen | unchanged | 1317..1363 | 1317..1363 | 884c872a746f513558473b04be39ad4fa04013a88e66a73c4dff3eb0fc6e2504 |

| inspect_prepared | unchanged | 1535..1579 | 1538..1582 | 474b6b43175cd5c662109f02ac24cf1e9148a82d1047b785cbb1c355e13b7636 |

| inspect_remote | unchanged | 1042..1076 | 1042..1076 | 1def34c6cbdaffd2021c2ff805c428fcef4a11d3a2ce2bebb4fe512149a4c47b |

| inventory | unchanged | 1079..1114 | 1079..1114 | 12c46f1c5b191b7fdd68455b26c33fc259c4119b9b3b5b9647a93a5737ebb22f |

| inventory.walk | unchanged | 1082..1112 | 1082..1112 | 14509512ab33bd3066c48331714922123e25b543cae89ca20eb2ed0ffe8845aa |

| latch | unchanged | 1125..1127 | 1125..1127 | 1fc81f469b2139c30fcccfcc0ba115eec20e8deb4cd190d363f56bf31fe80640 |

| main | changed | 1730..1856 | 1733..1864 | 27a555d64a281a6d4ed7019c6e989a928b78df8b17203704603496dc1075dcf5 |

| observe_exit | unchanged | 1130..1137 | 1130..1137 | 94a6fafecbef10b6509533a0d792cce1fc61b4fcb99d9183118da1916b83610c |

| open_root | unchanged | 1001..1017 | 1001..1017 | c7aee36ba2ba05b59397e13a30160ffc77a94a2360d705ff22a54dfe69ae163f |

| read_build_record | unchanged | 1260..1273 | 1260..1273 | 6ea479ece901fb742b3e22a71d973af62d23dc35897fa04bb33f256f2fc4a8fa |

| read_leaf | unchanged | 1020..1039 | 1020..1039 | 963beb17880dcd6f322370c6df7a7d2a0c1cf573b049cbcf990faf02e4af389f |

| read_new_leaf | unchanged | 1238..1257 | 1238..1257 | 2b9c6970ae56804639e26a755b436528d84ae41d4ae0df09f3cea42d86b5b46c |

| reference | unchanged | 1276..1281 | 1276..1281 | fd11ab41a612c579f21d048250603a916429d50a7022e540a71a0bb87f698fe3 |

| remaining | unchanged | 946..949 | 946..949 | b723e8885f1273197aa575682bee963f2cefbec7b0df8c8b6a051a0e26ac7c78 |

| require | unchanged | 933..935 | 933..935 | fa57bb7c8de0a6f63c620d5d56849a58849c546317137baa3463c4ce8f7d4766 |

| reserved_put | unchanged | 1482..1513 | 1485..1516 | 2af9e40a0c20195b3e4cb9fbd2073581c70d05a502cccba443a7eee67c945ae0 |

| same_data | unchanged | 985..993 | 985..993 | b03e25037d583929f50a0cc97c4069c4e211552782a8f14037cba370a49734fe |

| sha | unchanged | 971..972 | 971..972 | 4e627faf4b903521e935f7c6f3ae93545856f821d0e2565ba7ea6cf97d3d0e6c |

| stamp | unchanged | 996..998 | 996..998 | 5ad54262f0589d8245e2a4e1d0fc970a2264bcd8ad3c9eee89e66022b017e1c0 |

| startup | unchanged | 1117..1122 | 1117..1122 | 77f6e5efc6f48392d70f305e7f8be86b6d20f82574df1cff818d28389c088bd9 |

| until | unchanged | 952..968 | 952..968 | 00ff5b79c8d3acd8bbcd3e298fb472fab7427f91611e93116c199224033746ff |

| until.expired | unchanged | 959..960 | 959..960 | bc1ec102edf0eae776d4cdfa698bb0528d7764f751efecc51a5cd16bd1bd3505 |
