"""Copy the real catalogue into a lab whose only TCP ports are 24820-24829."""
import pathlib
import shutil
import subprocess
import sys

source = pathlib.Path(sys.argv[1]).resolve()
lab = pathlib.Path(sys.argv[2]).resolve()
if lab.parent != pathlib.Path('/tmp') or not lab.name.startswith('oe-win-port-'):
    raise SystemExit('destination must be a new /tmp/oe-win-port-* directory')
lab.mkdir(mode=0o700)
archive = subprocess.Popen(['git', 'archive', 'HEAD'], cwd=source, stdout=subprocess.PIPE)
subprocess.run(['tar', '-x', '-C', str(lab)], stdin=archive.stdout, check=True)
archive.stdout.close()
if archive.wait():
    raise SystemExit('git archive failed')

def replace(path, before, after):
    path = lab / path
    text = path.read_text()
    if before not in text:
        raise SystemExit('lab guard changed: ' + str(path))
    path.write_text(text.replace(before, after))

replace('tools/e2e/src/main.rs', 'options.lab_range.0 < 24900', 'options.lab_range.0 < 24825')
replace('tools/interop-lab/lab.sh', 'first >= 24900', 'first >= 24825')
replace('tools/interop-lab/lab.sh', '    mkdir -p "$ROOT/.state"',
        '    mkdir -p "$ROOT/.state"\n    cp "$ROOT/container.sh" "$ROOT/.state/port-peer.sh"')
replace('tools/interop-lab/lab.sh', '"$CONTAINER" lab-peer "$cmd"',
        '"$CONTAINER" bash /lab/port-peer.sh "$cmd"')
replace('tools/interop-lab/container.sh', '10#$1 >= 24900', '10#$1 >= 24820')
shutil.copy2(source / 'target/release/openergy', lab / 'openergy')
print(lab)
