#!/usr/bin/env python3
"""Read-only feed GETs on the authorized macOS host and m1-dev SSH target.
No application, desktop, daemon, preferences, or credentials are changed.
"""
import argparse
import json
import subprocess
from pathlib import Path

# Only JSON summaries return over the bridge. Temporary response files are deleted.
PAYLOAD = r'''
import datetime, hashlib, json, os, platform, subprocess, tempfile
from pathlib import Path
import xml.etree.ElementTree as ET
report = {'os_build': subprocess.check_output(['sw_vers'], text=True),
          'architecture': platform.machine(), 'requests': []}
for agent in (None, 'Openergy/0.1.0 Sparkle/2.10.0'):
    with tempfile.TemporaryDirectory(prefix='oe-feed-get-') as tmp:
        body = Path(tmp) / 'body'
        command = ['curl', '--disable', '--silent', '--show-error', '--proto', '=https',
                   '--max-time', '30', '--max-filesize', '1048576', '--header', 'Cache-Control: no-cache',
                   '--output', str(body), '--write-out', '%{http_code}',
                   'https://openergy-updates.sergeserbinenko.com/appcast.xml']
        if agent:
            command += ['--user-agent', agent]
        started = datetime.datetime.now(datetime.timezone.utc).isoformat()
        result = subprocess.run(command, capture_output=True, text=True,
                                env={k: v for k, v in os.environ.items() if k in ('PATH', 'HOME', 'TMPDIR')})
        data = body.read_bytes() if body.exists() else b''
        count = None
        if result.stdout == '200':
            try:
                rss = ET.fromstring(data)
                count = len(rss.findall('channel/item')) if rss.tag == 'rss' and rss.find('channel') is not None else None
            except ET.ParseError:
                pass
        report['requests'].append({'started_utc': started, 'user_agent': agent or 'curl default',
                                   'exit': result.returncode, 'http_status': result.stdout,
                                   'response_bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(),
                                   'release_count': count, 'stderr': result.stderr})
print(json.dumps(report))
'''

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
if args.output.exists():
    parser.error('select a fresh output path')
results = []
for label, command in [('host', ['mac', 'python3', '-']),
                       ('m1-dev', ['mac', 'ssh', '-o', 'BatchMode=yes', 'm1-dev', 'python3', '-'])]:
    result = subprocess.run(command, input=PAYLOAD, text=True, capture_output=True, timeout=75)
    results.append({'client': label, 'command': command, 'exit': result.returncode,
                    'report': json.loads(result.stdout) if result.returncode == 0 else None,
                    'stderr': result.stderr})
with args.output.open('x') as f:
    json.dump(results, f, indent=2)
    f.write('\n')
args.output.chmod(0o600)
print(json.dumps(results))
