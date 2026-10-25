# Cache exclusion and loader scope

execution=false. This is source/library text analysis. No supplied module,
interpreter, cache, helper, preparer or checker was entered.

PREPARE-P2-2 is addressed by absence gates on three fixed names in ROOT:
`__pycache__`, `common.pyc`, `common.pyo`. A name of any type, including a
symlink, rejects. `absent` uses `os.stat(..., follow_symlinks=False)` relative
to the retained root FD. Only FileNotFoundError admits. No cache is removed,
repaired, opened as code or retried. Root ownership stays exclusive through exit
under the existing mandatory CONFIG parent duty. The gates do not implement a
new filesystem lock or prove absence against a concurrent same-user mutator.

Placement in the repaired complete source:

1. main's initial absence loop follows inspect_frozen, before inventory, guard
   growth, SYSTEM helper, common or private entry.
2. import_common repeats all17 source pins, then checks all three names before
   spec_from_file_location/module_from_spec/exec_module.
3. main repeats final artifact/source pins and absent-ledger checks, then checks
   all three names again before primary_attempted=True and collect('prepare').

SYSTEM's explicit `.py` spec selects SourceFileLoader. Its get_code can accept
compatible bytecode before it obtains source bytes. Timestamp validity or an
unchecked hash flag is not the SHA-256 pin. dont_write_bytecode/-B only prevents
writes. The primary's unchanged `from common import *` also uses ordinary import
machinery. The root directory absence covers the ordinary cache paths for the
fixed common.py in both runtimes, including:

- `ROOT/__pycache__/common.cpython-39.pyc`
- `ROOT/__pycache__/common.cpython-313.pyc`
- every optimization-tag variant and timestamp/checked/unchecked hash form
  under that directory, including any other compatible tag.

For the explicit SYSTEM source loader, root `common.pyc` is not the derived
cache path. For ordinary file-module lookup, source suffixes precede bytecode
suffixes; with the pinned common.py present, root common.pyc is not selected
in place of it. Sourceless `.pyc` lookup matters if source is absent, which the
repeated pin reads already reject. `.pyo` is not in normal CPython3.9/3.13
BYTECODE_SUFFIXES; optimized Python3 caches also use `.pyc` under __pycache__.
The two legacy names are conservative extra exclusions, not claims that either
would win loader selection in this exact admitted layout.

The local public CPython3.9 `_bootstrap_external.py` was read only as text.
Its exact path/64569-byte identity, SHA-256 and line ranges are in bindings.json.
The relevant text is cache_from_source348..435, spec_from_file_location696..748,
exec_module844..850, get_code916..998, FileFinder1515..1558 and loader-order
1634..1644. The review's exact source identity matches. This local text is not
a fresh qualification of the M1's3.13 implementation or kernel.

The fixed SYSTEM `-I -S -B -` and private `-B prepare.py` vectors, all ten startup
keys and their values are unchanged. No `-X pycache_prefix` or
PYTHONPYCACHEPREFIX is introduced or admitted by that closed startup map.
Normal common.py cache derivation assumes the bound trusted interpreters and
standard fixed startup behavior with no redirected cache prefix or custom import
hook. This packet does not qualify arbitrary shadow packages/extensions,
modified import machinery or a nonstandard interpreter. Exact runtime/prefix
and actual source/layout assessment remain external parent duties.

Trusted SYSTEM standard-library caches outside ROOT and caches in the
previously qualified private prefix remain in the trusted runtime scope. This
repair neither disables their reads nor authenticates every standard-library
cache anew. It adds no source-only loader and changes no standard-library or
upstream supplied module bytes. The new gate is limited to root common caches
for the fixed layout. Actual initial root absence and continued exclusivity are
future observations. No actual M1 cache was read or asserted present/absent.
