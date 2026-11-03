"""Read-only checks on a published archive extracted into a private run directory."""
from pathlib import Path
import hashlib
import json
import os
import plistlib
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

root = Path(sys.argv[1]).resolve()
assert str(root).startswith('/private/tmp/oe-candidate-20261103.')
assert root.stat().st_mode & 0o077 == 0 and root.stat().st_uid == os.getuid()
app = root / 'extracted/Openergy.app'
archive = root / 'Openergy-0.1.0-macos-universal.zip'
records = []


def run(args, timeout=45):
    began = time.monotonic()
    try:
        result = subprocess.run(args, cwd=root, capture_output=True, text=True, timeout=timeout)
        record = dict(arguments=args, exit=result.returncode, stdout=result.stdout,
                      stderr=result.stderr, elapsed_seconds=round(time.monotonic()-began, 3))
    except subprocess.TimeoutExpired:
        record = dict(arguments=args, exit=None, timeout_seconds=timeout)
    records.append(record)
    return record


info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
run(['sw_vers'])
run(['uname', '-m'])
run(['shasum', '-a', '256', '-c', str(archive)+'.sha256'])
for target in (archive, app):
    run(['xattr', '-p', 'com.apple.quarantine', str(target)])
for target in (app, app/'Contents/Helpers/openergy', app/'Contents/Frameworks/Sparkle.framework'):
    run(['codesign', '--verify', '--strict', '--verbose=4', str(target)])
    run(['codesign', '-dvvv', '--entitlements', ':-', str(target)])
run(['codesign', '--verify', '--deep', '--strict', '--verbose=4', str(app)])
run(['xcrun', 'stapler', 'validate', '-v', str(app)])
run(['spctl', '--assess', '--type', 'execute', '--verbose=4', str(app)])
run(['file', str(app/'Contents/MacOS/Openergy'), str(app/'Contents/Helpers/openergy')])
run(['lipo', '-archs', str(app/'Contents/MacOS/Openergy')])
run(['lipo', '-archs', str(app/'Contents/Helpers/openergy')])
run([str(app/'Contents/Helpers/openergy'), '--version'])
feed = run(['curl', '--silent', '--show-error', '--location', '--connect-timeout', '10',
            '--max-time', '30', '--output', str(root/'appcast.xml'), '--write-out',
            '%{http_code}', info['SUFeedURL']])
feed_path = root/'appcast.xml'
feed_result = dict(url=info['SUFeedURL'], curl_exit=feed['exit'], http_status=feed.get('stdout'))
if feed_path.exists():
    data = feed_path.read_bytes()
    feed_result.update(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
    try:
        tree = ET.fromstring(data)
        feed_result['release_count'] = len(tree.findall('./channel/item'))
        feed_result['releases'] = [ET.tostring(item, encoding='unicode') for item in tree.findall('./channel/item')]
    except ET.ParseError:
        feed_result['release_count'] = None
        feed_result['parse_error'] = True
files = []
for path in sorted(app.rglob('*')):
    if path.is_file() and not path.is_symlink():
        files.append(dict(path=str(path.relative_to(app)), bytes=path.stat().st_size,
                          sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
result = dict(archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest(),
              archive_bytes=archive.stat().st_size, info_plist=info, commands=records,
              feed=feed_result, files=files)
raw = json.dumps(result, indent=2)
(root/'artifact-inspection.private.json').write_text(raw+'\n')
sanitized = raw.replace(str(root), '<private-run>').replace(str(Path.home()), '<test-home>')
sanitized = re.sub(r'/var/folders/[^\s"\\]+', '<system-temp>', sanitized)
print(sanitized)
