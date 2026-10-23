#!/usr/bin/env python3
"""Generate distributable notices from the locked, cached Cargo package sources.

Run at the repository root. This command does not build or execute Openergy.
Unknown license layouts fail for manual review instead of losing notices.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
ASN1_URL = ('https://raw.githubusercontent.com/rusticata/asn1-rs/'
            'a20e5f7319c896737ad0f2557037817b91ad854f/LICENSE-MIT')
X11_URL = ('https://github.com/freedesktop-unofficial-mirror/'
           'xorg__proto__x11proto/blob/'
           '6d4acb0e3a6568a8faaa651d4e3d32f917b9067b/keysymdef.h')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def package_notices(package):
    root = Path(package['manifest_path']).parent
    files = sorted(p for p in root.iterdir() if p.is_file())
    mit = [p for p in files if p.name.lower() in ('license-mit', 'license-mit.txt')]
    if package['name'] == 'ring':
        # ring's root LICENSE delegates to BoringSSL, fiat, and once_cell.
        selected = sorted(p for p in root.rglob('*') if p.is_file()
                          and p.name.upper().startswith(('LICENSE', 'NOTICE')))
    elif mit:
        selected = mit
        if 'Unicode-3.0' in package['license']:
            selected += [root / 'LICENSE-UNICODE']
    elif package['name'] == 'asn1-rs-impl' and package['version'] == '0.2.0':
        # This .crate omitted the license files. Use its recorded VCS revision.
        vcs = json.loads((root / '.cargo_vcs_info.json').read_text())
        if vcs['git']['sha1'] != 'a20e5f7319c896737ad0f2557037817b91ad854f':
            raise ValueError('asn1-rs-impl source revision changed')
        p = HERE / 'licenses/asn1-rs-impl-0.2.0-MIT.txt'
        return [('upstream LICENSE-MIT', p.read_bytes(), ASN1_URL)]
    else:
        selected = [p for p in files if p.name.lower() in
                    ('license', 'license.md', 'license.txt', 'copying')]
    selected += [p for p in files if p.name.upper().startswith('NOTICE')]
    selected = sorted(set(selected))
    if not selected:
        raise ValueError('manual license review required: ' + package['name'])
    return [(p.relative_to(root).as_posix(), p.read_bytes(), None) for p in selected]


def swift_notices():
    manifest = (ROOT / 'apps/macos/Package.swift').read_text()
    lock = ROOT / 'apps/macos/Package.resolved'
    if not lock.exists():
        if re.search(r'\.package\s*\(', manifest):
            raise ValueError('Swift package dependencies require a committed Package.resolved')
        return [], {}, None
    pins = json.loads(lock.read_bytes()).get('pins', [])
    policy = json.loads((HERE / 'licenses/Sparkle-2.10.0.json').read_bytes())
    if len(pins) != 1:
        raise ValueError('manual Swift dependency inventory review required')
    pin = pins[0]
    if (pin.get('identity') != 'sparkle'
            or pin.get('location', '').removesuffix('.git') != policy['source_url']
            or pin.get('state', {}).get('version') != policy['version']
            or pin.get('state', {}).get('revision') != policy['revision']):
        raise ValueError('Swift dependency differs from the reviewed Sparkle release')
    declaration = (r'\.package\s*\(\s*url:\s*"https://github\.com/sparkle-project/Sparkle(?:\.git)?"'
                   r'\s*,\s*exact:\s*"2\.10\.0"\s*\)')
    if not re.search(declaration, manifest):
        raise ValueError('Sparkle must use the reviewed exact Swift package version')
    license_file = HERE / 'licenses/Sparkle-2.10.0-LICENSE.txt'
    data = license_file.read_bytes()
    digest = sha(data)
    if digest != policy['license_sha256']:
        raise ValueError('Sparkle upstream license bytes differ from the reviewed release')
    package = {'name': policy['name'], 'version': policy['version'],
               'declared_license': policy['license_expression'],
               'source': 'swiftpm+' + policy['source_url'], 'url': policy['source_url'],
               'upstream_revision': policy['revision'],
               'binary_artifact_sha256': policy['binary_artifact_sha256'],
               'license_files': [{'path': 'LICENSE', 'sha256': digest,
                                  'source_url': policy['license_source_url']}]}
    return [package], {digest: data.decode('utf-8')}, sha(lock.read_bytes())


def generate(metadata):
    texts = {}
    packages = []
    for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
        source = package.get('source')
        package_url = 'https://crates.io/crates/' + package['name'] + '/' + package['version']
        upstream = None
        if package.get('source') is None:
            if package['id'] in metadata['workspace_members']:
                continue
            # A path dependency can still be third-party source. Fail closed for
            # unknown paths instead of silently dropping their license notices.
            manifest = Path(package['manifest_path']).resolve()
            if (package['name'], package['version'], manifest) != (
                    'mdns-sd', '0.21.6', ROOT / 'vendor/mdns-sd/Cargo.toml'):
                raise ValueError('manual path source review required: ' + package['name'])
            upstream = json.loads((manifest.parent / 'UPSTREAM.json').read_text())
            source = 'path+vendor/mdns-sd'
            package_url = 'vendor/mdns-sd/OPENERGY-PATCH.md'
        elif source != 'registry+https://github.com/rust-lang/crates.io-index':
            raise ValueError('manual source review required: ' + package['name'])
        if not package.get('license'):
            raise ValueError('missing license: ' + package['name'])
        licenses = []
        for name, data, url in package_notices(package):
            digest = sha(data)
            text = data.decode('utf-8')
            texts[digest] = text
            entry = {'path': name, 'sha256': digest}
            if url:
                entry['source_url'] = url
            licenses.append(entry)
        entry = {'name': package['name'], 'version': package['version'],
                         'declared_license': package['license'],
                         'source': source, 'url': package_url, 'license_files': licenses}
        if upstream:
            entry['upstream_archive_sha256'] = upstream['archive_sha256']
            entry['upstream_revision'] = upstream['revision']
            entry['locally_modified'] = True
        packages.append(entry)
    if not packages:
        raise ValueError('empty external dependency inventory')
    swift_packages, swift_texts, swift_lock_sha256 = swift_notices()
    packages.extend(swift_packages)
    texts.update(swift_texts)
    x11 = (HERE / 'licenses/X11-keysymdef.txt').read_bytes()
    texts[sha(x11)] = x11.decode('utf-8')
    intro = '''# Third-party notices

Openergy's own work is available under MIT OR Apache-2.0. See LICENSE-MIT
and LICENSE-APACHE. Third-party material keeps its own terms below.

This inventory covers every external package resolved by the root Cargo.lock
and the reviewed macOS Package.resolved, including other platforms, build tools,
and development dependencies. It is a
conservative source inventory, not a claim that each package is linked into
every binary. For dual-licensed packages with a separate MIT file, the MIT
option is used here. Combined license files and ring's delegated notices are
retained in full. Unicode's additional terms are retained with unicode-ident.
The table preserves each package's original license expression.
The mdns-sd path dependency is a locally modified copy of 0.21.6. Its linked
patch record identifies the original archive and the local source delta.

The macOS app's pinned Sparkle binary framework retains the complete upstream
license and bundled external notices. Apple and Windows system frameworks and
the Rust toolchain are not vendored by this repository.
The SVG icon and raster verification fixtures have no embedded third-party
artwork attribution found in the tracked files; this is not an authorship proof.

Synergy, Deskflow, Barrier, and Input Leap are separate products used as
compatibility references or optional lab peers. Their names do not imply
endorsement. Their executables, bundles, fonts, source maps, and private TLS
material must not be added to Openergy release archives. The optional Docker
lab installs Deskflow and Barrier under their upstream licenses; this file does
not license that image or replace its GPL source-distribution obligations.
Synergy App Ltd is the current vendor name in its published EULA (2026-09-23),
formerly Symless. See https://symless.com/synergy/legal/eula.

Keep this file with binary distributions. Regenerate after dependency changes:

```sh
python3 docs/verification/public-release-audit-2026-10-21/generate-notices.py
```

## Dependency inventory

| Package | Declared license | License text SHA-256 prefixes |
| --- | --- | --- |
'''
    rows = []
    for package in packages:
        refs = ', '.join('[' + f['sha256'][:12] + '](#text-' + f['sha256'][:12] + ')'
                         for f in package['license_files'])
        rows.append('| [' + package['name'] + ' ' + package['version'] + ']('
                    + package['url'] + ') | ' + package['declared_license'] + ' | ' + refs + ' |')
    body = intro + '\n'.join(rows) + '\n\n## X11 keysym data\n\n'
    body += ('`crates/platform-linux/src/unicode_keysyms.rs` identifies Xorg\n'
             '`keysymdef.h` as its source of numeric mappings. Retain the Open Group\n'
             'and Digital Equipment notices with these derived data. The original\n'
             'generator and exact header revision were not recorded. The permission\n'
             'header below was checked against the X.org historical mirror:\n\n'
             + X11_URL + '\n\n[License text](#text-' + sha(x11)[:12] + ').\n\n'
             '## License texts\n\n'
             'Texts with identical SHA-256 are included once. The machine-readable\n'
             'inventory records the source filename and full hash of its original bytes.\n'
             'Only trailing whitespace is removed from the Markdown text rendering.\n\n')
    for digest, text in sorted(texts.items()):
        rendered = '\n'.join(line.rstrip() for line in text.splitlines()).rstrip('\n')
        body += '### text-' + digest[:12] + '\n\n```text\n' + rendered + '\n```\n\n'
    inventory = {'schema_version': 1, 'cargo_lock_sha256': sha((ROOT / 'Cargo.lock').read_bytes()),
                 'swift_package_resolved_sha256': swift_lock_sha256,
                 'packages': packages, 'x11_notice_sha256': sha(x11)}
    return body.rstrip() + '\n', json.dumps(inventory, indent=2, ensure_ascii=False) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='fail if committed notices are stale')
    args = parser.parse_args()
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--offline', '--format-version', '1'], cwd=ROOT))
    body, inventory = generate(metadata)
    for path, expected in [(ROOT / 'THIRD_PARTY_NOTICES.md', body),
                           (HERE / 'dependency-licenses.json', inventory)]:
        if args.check:
            if not path.exists() or path.read_text() != expected:
                raise ValueError(path.name + ' is stale; regenerate notices')
        else:
            path.write_text(expected)
    print('notices ' + ('verified' if args.check else 'generated'))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print('notice generation failed: ' + str(error), file=sys.stderr)
        sys.exit(1)
