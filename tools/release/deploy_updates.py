#!/usr/bin/env python3
"""Create Openergy's isolated Cloudflare R2 feed, or verify its public HTTPS path.

Bootstrap never replaces an existing feed. Publishing a signed application is
a separate release action; this command only creates an empty update channel.
"""
from __future__ import annotations

import argparse
import hashlib
import http.client
import io
import ipaddress
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / "deployment/cloudflare/updates.json"
API = "https://api.cloudflare.com/client/v4"


def cloudflare(method, path, body=None):
    token = os.environ.get("CLOUDFLARE_API_TOKEN")
    if not token:
        raise ValueError("CLOUDFLARE_API_TOKEN is required for bootstrap")
    request = urllib.request.Request(
        API + path, method=method,
        data=None if body is None else json.dumps(body).encode(),
        headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            result = json.load(response)
    except urllib.error.HTTPError as error:
        raise ValueError(f"Cloudflare {method} request failed with HTTP {error.code}") from None
    if result.get("success") is not True:
        raise ValueError("Cloudflare rejected the requested operation")
    return result["result"]


def configuration():
    config = json.loads(CONFIG.read_text())
    for key in ("zone", "domain"):
        if not re.fullmatch(r"[a-z0-9.-]+", config[key]):
            raise ValueError("invalid configured hostname")
    if not config["domain"].endswith("." + config["zone"]):
        raise ValueError("update hostname must be inside its configured zone")
    if not re.fullmatch(r"openergy-[a-z0-9-]+", config["bucket"]):
        raise ValueError("bootstrap only creates a dedicated Openergy bucket")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", config["wrangler_version"]):
        raise ValueError("Wrangler must have an exact pinned version")
    return config


def fetch(url, resolve_ip=None):
    if resolve_ip is None:
        with urllib.request.urlopen(urllib.request.Request(url, headers={"Cache-Control": "no-cache"}), timeout=30) as response:
            data = response.read(1024 * 1024 + 1)
            if len(data) > 1024 * 1024:
                raise ValueError("verification response exceeds its limit")
            return response.status, response.url, response.headers, data
    host = urllib.parse.urlsplit(url).hostname
    address = ipaddress.ip_address(resolve_ip)
    mapped = "[" + str(address) + "]" if address.version == 6 else str(address)
    with tempfile.TemporaryDirectory(prefix="openergy-https-check-") as name:
        header, body = Path(name) / "headers", Path(name) / "body"
        command = ["curl", "--disable", "--silent", "--show-error", "--proto", "=https",
                   "--resolve", f"{host}:443:{mapped}", "--max-time", "30", "--max-filesize", "1048576",
                   "--header", "Cache-Control: no-cache", "--dump-header", str(header),
                   "--output", str(body), "--write-out", "%{http_code}", url]
        environment = {key: value for key, value in os.environ.items() if key in ("PATH", "HOME", "TMPDIR")}
        result = subprocess.run(command, env=environment, capture_output=True, text=True, check=True, timeout=35)
        stream = io.BytesIO(header.read_bytes())
        stream.readline()
        return int(result.stdout), url, http.client.parse_headers(stream), body.read_bytes()


def verify(config, resolve_ip=None):
    base = "https://" + config["domain"]
    if resolve_ip is not None:
        address = ipaddress.ip_address(resolve_ip)
        dns_url = "https://cloudflare-dns.com/dns-query?" + urllib.parse.urlencode({"name": config["domain"], "type": "A" if address.version == 4 else "AAAA"})
        request = urllib.request.Request(dns_url, headers={"Accept": "application/dns-json"})
        with urllib.request.urlopen(request, timeout=30) as response:
            dns = json.load(response)
        if dns.get("Status") != 0 or not any(answer.get("data") == str(address) for answer in dns.get("Answer", [])):
            raise ValueError("request-specific address must match the public DNS response")
    status, url, headers, data = fetch(base + "/appcast.xml", resolve_ip)
    if url != base + "/appcast.xml" or status != 200:
        raise ValueError("feed did not return a direct HTTPS 200 response")
    if headers.get_content_type() not in ("application/rss+xml", "application/xml", "text/xml"):
        raise ValueError("feed has an unexpected content type")
    cache_control = headers.get("Cache-Control", "")
    rss = ET.fromstring(data)
    channel = rss.find("channel")
    if rss.tag != "rss" or channel is None or channel.findtext("title") != "Openergy Updates":
        raise ValueError("public response is not the Openergy RSS feed")
    status, _, _, page = fetch(base + "/index.html", resolve_ip)
    if status != 200 or b"Openergy Updates" not in page:
        raise ValueError("update channel page is unavailable")
    try:
        status, _, _, _ = fetch(base + "/openergy-verification-missing-object", resolve_ip)
    except urllib.error.HTTPError as error:
        if error.code != 404:
            raise ValueError("missing update object did not return HTTP 404") from None
    else:
        if status != 404:
            raise ValueError("missing update object did not return HTTP 404")
    return {"feed_url": base + "/appcast.xml", "http_status": 200,
            "feed_sha256": hashlib.sha256(data).hexdigest(),
            "published_updates": len(channel.findall("item")),
            "cache_control": cache_control, "missing_object_status": 404,
            "request_specific_resolver_ip": resolve_ip,
            "public_dns_verified_separately": resolve_ip is not None,
            "tls_certificate_validation": "system trust; no verification bypass"}


def bootstrap(config):
    zones = cloudflare("GET", "/zones?name=" + urllib.parse.quote(config["zone"]))
    if len(zones) != 1 or zones[0]["status"] != "active":
        raise ValueError("one active configured Cloudflare zone is required")
    zone = zones[0]
    account = zone["account"]["id"]
    bucket_path = f"/accounts/{account}/r2/buckets"
    buckets = cloudflare("GET", bucket_path)["buckets"]
    if any(bucket["name"] == config["bucket"] for bucket in buckets):
        domains = cloudflare("GET", bucket_path + "/" + config["bucket"] + "/domains/custom")["domains"]
        if any(domain["domain"] == config["domain"] and domain["enabled"] for domain in domains):
            return {"created_bucket": False, "existing_feed_preserved": True, **verify(config)}
        raise ValueError("bucket already exists without the expected public domain; refusing to replace data")
    existing = cloudflare("GET", f"/zones/{zone['id']}/dns_records?name=" + config["domain"])
    if existing:
        raise ValueError("hostname already has a DNS record; bootstrap will not replace it")
    public = ROOT / "deployment/cloudflare/public"
    if ET.parse(public / "appcast.xml").getroot().findall("channel/item"):
        raise ValueError("bootstrap must not publish an application update")
    # All preparation and validation precede this first external mutation.
    cloudflare("POST", bucket_path, {"name": config["bucket"]})
    with tempfile.TemporaryDirectory(prefix="openergy-r2-bootstrap-") as temporary:
        work = Path(temporary)
        work.chmod(0o700)
        environment = dict(os.environ, CLOUDFLARE_ACCOUNT_ID=account,
                           WRANGLER_LOG_PATH=str(work / "wrangler.log"),
                           WRANGLER_SEND_METRICS="false", CI="1")
        for name, content_type in (("appcast.xml", "application/rss+xml"), ("index.html", "text/html; charset=utf-8")):
            command = ["pnpm", "dlx", "wrangler@" + config["wrangler_version"], "r2", "object", "put",
                       config["bucket"] + "/" + name, "--remote", "--file", str(public / name),
                       "--content-type", content_type, "--cache-control", "public, max-age=60, must-revalidate"]
            command = ["node", str(ROOT / "r2-storage-budget.mjs"), "exec",
                       config["bucket"], name, str(public / name), "--", *command]
            result = subprocess.run(command, cwd=work, env=environment, stdin=subprocess.DEVNULL,
                                    capture_output=True, text=True, timeout=180)
            if result.returncode != 0:
                raise ValueError(f"Wrangler failed to upload {name}; bucket retained for inspection")
    domain = cloudflare("POST", bucket_path + "/" + config["bucket"] + "/domains/custom",
                        {"domain": config["domain"], "zoneId": zone["id"], "enabled": True, "minTLS": "1.2"})
    last_error = None
    for _ in range(24):
        try:
            return {"created_bucket": True, "existing_feed_preserved": False,
                    "domain_enabled": domain["enabled"], "minimum_tls": domain.get("minTLS"),
                    "bucket": config["bucket"], **verify(config)}
        except (ValueError, OSError, ET.ParseError) as error:
            last_error = type(error).__name__
            time.sleep(5)
    raise ValueError("channel created; public DNS/TLS verification is still pending (" + str(last_error) + ")")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("bootstrap", "verify"))
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--resolve-ip", help="verify only: per-request address, checked against public DNS; never changes system DNS")
    args = parser.parse_args()
    if args.report.exists() or args.report.is_symlink():
        raise ValueError("select a fresh report path")
    config = configuration()
    if args.command == "bootstrap" and args.resolve_ip is not None:
        raise ValueError("--resolve-ip is only available for read-only verification")
    report = bootstrap(config) if args.command == "bootstrap" else verify(config, args.resolve_ip)
    report.update({"command": args.command, "application_release_published_by_this_command": False})
    with args.report.open("x") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    args.report.chmod(0o600)
    print(json.dumps(report))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, ET.ParseError, subprocess.SubprocessError) as error:
        raise SystemExit("updates deployment: " + str(error))
