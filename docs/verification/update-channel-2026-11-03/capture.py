#!/usr/bin/env python3
"""Read public HTTP endpoints and source metadata; never publishes or executes apps.

Run from any directory: python3 capture.py --output /fresh/private/results.json
Requires Python 3 and curl. Response bodies/cookies/addresses are not retained.
"""
import argparse
import base64
import concurrent.futures
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import subprocess
import tempfile
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
BASE = 'https://openergy-updates.sergeserbinenko.com'
API = 'https://api.github.com/repos/SergeSerb2/openergy/releases/tags/v0.1.0'
UA = 'Openergy/0.1.0 Sparkle/2.10.0'
ENV = {k: v for k, v in os.environ.items() if k in ('PATH', 'HOME', 'TMPDIR')}


def request(spec):
    name, url, agent = spec
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    with tempfile.TemporaryDirectory(prefix='oe-update-get-') as tmp:
        body, headers = Path(tmp) / 'body', Path(tmp) / 'headers'
        cmd = ['curl', '--disable', '--silent', '--show-error', '--proto', '=https',
               '--max-time', '30', '--max-filesize', '2097152', '--header',
               'Cache-Control: no-cache', '--dump-header', str(headers), '--output',
               str(body), '--write-out', '%{http_code}', url]
        if agent:
            cmd += ['--user-agent', agent]
        run = subprocess.run(cmd, env=ENV, capture_output=True, text=True)
        data = body.read_bytes() if body.exists() else b''
        safe_headers = {}
        for line in headers.read_text().splitlines() if headers.exists() else []:
            key, sep, value = line.partition(':')
            if sep and key.lower() in ('content-type', 'server', 'cache-control', 'cf-cache-status', 'x-mitmproxy-blocked-reason'):
                safe_headers[key.lower()] = value.strip()
        out = dict(name=name, url=url, started_utc=started, user_agent=agent or 'curl default',
                   curl_exit=run.returncode, http_status=int(run.stdout) if run.stdout.isdigit() else None,
                   headers=safe_headers, response_bytes=len(data), response_sha256=hashlib.sha256(data).hexdigest(),
                   release_count=None, tls='curl system trust, no bypass; no redirects followed',
                   stderr=run.stderr.strip())
        if out['http_status'] == 200 and url.endswith('appcast.xml'):
            try:
                rss = ET.fromstring(data)
                out['release_count'] = len(rss.findall('channel/item')) if rss.tag == 'rss' and rss.find('channel') is not None else None
            except ET.ParseError:
                out['parse_error'] = 'response is not XML'
        if out['http_status'] == 200 and url == API:
            release = json.loads(data)
            out['github_release'] = {k: release.get(k) for k in ('id', 'html_url', 'tag_name', 'name', 'draft', 'prerelease', 'published_at', 'target_commitish')}
            out['github_release']['assets'] = [{k: a.get(k) for k in ('name', 'size', 'digest', 'browser_download_url', 'updated_at')} for a in release.get('assets', [])]
            out['github_release']['body'] = release.get('body')
        return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('select a fresh output path')
    args.output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    specs = [('feed-ordinary', BASE + '/appcast.xml', None),
             ('feed-sparkle-style', BASE + '/appcast.xml', UA),
             ('channel-page', BASE + '/index.html', None),
             ('missing-object', BASE + '/openergy-verification-missing-object', None),
             ('github-alpha', API, 'Openergy-update-channel-audit')]
    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
        requests = list(pool.map(request, specs))
    files = ['apps/macos/Support/Info.plist', 'apps/macos/Sources/Openergy/App/AppUpdater.swift',
             'apps/macos/Sources/Openergy/App/UpdaterEndToEnd.swift', 'apps/macos/Package.swift',
             'deployment/cloudflare/updates.json', 'deployment/cloudflare/public/appcast.xml',
             'tools/release/sparkle.py', 'tools/release/deploy_updates.py',
             'tools/release/release.py', 'tools/release/trusted_release.py', '.github/workflows/draft-release.yml']
    plist = plistlib.loads((ROOT / files[0]).read_bytes())
    data = (ROOT / 'deployment/cloudflare/public/appcast.xml').read_bytes()
    config = subprocess.run(['python3', str(ROOT / 'tools/release/sparkle.py'), 'config'], env={**ENV, 'OPENERGY_REQUIRE_UPDATES': '1'}, capture_output=True, text=True)
    with tempfile.TemporaryDirectory(prefix='oe-feed-signature-') as tmp:
        folder = Path(tmp)
        (folder / 'public.der').write_bytes(bytes.fromhex('302a300506032b6570032100') + base64.b64decode(plist['SUPublicEDKey']))
        (folder / 'signature').write_bytes(base64.b64decode(re.search(rb'edSignature: (\S+)', data).group(1)))
        (folder / 'signed.xml').write_bytes(data[:int(re.search(rb'length: (\d+)', data).group(1))])
        sig = subprocess.run(['openssl', 'pkeyutl', '-verify', '-pubin', '-inkey', str(folder / 'public.der'),
                              '-keyform', 'DER', '-rawin', '-in', str(folder / 'signed.xml'),
                              '-sigfile', str(folder / 'signature')], capture_output=True, text=True)
    hosting = []
    for label, environment in [('inherited-environment', None), ('clean-environment', ENV)]:
        with tempfile.TemporaryDirectory(prefix='oe-hosting-verify-') as tmp:
            output = Path(tmp) / 'hosting.json'
            command = ['python3', str(ROOT / 'tools/release/deploy_updates.py'), 'verify', '--report', str(output)]
            check = subprocess.run(command, env=environment, capture_output=True, text=True)
            hosting.append(dict(environment=label, exit=check.returncode, stdout=check.stdout, stderr=check.stderr,
                                report=json.loads(output.read_text()) if output.exists() else None))
    urllib_comparison = []
    for agent in ('Python-urllib/3.12', 'curl/8.5.0', UA):
        started = datetime.datetime.now(datetime.timezone.utc).isoformat()
        req = urllib.request.Request(BASE + '/appcast.xml', headers={'User-Agent': agent, 'Cache-Control': 'no-cache'})
        try:
            response = urllib.request.urlopen(req, timeout=30)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            body = response.read(1048576)
            urllib_comparison.append(dict(started_utc=started, user_agent=agent, http_status=response.status,
                                          response_bytes=len(body), sha256=hashlib.sha256(body).hexdigest(),
                                          headers={k.lower(): v for k, v in response.headers.items() if k.lower() in ('content-type', 'server', 'cf-cache-status')}))
    report = dict(source_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  environment=dict(os=platform.system(), kernel=platform.release(), architecture=platform.machine(),
                                   python=platform.python_version(), curl=subprocess.check_output(['curl', '--version'], text=True).splitlines()[0]),
                  source_sha256={p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in files},
                  source_configuration={k: plist[k] for k in ('SUFeedURL', 'SUPublicEDKey', 'CFBundleShortVersionString', 'CFBundleVersion')},
                  config_command=dict(argv=['OPENERGY_REQUIRE_UPDATES=1', 'python3', 'tools/release/sparkle.py', 'config'], exit=config.returncode, stdout=config.stdout, stderr=config.stderr),
                  source_feed=dict(length=len(data), sha256=hashlib.sha256(data).hexdigest(),
                                   release_count=len(ET.fromstring(data).findall('channel/item')),
                                   signature=re.search(rb'edSignature: (\S+)', data).group(1).decode(),
                                   signed_length=int(re.search(rb'length: (\d+)', data).group(1))),
                  signature_verification=dict(tool=subprocess.check_output(['openssl', 'version'], text=True).strip(),
                                              method='Ed25519 pkeyutl -verify -pubin -keyform DER -rawin over first signed_length bytes; bundle public key',
                                              exit=sig.returncode, stdout=sig.stdout, stderr=sig.stderr),
                  hosting_verifier=hosting,
                  urllib_comparison=urllib_comparison,
                  inherited_proxy_variable_names=[k for k in os.environ if k.lower() in ('http_proxy', 'https_proxy', 'all_proxy', 'no_proxy')],
                  requests=requests,
                  limits=['No signed-app execution in this capture; user-agent emulation only.',
                          'No authenticated Cloudflare rules inspected; no credentials or deployment changes.',
                          'HTTP results describe this host/network path only; inaccessible feeds have unknown release counts.',
                          'GitHub API metadata is separate from Sparkle publication; no release binary downloaded or executed.'])
    with args.output.open('x') as f:
        json.dump(report, f, indent=2)
        f.write('\n')
    args.output.chmod(0o600)
    print(json.dumps({'output': args.output.name, 'requests': [{k: r[k] for k in ('name', 'http_status', 'curl_exit', 'release_count', 'headers')} for r in requests]}))


if __name__ == '__main__':
    main()
