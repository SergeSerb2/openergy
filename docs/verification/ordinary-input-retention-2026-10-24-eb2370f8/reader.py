"""Own static reader. Supplied files are bytes/JSON/AST only; no product calls.

The private discovery journal supplies the finite, provenance-derived path plan.
No private absolute path, raw command, environment, endpoint, or log is emitted.
Run only for this file-retention audit; this is not an application verifier.
"""
import ast
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys

LIMIT = 512 * 1024 * 1024
RESERVE = 8 * 1024 * 1024  # Conservative allowance for preliminary static reads.
MAX_ENTRIES = 2000
OUT = Path(__file__).absolute().parent
ARCHIVE = "docs/verification/ordinary-app-e2e-2026-10-23-t3-79498fe8"
R7 = "docs/verification/ordinary-controller-repair-r7-2026-10-23-t3-ad9d06e1"
ORIGINAL = "498190feb00f51039f500ae618ac7de5fb113d86"


def sha(data):
    return hashlib.sha256(data).hexdigest()


class Reader:
    def __init__(self, journal):
        self.used = 0
        self.events = []
        self.journal = Path(journal).absolute()
        self.private = self.journal.parent
        self.state = json.loads(self.read(self.journal))
        self.repo = Path(self.state["repo"])
        self.plan = self.state["plan"]
        assert len(self.plan) == len({x["path"] for x in self.plan}) <= 64
        self.locations = {x["label"]: Path(x["path"]) for x in self.plan}
        assert self.private.lstat().st_mode & 0o777 == 0o700
        assert OUT.parent == self.repo / "docs/verification"

    def charge(self, count):
        assert self.used + count + RESERVE <= LIMIT, "read budget exhausted"
        self.used += count

    def checked(self, path):
        # The sole system alias in the icon provenance is /var -> /private/var.
        path = Path(path).absolute()
        if str(path).startswith("/var/"):
            path = Path("/private") / path.relative_to("/")
        assert ".." not in path.parts, "unexpected parent traversal"
        assert not any(x in path.parts for x in [".ssh", ".aws", ".gnupg", "swift-security"])
        for parent in reversed(path.parents):
            try:
                info = parent.lstat()
            except FileNotFoundError:
                return path
            assert stat.S_ISDIR(info.st_mode), "unexpected ancestor or symlink; stop"
        return path

    def read(self, path):
        path = self.checked(path)
        before = path.lstat()
        assert stat.S_ISREG(before.st_mode), "unexpected nonregular input; stop"
        self.charge(before.st_size)
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
        with os.fdopen(fd, "rb") as stream:
            opened = os.fstat(stream.fileno())
            assert (opened.st_dev, opened.st_ino) == (before.st_dev, before.st_ino)
            data = stream.read(before.st_size + 1)
            after = os.fstat(stream.fileno())
        assert len(data) == before.st_size
        assert (before.st_size, before.st_mtime_ns, before.st_ctime_ns) == (
            after.st_size, after.st_mtime_ns, after.st_ctime_ns)
        return data

    def git(self, *args, input_bytes=None):
        # The only child calls are explicit, read-only Git object readers.
        assert args[0] in ["ls-tree", "cat-file", "rev-parse"]
        result = subprocess.run(["git", "--no-optional-locks", *args], cwd=self.repo,
                                input=input_bytes, capture_output=True, check=True)
        self.charge(len(result.stdout) + len(result.stderr))
        return result.stdout

    def tree(self, root):
        root = self.checked(root)
        if not root.exists():
            return None
        assert stat.S_ISDIR(root.lstat().st_mode), "unexpected artifact root; stop"
        rows = []

        def walk(directory):
            with os.scandir(directory) as entries:
                children = sorted(entries, key=lambda x: x.name)
            assert len(children) <= MAX_ENTRIES
            for child in children:
                assert len(rows) < MAX_ENTRIES, "artifact entry bound; stop"
                path = Path(child.path)
                info = path.lstat()
                row = {"path": str(path.relative_to(root)), "mode": oct(stat.S_IMODE(info.st_mode))}
                rows.append(row)
                if stat.S_ISLNK(info.st_mode):
                    target = os.readlink(path)
                    assert not os.path.isabs(target), "absolute symlink; stop"
                    lexical = Path(os.path.normpath(path.parent / target))
                    assert lexical.is_relative_to(root), "symlink escape; stop"
                    assert path.resolve().is_relative_to(root.resolve()), "symlink escape; stop"
                    row["link"] = target
                elif stat.S_ISREG(info.st_mode):
                    data = self.read(path)
                    row.update(bytes=len(data), sha256=sha(data))
                elif stat.S_ISDIR(info.st_mode):
                    row["directory"] = True
                    walk(path)
                else:
                    raise AssertionError("unexpected artifact file type; stop")

        walk(root)
        return sorted(rows, key=lambda x: x["path"])

    def protected(self, relative, commit):
        listing = self.git("ls-tree", "-rz", commit, "--", relative)
        expected = {}
        for item in listing.split(b"\0"):
            if item:
                prefix, name = item.split(b"\t", 1)
                mode, kind, blob = prefix.decode().split()
                assert kind == "blob" and mode in ["100644", "100755"]
                expected[name.decode()] = (mode, blob)
        assert expected
        request = b"".join(x[1].encode() + b"\n" for x in expected.values())
        objects = self.git("cat-file", "--batch", input_bytes=request)
        offset = 0
        rows = []
        for name, (mode, blob) in expected.items():
            end = objects.index(b"\n", offset)
            header = objects[offset:end].decode().split()
            assert header[:2] == [blob, "blob"]
            size = int(header[2])
            data = objects[end + 1:end + 1 + size]
            offset = end + 2 + size
            disk = self.read(self.repo / name)
            assert data == disk, "protected bytes differ"
            rows.append(dict(path=name, bytes=size, sha256=sha(data), git_blob=blob))
        assert offset == len(objects)
        self.events.append({"protected":relative, "rows":rows})
        return rows

    def save(self, path, value):
        data = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
        return sha(data)

    def copy(self, path, relative, expected):
        data = self.read(path)
        assert sha(data) == expected, "copy source hash mismatch"
        dest = self.private / relative
        dest.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        assert dest.is_relative_to(self.private)
        fd = os.open(dest, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
        copied = self.read(dest)
        assert data == copied
        return dict(retained_name=relative, bytes=len(data), before_sha256=expected,
                    copied_sha256=sha(copied), stored_mode="0o600",
                    original_mode=oct(stat.S_IMODE(path.lstat().st_mode)),
                    executable_mode_retained_as_metadata_only=True)

    def audit(self):
        b = self.repo / ARCHIVE
        original = self.protected(ARCHIVE, ORIGINAL)
        assert len(original) == 87
        frozen = self.protected(R7, "d56c65eb8da5f3d8b4fd58fc0e933360754ba6d6")
        review_rel = str(self.locations["R8-full-review"].relative_to(self.repo))
        review = self.protected(review_rel, "172ee85")
        data = {p.name: json.loads(self.read(p)) for p in [
            b / "raw/prepared.json", b / "raw/bundle-signed-tree.json",
            b / "raw/framework-artifact-bindings.json", b / "raw/framework-resolution.json",
            b / "raw/app-input-bindings.json", b / "raw/daemon-397-input-bindings.json",
            b / "raw/inherited-build-records.json", b / "raw/icon-copy.json",
            b / "raw/packaging-tool-binding.json"]}
        for package in [b, self.repo / R7]:
            for path in package.glob("*.py"):
                ast.parse(self.read(path), filename=path.name)
        signed_expected = data["bundle-signed-tree.json"]
        assert len(signed_expected) == 167
        framework = data["framework-artifact-bindings.json"]
        assert framework["expected"] == framework["actual"] and framework["match"]
        assert len(framework["expected"]) == 124
        observations = []
        for item in self.plan:
            label, kind = item["label"], item["kind"]
            path = self.checked(item["path"])
            row = {"location_id":label,"kind":kind,
                   "path_binding_sha256":sha(item["path"].encode())}
            try:
                info = path.lstat()
            except FileNotFoundError:
                row["status"] = "ABSENT"
            else:
                row["status"] = "PRESENT"
                assert not stat.S_ISLNK(info.st_mode), "unexpected location symlink; stop"
                if kind in ["signed-tree", "signed-candidate"]:
                    inventory = self.tree(path)
                    row.update(entries=len(inventory), complete_signed_inventory_match=inventory == signed_expected)
                    self.events.append({"location_id":label,"inventory":inventory})
                elif kind in ["artifact-tree", "artifact-subtree"]:
                    inventory = self.tree(path)
                    # Full cache tree only: subtree presence alone cannot qualify all 124 inputs.
                    actual = [{"path":"artifacts/" + x["path"], **(
                        {"symlink":x["link"]} if "link" in x else
                        {"bytes":x["bytes"], "sha256":x["sha256"]})}
                        for x in inventory if not x.get("directory")]
                    row.update(entries=len(inventory), complete_artifact_match=(
                        kind == "artifact-tree" and actual == framework["expected"]))
                    self.events.append({"location_id":label,"inventory":inventory})
                elif "expected_sha256" in item:
                    content = self.read(path)
                    row.update(bytes=len(content),sha256=sha(content),
                               expected_sha256=item["expected_sha256"],
                               match=sha(content) == item["expected_sha256"])
            observations.append(row)

        receipts = []
        copies = []
        for item in data["inherited-build-records.json"]:
            archived = b / "raw" / item["copy"]
            content = self.read(archived)
            assert sha(content) == item["sha256"]
            source = self.checked(item["source"])
            row = {"name":item["copy"],"archive_match":True}
            try:
                original_source = self.read(source)
            except FileNotFoundError:
                row["original_source_status"] = "ABSENT"
            else:
                row.update(original_source_status="PRESENT",original_source_match=sha(original_source) == item["sha256"])
            receipts.append(row)
            copies.append(self.copy(archived,"receipts/" + item["copy"],item["sha256"]))

        binding_rows = []
        for root_label in ["current-exact-source-list", "ordinary-source-root"]:
            if root_label not in self.locations:
                continue
            root = self.locations[root_label]
            for item in data["app-input-bindings.json"]:
                expected = item["actual_sha256"]
                name = item["path"]
                assert not Path(name).is_absolute() and ".." not in Path(name).parts
                try:
                    content = self.read(root / name)
                    row = dict(root=root_label,group="app",path=name,sha256=sha(content),
                               ordinary_source_match=sha(content) == expected,
                               compiled_source_match=sha(content) == item["sha256"])
                except FileNotFoundError:
                    row = dict(root=root_label,group="app",path=name,status="ABSENT")
                binding_rows.append(row)
            for name, item in data["daemon-397-input-bindings.json"].items():
                assert not Path(name).is_absolute() and ".." not in Path(name).parts
                try:
                    content = self.read(root / name)
                    row = dict(root=root_label,group="daemon",path=name,sha256=sha(content),
                               ordinary_source_match=sha(content) == item["sha256"])
                except FileNotFoundError:
                    row = dict(root=root_label,group="daemon",path=name,status="ABSENT")
                binding_rows.append(row)

        source_records = []
        app_path = self.locations["original-app-record-directory"] / "build-source.json"
        app_data = self.read(app_path)
        app_records = json.loads(app_data)
        app_semantic = all({k:r[k] for k in ["path","bytes","sha256"]} ==
                           {k:e[k] for k in ["path","bytes","sha256"]}
                           for r,e in zip(app_records,data["app-input-bindings.json"]))
        assert len(app_records) == 29 and app_semantic
        daemon_path = self.locations["original-daemon-record-directory"] / "source-bindings.json"
        daemon_data = self.read(daemon_path)
        daemon_records = json.loads(daemon_data)
        expected_daemon = data["daemon-397-input-bindings.json"]
        assert set(daemon_records) == set(expected_daemon)
        assert all(daemon_records[n]["sha256"] == e["expected_sha256"] for n,e in expected_daemon.items())
        for name,path,content,count in [("app-build-source.json",app_path,app_data,29),
                                         ("daemon-source-bindings.json",daemon_path,daemon_data,397)]:
            source_records.append(dict(name=name,sha256=sha(content),records=count,
                                       binding_check="semantic equality to immutable original input records"))
            copies.append(self.copy(path,"source-bindings/" + name,sha(content)))
        # Preserve immutable binding evidence as data, without transforming its bytes.
        for name in ["prepared.json","bundle-signed-tree.json","framework-artifact-bindings.json",
                     "framework-resolution.json","app-input-bindings.json","daemon-397-input-bindings.json",
                     "icon-copy.json","packaging-tool-binding.json","inherited-build-records.json",
                     "app-source-architecture.json","daemon-source-architecture.json","ordinary-Info.plist"]:
            content = self.read(b / "raw" / name)
            copies.append(self.copy(b / "raw" / name,"bindings/" + name,sha(content)))
        daemon_observation = next(x for x in observations if x["location_id"] == "compiled-daemon")
        if daemon_observation.get("match"):
            copies.append(self.copy(self.locations["compiled-daemon"],"inputs/daemon.bin.data",daemon_observation["sha256"]))

        # Repeat protected byte checks after all source reads and new private writes.
        assert original == self.protected(ARCHIVE, ORIGINAL)
        assert frozen == self.protected(R7, "d56c65eb8da5f3d8b4fd58fc0e933360754ba6d6")
        assert review == self.protected(review_rel, "172ee85")
        private_inventory = self.tree(self.private)
        journal = dict(plan=self.plan,plan_evidence=self.state["plan_evidence"],
                       events=self.events,observations=observations,source_bindings=binding_rows,
                       copy_rows=copies,private_inventory_before_receipt=private_inventory)
        journal_sha = self.save(self.private / "audit-journal.json",journal)
        receipt = dict(kind="FILE INPUT RETENTION ONLY; no reassembly/signing/execution/acceptance",
                       copy_rows=copies,journal_sha256=journal_sha,
                       source_locations=len(self.plan),read_content_bytes=self.used,
                       preliminary_read_allowance_bytes=RESERVE)
        receipt_sha = self.save(self.private / "copy-receipt.json",receipt)
        summary = dict(kind="STATIC FILE RETENTION ONLY",original_files_unchanged=len(original),
                       R7_files_unchanged=len(frozen),R8_files_unchanged=len(review),
                       original_commit=ORIGINAL,locations=observations,receipts=receipts,
                       source_record_data=source_records,source_binding_results=binding_rows,
                       copies=copies,private_copy_receipt_sha256=receipt_sha,
                       private_journal_sha256=journal_sha,source_locations=len(self.plan),
                       max_artifact_entries=MAX_ENTRIES,content_bytes_read=self.used,
                       preliminary_read_allowance_bytes=RESERVE,
                       conservative_read_total_bytes=self.used+RESERVE,read_limit_bytes=LIMIT,
                       framework_records=124,signed_tree_records=167)
        self.save(OUT / "results.json",summary)
        self.save(OUT / "protected-original-index.json",original)
        print(json.dumps({k:summary[k] for k in ["original_files_unchanged","R7_files_unchanged",
            "R8_files_unchanged","source_locations","conservative_read_total_bytes",
            "private_copy_receipt_sha256"]},sort_keys=True))


if __name__ == "__main__":
    assert len(sys.argv) == 2, "private discovery journal argument required"
    Reader(sys.argv[1]).audit()
