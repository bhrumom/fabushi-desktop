"""Actions-only historical BASE producer -> current Host/App recovery evidence."""

import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tarfile
import traceback


BASE = "e19a0a265e400a4c9b1d96520c84fb612a3843e8"
BASE_TREE = "5be2d7ca128dae5fefc8f7551e7088934daf19ac"
PARENT = "source/host/tests/production_checkpoint_restart_contract.rs"
SUPPORT = "source/host/tests/support/prior_root_historical_fixture.rs"
DECLARATION = '\n#[path = "support/prior_root_historical_fixture.rs"]\nmod prior_root_historical_fixture;\n'
TARGET = "production_checkpoint_restart_contract"
PREFIX = "prior_root_historical_fixture::"
ROOT = pathlib.Path(".evidence/sqlite-native-path/prior-root-identity").resolve()
CHECKOUT = pathlib.Path.cwd().resolve()
ROOT.mkdir(parents=True, exist_ok=False)
RECEIPT = {
    "contract": "OCRM-CANONICAL-PRIOR-ROOT-IDENTITY-v1",
    "base_commit": BASE, "base_tree": BASE_TREE,
    "result": "failed-or-incomplete", "commands": [], "utilities": [],
    "run_id": os.environ["GITHUB_RUN_ID"],
    "run_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
    "job": os.environ["GITHUB_JOB"], "runner_os": os.environ["RUNNER_OS"],
    "scope": "Exact historical BASE production with a test-only overlay creates two real closed completed histories. Current Host readers and the existing bound shipping App acknowledge/no-op without provider credentials or replay. Utility successes are two evidence operations, not extra product test counts. Controlled HTTP/effects do not establish real provider, interrupted-turn replay, cross-platform package or global acceptance.",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_digest(path):
    with pathlib.Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    pathlib.Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def save():
    write_json(ROOT / "receipt.json", RECEIPT)


def git(checkout, *args):
    return subprocess.check_output(["git", "-C", str(checkout), *args])


def run(command, stem, cwd=CHECKOUT, env=None, cargo_json=False):
    stdout_path, stderr_path = ROOT / (stem + ".stdout.log"), ROOT / (stem + ".stderr.log")
    entry = {"argv": list(map(str, command)), "cwd": str(cwd), "stem": stem,
             "exit": None, "capture_exit": None, "environment_additions": env or {}}
    RECEIPT["commands"].append(entry)
    RECEIPT["active_command"] = stem
    save()
    try:
        with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
            completed = subprocess.run(entry["argv"], cwd=cwd, env=os.environ | (env or {}),
                                       stdout=stdout, stderr=stderr, check=False)
        entry["exit"], entry["capture_exit"] = completed.returncode, 0
    except OSError:
        entry["capture_exit"] = 1
        raise
    finally:
        for label, path in (("stdout", stdout_path), ("stderr", stderr_path)):
            if path.is_file():
                entry[label] = path.name
                entry[label + "_sha256"] = file_digest(path)
        save()
    out, err = stdout_path.read_text(errors="replace"), stderr_path.read_text(errors="replace")
    print(json.dumps({"command": entry["argv"], "exit": entry["exit"]}), flush=True)
    if cargo_json:
        if entry["exit"]:
            for line in out.splitlines():
                event = json.loads(line)
                if event.get("reason") == "compiler-message":
                    print(event.get("message", {}).get("rendered", line), flush=True)
    else:
        print(out, flush=True)
    print(err, flush=True)
    if entry["exit"]:
        raise RuntimeError("command failed: " + stem)
    RECEIPT.pop("active_command", None)
    save()
    return out


def source_snapshot(checkout, subject, overlays, stem):
    if git(checkout, "rev-parse", "HEAD").decode().strip() != subject:
        raise RuntimeError("source checkout HEAD changed: " + stem)
    # Enumerate every tracked blob. This verifies the historical build source,
    # rather than relying only on a hand-selected list of checkpoint files.
    entries = git(checkout, "ls-tree", "-r", "-z", "HEAD").split(b"\0")
    inputs = {}
    for raw in entries:
        if not raw:
            continue
        header, name = raw.split(b"\t", 1)
        mode, kind, blob = header.decode().split()
        path = name.decode()
        if kind != "blob":
            raise RuntimeError("unresolved tracked non-blob build source: " + path)
        local = checkout / path
        if mode == "120000":
            if not local.is_symlink():
                raise RuntimeError("tracked symlink mode changed: " + path)
            data = os.fsencode(os.readlink(local))
        else:
            if local.is_symlink() or not local.is_file():
                raise RuntimeError("tracked regular file unavailable: " + path)
            data = local.read_bytes()
            if bool(local.stat().st_mode & 0o111) != (mode == "100755"):
                raise RuntimeError("tracked executable mode changed: " + path)
        actual_blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        if path in overlays:
            if data != overlays[path]:
                raise RuntimeError("test overlay differs from its captured bytes: " + path)
        elif actual_blob != blob:
            raise RuntimeError("tracked source differs from exact subject: " + path)
        inputs[path] = {"mode": mode, "subject_blob": blob, "actual_blob": actual_blob,
                        "sha256": digest(data), "bytes": len(data), "test_overlay": path in overlays}
    for path, data in overlays.items():
        if path not in inputs:
            if (checkout / path).read_bytes() != data:
                raise RuntimeError("new test overlay differs: " + path)
            inputs[path] = {"mode": "100644", "subject_blob": None,
                            "actual_blob": hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest(),
                            "sha256": digest(data), "bytes": len(data), "test_overlay": True}
    if overlays:
        changed = set(git(checkout, "diff", "--name-only", "HEAD").decode().splitlines())
        untracked = set(git(checkout, "ls-files", "--others", "--exclude-standard").decode().splitlines())
        if changed != {PARENT} or untracked != {SUPPORT}:
            raise RuntimeError("historical checkout has an undeclared modification")
    artifact = ROOT / (stem + ".json")
    write_json(artifact, inputs)
    RECEIPT[stem] = {"path": artifact.name, "sha256": file_digest(artifact), "file_count": len(inputs)}
    save()
    return inputs


def build_host(checkout, stem, binary_root):
    output = run(["cargo", "test", "--locked", "--manifest-path", str(checkout / "source/host/Cargo.toml"),
                  "--test", TARGET, "--no-run", "--message-format=json"], stem, cwd=checkout, cargo_json=True)
    executables = set()
    for line in output.splitlines():
        event = json.loads(line)
        target = event.get("target", {})
        if (event.get("reason") == "compiler-artifact" and target.get("name") == TARGET
                and target.get("kind") == ["test"] and event.get("profile", {}).get("test") is True
                and pathlib.Path(target["src_path"]).resolve() == (checkout / PARENT).resolve()
                and event.get("executable")):
            executables.add(pathlib.Path(event["executable"]).resolve())
    if len(executables) != 1:
        raise RuntimeError("Host test compiler artifact absent or ambiguous")
    original = executables.pop()
    target_root = pathlib.Path(os.environ["CARGO_TARGET_DIR"]).resolve()
    if not original.is_file() or not original.is_relative_to(target_root):
        raise RuntimeError("Host test executable outside the declared shared target")
    # Keep the H executable before rebuilding BASE in the existing cache.
    retained = binary_root / stem
    shutil.copy2(original, retained)
    sha = file_digest(original)
    if file_digest(retained) != sha:
        raise RuntimeError("retained compiler artifact changed during copy")
    RECEIPT[stem + "_executable"] = {"compiler_path": str(original), "retained_path": str(retained), "sha256": sha}
    save()
    return retained, sha


def exact_utility(executable, name, stem, env):
    full_name = PREFIX + name
    listing = run([executable, full_name, "--exact", "--ignored", "--list"], stem + "-list", env=env)
    if re.findall(r"^(.+): test$", listing, re.MULTILINE) != [full_name]:
        raise RuntimeError("ignored utility selection absent or ambiguous")
    output = run([executable, full_name, "--exact", "--ignored", "--test-threads=1"], stem, env=env)
    names = re.findall(r"^test (\S+) \.\.\. ok$", output, re.MULTILINE)
    summaries = re.findall(r"^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;", output, re.MULTILINE)
    if names != [full_name] or len(summaries) != 1 or summaries[0][:5] != ("ok", "1", "0", "0", "0"):
        raise RuntimeError("utility did not produce its exact single successful completion")
    RECEIPT["utilities"].append({"name": full_name, "names": names, "summaries": summaries, "result": "passed"})
    save()


try:
    if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ["RUNNER_OS"] != "Linux":
        raise RuntimeError("this instrument requires its declared Linux GitHub Actions job")
    head = git(CHECKOUT, "rev-parse", "HEAD").decode().strip()
    tree = git(CHECKOUT, "rev-parse", "HEAD^{tree}").decode().strip()
    if head != os.environ["FABUSHI_TARGET_SHA"] or head == BASE:
        raise RuntimeError("current consumer exact subject mismatch")
    lock = json.loads((CHECKOUT / "projects/openclaw-rust/upstream.lock.json").read_text())
    RECEIPT.update({"subject_commit": head, "subject_tree": tree,
                    "upstream": {key: lock["upstream"][key] for key in ("repository", "commit", "tree")},
                    "rustc": subprocess.check_output(["rustc", "--version", "--verbose"], text=True),
                    "environment": {key: os.environ[key] for key in ("TMPDIR", "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS")}})
    head_inputs = source_snapshot(CHECKOUT, head, {}, "head-inputs-before")
    app_receipt_path = CHECKOUT / ".evidence/sqlite-native-path/automation-app/receipt.json"
    app_receipt = json.loads(app_receipt_path.read_text())
    if (app_receipt.get("result") != "passed" or app_receipt.get("subject_commit") != head
            or app_receipt.get("subject_tree") != tree):
        raise RuntimeError("current shipping App evidence is missing or belongs to another H")
    app = pathlib.Path(app_receipt["executable"]["path"]).resolve()
    app_sha = app_receipt["executable"]["sha256"]
    if not app.is_relative_to(pathlib.Path(os.environ["CARGO_TARGET_DIR"]).resolve()) or file_digest(app) != app_sha:
        raise RuntimeError("bound shipping App executable changed")
    owned = pathlib.Path(os.environ["RUNNER_TEMP"]) / ("prid-" + os.environ["GITHUB_RUN_ID"] + "-" + os.environ["GITHUB_RUN_ATTEMPT"])
    owned.mkdir(exist_ok=False)
    binary_root = owned / "binaries"
    binary_root.mkdir()
    retained_app = binary_root / "head-app"
    shutil.copy2(app, retained_app)
    if file_digest(retained_app) != app_sha:
        raise RuntimeError("retained shipping App executable differs")
    RECEIPT["app"] = {"receipt_sha256": file_digest(app_receipt_path), "compiler_path": str(app),
                      "retained_path": str(retained_app), "sha256": app_sha}
    current_binary, current_sha = build_host(CHECKOUT, "head-host", binary_root)

    run(["git", "fetch", "--no-tags", "--depth=1", "origin", BASE], "base-fetch")
    if git(CHECKOUT, "rev-parse", BASE + "^{tree}").decode().strip() != BASE_TREE:
        raise RuntimeError("historical production tree differs")
    base_root = owned / "base-source"
    run(["git", "worktree", "add", "--detach", str(base_root), BASE], "base-checkout")
    parent_before = (base_root / PARENT).read_bytes()
    if b"mod prior_root_historical_fixture;" in parent_before or (base_root / SUPPORT).exists():
        raise RuntimeError("historical test-only overlay is not an addition")
    support = (CHECKOUT / SUPPORT).read_bytes()
    overlay = {PARENT: parent_before + DECLARATION.encode(), SUPPORT: support}
    for path, data in overlay.items():
        destination = base_root / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        evidence = ROOT / "overlay" / path
        evidence.parent.mkdir(parents=True, exist_ok=True)
        evidence.write_bytes(data)
    base_inputs = source_snapshot(base_root, BASE, overlay, "base-inputs-before")
    historical_binary, historical_sha = build_host(base_root, "base-host", binary_root)
    capture_id = ":".join((os.environ["GITHUB_RUN_ID"], os.environ["GITHUB_RUN_ATTEMPT"], os.environ["GITHUB_JOB"]))
    provenance = {"format": "prid-v1-capture", "base_commit": BASE, "base_tree": BASE_TREE,
                  "head_commit": head, "head_tree": tree, "support_module_sha256": digest(support), "capture_id": capture_id}
    export = ROOT / "historical-fixtures"
    export.mkdir()
    base_provenance = ROOT / "base-provenance.json"
    write_json(base_provenance, provenance | {"phase": "base-export", "parent_harness_sha256": digest(overlay[PARENT]),
                                           "host_binary_sha256": historical_sha, "app_binary_sha256": None})
    exact_utility(historical_binary, "export_base_writer_fixtures", "historical-export", {
        "FABUSHI_PRIOR_ROOT_FIXTURE_EXPORT_DIR": str(export),
        "FABUSHI_PRIOR_ROOT_FIXTURE_PROVENANCE_FILE": str(base_provenance),
    })
    inventory_sha = file_digest(export / "inventory.json")
    archive = ROOT / "historical-fixtures.tar.gz"
    with tarfile.open(archive, "x:gz") as tar:
        tar.add(export, arcname="historical-fixtures", recursive=True)
    archive_sha = file_digest(archive)
    RECEIPT["historical_payload"] = {"directory": export.name, "inventory_sha256": inventory_sha,
                                     "archive": archive.name, "archive_sha256": archive_sha, "archive_bytes": archive.stat().st_size}
    save()
    consume = ROOT / "current-consumer"
    consume.mkdir()
    head_provenance = ROOT / "head-provenance.json"
    write_json(head_provenance, provenance | {"phase": "head-consume", "parent_harness_sha256": file_digest(CHECKOUT / PARENT),
                                           "host_binary_sha256": current_sha, "app_binary_sha256": app_sha})
    exact_utility(current_binary, "consume_base_writer_fixtures", "current-consume", {
        "FABUSHI_PRIOR_ROOT_FIXTURE_IMPORT_DIR": str(export),
        "FABUSHI_PRIOR_ROOT_FIXTURE_CONSUMER_DIR": str(consume),
        "FABUSHI_PRIOR_ROOT_FIXTURE_PROVENANCE_FILE": str(head_provenance),
        "FABUSHI_PRIOR_ROOT_FIXTURE_INVENTORY_SHA256": inventory_sha,
        "FABUSHI_ORIGINAL_TURN_APP_TEST_EXECUTABLE": str(retained_app),
        "FABUSHI_ORIGINAL_TURN_APP_TEST_SHA256": app_sha,
    })
    if file_digest(export / "inventory.json") != inventory_sha or file_digest(archive) != archive_sha:
        raise RuntimeError("original historical payload changed after consumption")
    if source_snapshot(base_root, BASE, overlay, "base-inputs-after") != base_inputs:
        raise RuntimeError("historical build source changed during capture")
    if source_snapshot(CHECKOUT, head, {}, "head-inputs-after") != head_inputs:
        raise RuntimeError("current build source changed during capture")
    for executable, expected in ((current_binary, current_sha), (historical_binary, historical_sha), (retained_app, app_sha)):
        if file_digest(executable) != expected:
            raise RuntimeError("executed retained binary changed")
    consumer_inventory_sha = file_digest(consume / "inventory.json")
    # The artifact uploader excludes dotfiles from directory payloads. Keep
    # this closed, inventoried subtree complete without widening that upload.
    consumer_archive = ROOT / "current-consumer.tar.gz"
    with tarfile.open(consumer_archive, "x:gz") as tar:
        tar.add(consume, arcname="current-consumer", recursive=True)
    if file_digest(consume / "inventory.json") != consumer_inventory_sha:
        raise RuntimeError("current consumer inventory changed during archive capture")
    RECEIPT["consumer_inventory_sha256"] = consumer_inventory_sha
    RECEIPT["consumer_payload"] = {
        "directory": consume.name, "inventory_sha256": consumer_inventory_sha,
        "archive": consumer_archive.name, "archive_sha256": file_digest(consumer_archive),
        "archive_bytes": consumer_archive.stat().st_size,
    }
    RECEIPT["result"] = "passed"
except Exception:
    RECEIPT["error"] = traceback.format_exc()
finally:
    save()
    print(json.dumps(RECEIPT, indent=2), flush=True)
sys.exit(0 if RECEIPT["result"] == "passed" else 1)
