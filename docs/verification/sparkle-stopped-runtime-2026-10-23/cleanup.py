#!/usr/bin/env python3
"""Remove only exact fixture cache roots absent in this case's saved baseline."""
from pathlib import Path
import importlib.util
import json
import os
import shutil
import sys

sys.dont_write_bytecode = True
path = Path(__file__).resolve().with_name('controller.py')
spec = importlib.util.spec_from_file_location('controller', path)
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)
case = sys.argv[1]
assert case in ['negative', 'positive']
directory = c.OUT / (case + '-cli-once')
output = c.OUT / (case + '-cleanup.json')
assert not output.exists(), 'Cleanup cannot repeat'
baseline = json.loads((directory / 'preflight.json').read_text())
assert not baseline['processes'] and not baseline['jobs']
assert all(not x['exists'] and not x['symlink'] for x in baseline['caches'])
active = c.processes()
active_jobs = c.jobs()
before = c.cache_snapshot()
receipt = {'started_at': c.now(), 'processes_before': active, 'jobs_before': active_jobs,
           'cache_snapshot_before': before, 'actions': [],
           'policy': 'Only exact roots with saved absence baseline, matching fixture ID and current UID; no shared state reversal'}
try:
    assert not active, 'Fixture processes remain; refuse cache deletion'
    assert not active_jobs, 'Fixture jobs remain; preserve for exact job ownership inspection'
    for entry in before:
        if not entry['exists']:
            continue
        root = Path(entry['path'])
        assert root in c.CACHES and root.name in [c.ID, c.ID + '.driver']
        assert not root.is_symlink() and root.stat().st_uid == os.getuid()
        # Preserve and bind the full tree before deleting only this exact owned root.
        action = {'path': str(root), 'uid': root.stat().st_uid,
                  'inode': root.stat().st_ino, 'tree': c.inventory(root, directories=True)}
        receipt['actions'].append(action)
        c.save(output, receipt)
        shutil.rmtree(root)
        action['removed_at'] = c.now()
    receipt.update(processes_after=c.processes(), jobs_after=c.jobs(),
                   caches_after=c.cache_snapshot(), completed_at=c.now(),
                   shared_launchservices_or_gatekeeper_reversed=False)
except Exception as e:
    receipt['error'] = repr(e)
    raise
finally:
    c.save(output, receipt)
print(json.dumps({'case': case, 'removed': [a['path'] for a in receipt['actions']],
                  'remaining_processes': receipt.get('processes_after'),
                  'remaining_jobs': receipt.get('jobs_after'), 'receipt': str(output)}, indent=2))
