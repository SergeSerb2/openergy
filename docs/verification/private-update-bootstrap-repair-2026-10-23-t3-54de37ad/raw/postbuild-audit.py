#!/usr/bin/env python3
"""Read-only reconciliation after the first build controller assertion.

Does not run either product, packaging, a compiler, or any build command.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess

REPO = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
BASE = '1fd5144628c0494f1ebf561581e0d84fd7231523'
HELD = 'a0342aef6b4e4712e7bf73c992b788f26d0e98da'
PIN = 'eef1a539a373c1f1a320624b1130fc5de7b2e100'


def sha(bytes): return hashlib.sha256(bytes).hexdigest()


def save(name, data):
    with (OUT / name).open('x') as stream:
        json.dump(data, stream, indent=2, sort_keys=True)
        stream.write('\n')


def git(*args):
    return subprocess.check_output(['git', *args], cwd=REPO, env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0'})


def main():
    preparation = json.loads((OUT / 'preparation.json').read_text())
    root = Path(preparation['private_root'])
    original = Path(preparation['historical_cache'])
    values = {name: json.loads((OUT / (name + '.json')).read_text()) for name in
              ['cache-before', 'cache-copy', 'cache-after', 'cache-copy-after']}
    assert values['cache-before'] == values['cache-copy'] == values['cache-after']
    before = {entry['path']: entry for entry in values['cache-copy']}
    after = {entry['path']: entry for entry in values['cache-copy-after']}
    differences = [dict(path=p, before=before.get(p), after=after.get(p))
                   for p in sorted(before.keys() | after.keys()) if before.get(p) != after.get(p)]
    assert len(differences) == 1 and differences[0]['path'] == 'checkouts/Sparkle/.git/index'
    # Read the indexes as Git staged-object lists, with optional index writes off.
    indexes = []
    for base in [original, root / 'swift']:
        checkout = base / 'checkouts/Sparkle'
        staged = git('-C', str(checkout), '-c', 'core.fsmonitor=false', 'ls-files', '--stage', '-z')
        head = git('-C', str(checkout), 'rev-parse', 'HEAD').decode().strip()
        assert head == PIN
        indexes.append(dict(root=str(base), head=head, staged_objects_sha256=sha(staged),
                            tracked_records=staged.count(b'\0')))
    assert indexes[0]['staged_objects_sha256'] == indexes[1]['staged_objects_sha256']
    assert sha((original / 'checkouts/Sparkle/.git/index').read_bytes()) == before['checkouts/Sparkle/.git/index']['sha256']
    assert sha((root / 'swift/checkouts/Sparkle/.git/index').read_bytes()) == after['checkouts/Sparkle/.git/index']['sha256']
    save('cache-reconciliation.json', dict(first_controller_exit=1,
        failure='strict complete cache equality failed after both compile/link exits 0',
        rerun=False, original_cache_records=len(values['cache-before']), original_selected_cache_unchanged=True,
        copied_records=len(before), copied_differences=differences, index_semantics=indexes,
        interpretation='private Git index metadata changed; staged paths/modes/object IDs and all source/artifact bytes unchanged',
        parent_advisory='Parent reported 114 old swift/out removals from a different copied build database. No old out/build database was copied here; no parent cache was repaired or written.',
        parent_advisory_effect_on_selected_inventories='none: original before/copy/original after records match exactly',
        original_workspace_state_current_sha256=sha((original / 'workspace-state.json').read_bytes()),
        workspace_state_limit='controller did not save its in-memory prebuild workspace-state hash before assertion; no new before/after state-file claim',
        first_streams_preserved=True))
    products = [root / 'launcher/OpenergyPrivateLauncher', root / 'swift/arm64-apple-macosx/debug/Openergy']
    audits = []
    for product in products:
        for tool, args in [('otool', ['-l']), ('otool', ['-L']), ('file', [])]:
            name = product.name + '-' + tool + ''.join(args).replace('-', '')
            command = ['/usr/bin/' + tool] + args + [str(product)]
            result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
            (OUT / (name + '.stdout')).write_bytes(result.stdout)
            (OUT / (name + '.stderr')).write_bytes(result.stderr)
            audits.append(dict(argv=command, exit=result.returncode, stdout_sha256=sha(result.stdout),
                               stderr_sha256=sha(result.stderr)))
            assert result.returncode == 0
    libraries = (OUT / 'OpenergyPrivateLauncher-otoolL.stdout').read_text()
    assert '\t/usr/lib/libSystem.B.dylib ' in libraries and len(libraries.strip().splitlines()) == 2
    macho = (OUT / 'OpenergyPrivateLauncher-otooll.stdout').read_text()
    assert 'minos 15.0' in macho and 'LC_RPATH' not in macho and 'LC_DYLD_ENVIRONMENT' not in macho
    assert 'LC_MAIN' in macho
    save('artifact-audit.json', dict(kind='file and Mach-O load-command reads; neither executable invoked',
        commands=audits, launcher_libraries=['/usr/lib/libSystem.B.dylib'],
        launcher_minimum_os='15.0', template_header_sha256=sha((OUT / 'private-info-template.h').read_bytes()),
        products=[dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p.read_bytes())) for p in products]))
    sources = json.loads((OUT / 'build-source.json').read_text())
    assert all(sha((REPO / row['path']).read_bytes()) == row['sha256'] for row in sources)
    protected = ['apps/macos/Sources/Openergy/App/AppUpdater.swift', 'apps/macos/Sources/Openergy/App/AppEndToEnd.swift',
        'apps/macos/Sources/Openergy/App/UpdaterEndToEnd.swift', 'apps/macos/Sources/Openergy/Model/AppModel.swift',
        'apps/macos/Sources/Openergy/Daemon/DaemonSupervisor.swift', 'apps/macos/Package.swift',
        'apps/macos/Package.resolved', 'apps/macos/Support/Info.plist', 'apps/macos/scripts/bundle.sh']
    unchanged = []
    for path in protected:
        current = (REPO / path).read_bytes()
        assert current == git('show', BASE + ':' + path)
        unchanged.append(dict(path=path, sha256=sha(current), same_as=BASE))
    original_evidence = []
    for path in sorted((REPO / 'docs/verification/private-update-bootstrap-2026-10-23').rglob('*')):
        if path.is_file():
            relative = str(path.relative_to(REPO))
            assert path.read_bytes() == git('show', HELD + ':' + relative)
            original_evidence.append(dict(path=relative, sha256=sha(path.read_bytes())))
    assert (REPO / 'apps/macos/Sources/Openergy/App/OpenergyApp.swift').read_bytes() == git('show', HELD + ':apps/macos/Sources/Openergy/App/OpenergyApp.swift')
    summary = []
    for name in ['launcher', 'app']:
        result = json.loads((OUT / (name + '-result.json')).read_text())
        samples = json.loads((OUT / (name + '-priority.json')).read_text())
        assert result['exit'] == 0 and not result['timed_out'] and {s['nice'] for s in samples} == {15}
        assert sha((OUT / (name + '.stdout')).read_bytes()) == result['stdout_sha256']
        assert sha((OUT / (name + '.stderr')).read_bytes()) == result['stderr_sha256']
        summary.append(dict(name=name, observations=len(samples), pids=len({s['pid'] for s in samples}),
            nice=15, swift_frontend_observations=sum('swift-frontend' in s['executable'] for s in samples),
            clang_observations=sum('clang' in s['executable'] for s in samples)))
    save('source-audit.json', dict(compiled_sources_still_match=True, source_records=len(sources),
        swift_sources=sum(r['path'].endswith('.swift') and '/Sources/' in r['path'] for r in sources),
        source_inventory_sha256=sha(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()),
        protected_unchanged=unchanged, inherited_entry_unchanged=True, inherited_evidence=original_evidence,
        sample_summary=summary, source_only_limits='No source review substitutes for runtime reproduction. No packaging or product execution.',
        scope='SOURCE + WHOLE-APP BUILD ONLY'))
    print(json.dumps(dict(artifact_audit=True, source_audit=True, original_selected_cache_unchanged=True,
                         private_copy_difference='Git index metadata only', compiler_rerun=False), sort_keys=True))


if __name__ == '__main__':
    main()
