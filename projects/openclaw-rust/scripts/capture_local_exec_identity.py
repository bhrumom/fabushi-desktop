#!/usr/bin/env python3
"""Capture the LPI-v1 Rust/TypeScript boundary on its exact Actions subject."""

import hashlib
import json
import os
import pathlib
import re
import select
import signal
import subprocess
import sys
import time


ROOT = pathlib.Path(".evidence/sqlite-native-path/local-exec-process-identity")
SPEC = "projects/openclaw-rust/docs/specs/local-exec-process-identity-wire.md"
SCRIPT = "projects/openclaw-rust/scripts/capture_local_exec_identity.py"
CRATE = "source/node-agent-coordinator"
ENTRY = "source/electron-main/coordinator/local-exec-process-identity.contract.test.ts"
OUT = pathlib.Path("desktop/.tmp-grok-electron/ocrm-local-exec-identity")
BUNDLE = OUT / "local-exec-process-identity.contract.test.cjs"
META = OUT / "build-metafile.json"
BINARY = pathlib.Path("source/host/target/debug/mahayana-node-agent-coordinator")
UPSTREAM = {
    "repository": "openclaw/openclaw",
    "commit": "1440bb1771372334bea23d7642bb52deb7db1845",
    "tree": "90c8aab39972da11cfad220361781d49e1bc6700",
}
RUST_NAMES = [
    "runtime_spawns_pauses_and_resumes_with_generation_fencing",
    "runtime_adopts_busy_generation_and_replaces_idle_generation",
    "runtime_quarantines_same_pid_generation_change_without_duplicate_spawn",
    "client_pause_refusal_prevents_spawn_until_connection_recovers",
    "identity_helpers_require_exact_generation_arguments_and_bounded_publication_time",
    "process_identity_wire_omits_only_absent_optional_constraints",
    "process_identity_wire_retains_present_zero_and_empty_constraints",
]
TAP_NAMES = [
    "LPI actual Coordinator adopts busy discovery and re-adopts after control restart",
    "LPI actual Coordinator preserves pre-discovery spawn and full liveness identity",
    "LPI actual native identity consumer rejects null types and mismatched identities",
    "LPI control EOF settles the owned Coordinator without daemon mutation",
]
DIRECT_INPUTS = [
    ".github/workflows/rust-desktop-runtime.yml", SPEC, SCRIPT,
    "projects/openclaw-rust/upstream.lock.json",
    CRATE + "/Cargo.toml", CRATE + "/Cargo.lock",
    CRATE + "/tests/local_exec_supervisor_runtime.rs",
    ENTRY, "desktop/package.json", "desktop/package-lock.json",
    "desktop/scripts/build-electron-runtime.mjs",
    "source/electron-main/coordinator/coordinator-control-server.ts",
    "source/electron-main/coordinator/coordinator-executors.ts",
    "source/shared/rpc/coordinator-port.ts",
    "source/shared/local-exec-process-identity.ts",
    "source/electron-main/local-exec/local-exec-native.ts",
    "source/host/local-exec/local-exec-daemon-protocol.ts",
    "source/shared/node/mcp/mcp-result-factory.ts",
]


def save(receipt):
    (ROOT / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")


def git(*args):
    return subprocess.check_output(["git", *args])


def repository_path(path):
    """Metafile paths are relative to the shipping build's desktop working dir."""
    candidate = pathlib.Path(path)
    absolute = candidate.resolve(strict=True)
    relative = absolute.relative_to(pathlib.Path.cwd().resolve()).as_posix()
    if pathlib.Path(os.path.abspath(candidate)) != absolute:
        raise ValueError("Input traverses a symbolic link: " + str(path))
    if candidate.is_symlink() or not absolute.is_file():
        raise ValueError("Expected a regular owned input: " + str(path))
    return relative, absolute


def digest(path):
    relative, absolute = repository_path(path)
    raw = absolute.read_bytes()
    return {"path": relative, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def source_bindings(paths):
    result = {}
    for path in sorted(set(paths)):
        relative, absolute = repository_path(path)
        raw = absolute.read_bytes()
        committed = git("show", "HEAD:" + relative)
        if raw != committed:
            raise ValueError("Source differs from exact H: " + relative)
        tree_entry = git("ls-tree", "HEAD", "--", relative).decode("utf-8").strip()
        if not re.fullmatch(r"100(?:644|755) blob [0-9a-f]{40}\t" + re.escape(relative), tree_entry):
            raise ValueError("Source is not an exact-H regular blob: " + relative)
        result[relative] = {
            "git_blob": git("rev-parse", "HEAD:" + relative).decode("ascii").strip(),
            "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(),
            "matches_head_bytes": True,
        }
    return result


def rust_assessment(text, command_exit, capture_exit):
    text = text.replace("\r\n", "\n")
    observed = re.findall(r"^test (\S+) \.\.\. ok$", text, re.MULTILINE)
    summaries = [list(map(int, match)) for match in re.findall(
        r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; "
        r"(\d+) measured; (\d+) filtered out; finished in [0-9.]+s[ \t]*$",
        text, re.MULTILINE)]
    accepted = (command_exit == 0 and capture_exit == 0
                and sorted(observed) == sorted(RUST_NAMES)
                and summaries == [[len(RUST_NAMES), 0, 0, 0, 0]]
                and not re.search(r"^test result: FAILED|^test \S+ \.\.\. (?:FAILED|ignored)", text, re.MULTILINE))
    return {"required_positive_names": RUST_NAMES, "observed_positive_names": observed,
            "complete_summaries": summaries, "oracle_accepted": accepted}


def tap_assessment(text, command_exit, capture_exit):
    text = text.replace("\r\n", "\n")
    positives = re.findall(r"^ok (\d+) - ([^\n]+)$", text, re.MULTILINE)
    observed = [name for _, name in positives]
    numbers = [int(number) for number, _ in positives]
    expected = {"tests": 4, "suites": 0, "pass": 4, "fail": 0,
                "cancelled": 0, "skipped": 0, "todo": 0}
    summaries = {key: [int(value) for value in re.findall(
        r"^# " + key + r" (\d+)$", text, re.MULTILINE)] for key in expected}
    plans = [int(value) for value in re.findall(r"^1\.\.(\d+)$", text, re.MULTILINE)]
    accepted = (command_exit == 0 and capture_exit == 0
                and sorted(observed) == sorted(TAP_NAMES)
                and numbers == [1, 2, 3, 4] and plans == [4]
                and re.findall(r"^TAP version 13$", text, re.MULTILINE) == ["TAP version 13"]
                and all(summaries[key] == [value] for key, value in expected.items())
                and not re.search(r"^\s*(?:not ok\b|Bail out!)|#\s*(?:SKIP|TODO)\b", text, re.MULTILINE | re.IGNORECASE))
    return {"required_positive_names": TAP_NAMES, "observed_positive_names": observed,
            "positive_numbers": numbers, "plans": plans,
            "complete_summaries": summaries, "oracle_accepted": accepted}


def qualify_oracles():
    rust = "".join("test " + name + " ... ok\n" for name in RUST_NAMES)
    rust += "test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n"
    tap = "TAP version 13\n" + "".join(
        "ok " + str(number) + " - " + name + "\n" for number, name in enumerate(TAP_NAMES, 1))
    tap += "1..4\n# tests 4\n# suites 0\n# pass 4\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n"
    cases = [
        ("rust-positive", rust_assessment, rust, 0, 0, True),
        ("rust-missing-name", rust_assessment, rust.replace("test " + RUST_NAMES[0] + " ... ok\n", ""), 0, 0, False),
        ("rust-duplicate-name", rust_assessment, rust + "test " + RUST_NAMES[0] + " ... ok\n", 0, 0, False),
        ("rust-ignored", rust_assessment, rust.replace("0 ignored", "1 ignored"), 0, 0, False),
        ("rust-filtered", rust_assessment, rust.replace("0 filtered", "1 filtered"), 0, 0, False),
        ("rust-partial-summary", rust_assessment, rust.split("test result:")[0], 0, 0, False),
        ("rust-nonzero-exit", rust_assessment, rust, 9, 0, False),
        ("tap-positive", tap_assessment, tap, 0, 0, True),
        ("tap-zero", tap_assessment, "", 0, 0, False),
        ("tap-duplicate-name", tap_assessment, tap.replace(TAP_NAMES[1], TAP_NAMES[0]), 0, 0, False),
        ("tap-missing-name", tap_assessment, tap.replace("ok 1 - " + TAP_NAMES[0] + "\n", ""), 0, 0, False),
        ("tap-skipped", tap_assessment, tap.replace("# skipped 0", "# skipped 1"), 0, 0, False),
        ("tap-cancelled", tap_assessment, tap.replace("# cancelled 0", "# cancelled 1"), 0, 0, False),
        ("tap-todo-directive", tap_assessment, tap.replace(TAP_NAMES[0], TAP_NAMES[0] + " # TODO"), 0, 0, False),
        ("tap-duplicate-summary", tap_assessment, tap + "# pass 4\n", 0, 0, False),
        ("tap-wrong-plan", tap_assessment, tap.replace("1..4", "1..5"), 0, 0, False),
        ("tap-capture-error", tap_assessment, tap, 0, 1, False),
        ("tap-nonzero-exit", tap_assessment, tap, 3, 0, False),
        ("tap-unexpected-failure", tap_assessment, tap + "not ok 5 - late failure\n", 0, 0, False),
        ("rust-capture-error", rust_assessment, rust, 0, 1, False),
    ]
    controls = [{"control": name, "expected": expected,
                 "observed": assessor(text, code, capture)["oracle_accepted"]}
                for name, assessor, text, code, capture, expected in cases]
    return {"scope": "Synthetic result-oracle qualification; not product execution.",
            "controls": controls, "accepted": all(x["expected"] == x["observed"] for x in controls)}


def observe_unreaped_child(process):
    """Only this owner reaps the Popen child; waitid never releases its PID."""
    if process.returncode is not None:
        raise ChildProcessError("Owned child was already reaped; group identity is no longer reserved")
    info = os.waitid(os.P_PID, process.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
    if info is None:
        return {"pid": process.pid, "state": "not-exited", "wait_flags": "WEXITED|WNOHANG|WNOWAIT"}
    if info.si_pid != process.pid or info.si_code not in (os.CLD_EXITED, os.CLD_KILLED, os.CLD_DUMPED):
        raise ChildProcessError("Unexpected owned-child waitid identity or state")
    return {"pid": info.si_pid, "state": "exited", "si_code": info.si_code,
            "si_status": info.si_status, "wait_flags": "WEXITED|WNOHANG|WNOWAIT"}


def wait_unreaped(process, timeout_seconds):
    deadline = time.monotonic() + timeout_seconds
    while True:
        observed = observe_unreaped_child(process)
        if observed["state"] == "exited":
            return observed
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return None
        time.sleep(min(0.05, remaining))


def reap_observed_child(process, observed):
    if observed is None or observed["pid"] != process.pid or observed["state"] != "exited":
        raise ChildProcessError("Reaping requires the owned child's non-reaping exit observation")
    # No group signal may follow this call. WNOWAIT left this exact terminal
    # child waitable, so this waitpid does not wait for a live process.
    pid, status = os.waitpid(process.pid, 0)
    if pid != process.pid:
        raise ChildProcessError("Reaped a different child")
    process.returncode = os.waitstatus_to_exitcode(status)
    expected = observed["si_status"] if observed["si_code"] == os.CLD_EXITED else -observed["si_status"]
    if process.returncode != expected:
        raise ChildProcessError("Reaped status differs from the owned child's exit observation")
    return process.returncode


def observe_owned_session(process):
    observed = observe_unreaped_child(process)
    pgid, sid = os.getpgid(process.pid), os.getsid(process.pid)
    if pgid != process.pid or sid != process.pid:
        raise ChildProcessError("Owned child no longer anchors the session group created for it")
    return {**observed, "pgid": pgid, "sid": sid, "popen_returncode": process.returncode}


def stop_owned_session(process):
    """Keep the leader unreaped through the last group signal, or refuse it."""
    cleanup = {"pid": process.pid, "result": "unresolved", "events": [], "signals": [],
               "reap_confirmed": process.returncode is not None, "command_exit": process.returncode}
    try:
        for sig in (signal.SIGTERM, signal.SIGKILL):
            # A successful waitid establishes that this is still our unreaped
            # child. Its reserved PID and checked session/group identity make
            # the following killpg refer to the original owned group.
            observed = observe_owned_session(process)
            cleanup["events"].append({"event": "before-group-signal",
                                      "signal": sig.name, "observation": observed})
            os.killpg(process.pid, sig)
            cleanup["signals"].append(sig.name)
            cleanup["events"].append({"event": "group-signal", "signal": sig.name})
            exited = wait_unreaped(process, 5)
            if sig == signal.SIGTERM:
                cleanup["after_term"] = exited
        if exited is None:
            raise ChildProcessError("Owned leader did not exit after the final group signal")
        cleanup["command_exit"] = reap_observed_child(process, exited)
        cleanup["reap_confirmed"] = True
        cleanup["events"].append({"event": "reap", "command_exit": cleanup["command_exit"]})
        cleanup["result"] = "terminated-and-reaped"
    except (OSError, ValueError) as error:
        # No fallback uses a retained numeric PGID after an ownership failure.
        # Unresolved cleanup is non-passing, not inferred termination.
        cleanup["error"] = str(error)
        cleanup["command_exit"] = process.returncode
        cleanup["reap_confirmed"] = process.returncode is not None
    return cleanup


def read_probe_line(pipe, timeout_seconds=5):
    deadline, raw = time.monotonic() + timeout_seconds, bytearray()
    while len(raw) < 128:
        remaining = deadline - time.monotonic()
        if remaining <= 0 or not select.select([pipe.fileno()], [], [], remaining)[0]:
            raise ValueError("Owned cleanup qualification pipe timed out")
        byte = os.read(pipe.fileno(), 1)
        if not byte:
            raise ValueError("Owned cleanup qualification pipe closed before its response")
        raw.extend(byte)
        if byte == b"\n":
            return raw.decode("ascii")
    raise ValueError("Owned cleanup qualification response exceeded its bound")


def qualify_owned_cleanup():
    if os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux":
        raise ValueError("Actual owned-cleanup qualification is Linux Actions-only")
    leader_code = ("import signal,sys\n"
                   "signal.signal(signal.SIGTERM, lambda *_: sys.exit(23))\n"
                   "print('LEADER READY', flush=True)\n"
                   "while True: signal.pause()\n")
    peer_code = ("import sys\n"
                 "print('PEER READY', flush=True)\n"
                 "for line in sys.stdin: print('PEER ' + line.rstrip('\\n'), flush=True)\n")
    commands = {"leader": [sys.executable, "-u", "-c", leader_code],
                "peer": [sys.executable, "-u", "-c", peer_code]}
    log = ROOT / "owned-cleanup-qualification.log"
    evidence = {"scope": "Actual owned child and separate live peer; no PID-reuse simulation or product termination claim.",
                "commands": commands, "cwd": "repository root", "accepted": False,
                "stdout": {}, "teardown": [], "log": log.name}
    leader = peer = None
    cleanup_attempted = set()
    try:
        with log.open("wb") as errors:
            peer = subprocess.Popen(commands["peer"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                    stderr=errors, bufsize=0, start_new_session=True)
            leader = subprocess.Popen(commands["leader"], stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                      stderr=errors, bufsize=0, start_new_session=True)
            evidence["stdout"]["peer_ready"] = read_probe_line(peer.stdout)
            evidence["stdout"]["leader_ready"] = read_probe_line(leader.stdout)
            if evidence["stdout"] != {"peer_ready": "PEER READY\n", "leader_ready": "LEADER READY\n"}:
                raise ValueError("Cleanup qualification children did not establish readiness")
            evidence["peer_before"] = observe_owned_session(peer)
            if peer.pid == leader.pid or evidence["peer_before"]["pgid"] == leader.pid:
                raise ValueError("Qualification peer is not in a separate owned session")
            cleanup_attempted.add("leader")
            evidence["owned_cleanup"] = stop_owned_session(leader)
            cleanup = evidence["owned_cleanup"]
            final_guards = [event["observation"] for event in cleanup["events"]
                            if event["event"] == "before-group-signal" and event["signal"] == "SIGKILL"]
            if (cleanup["result"] != "terminated-and-reaped" or cleanup["command_exit"] != 23
                    or cleanup["signals"] != ["SIGTERM", "SIGKILL"]
                    or len(final_guards) != 1 or final_guards[0]["state"] != "exited"
                    or final_guards[0]["si_code"] != os.CLD_EXITED
                    or final_guards[0]["si_status"] != 23
                    or final_guards[0]["popen_returncode"] is not None
                    or cleanup["events"][-1]["event"] != "reap"):
                raise ValueError("Final group signal lacked the actual unreaped leader exit proof")
            # A real, already-reaped owner must not authorize another signal.
            evidence["reaped_owner_refusal"] = stop_owned_session(leader)
            refused = evidence["reaped_owner_refusal"]
            if refused["result"] != "unresolved" or refused["signals"] or refused["events"]:
                raise ValueError("Reaped owner was reused as group signalling authority")
            evidence["peer_after"] = observe_owned_session(peer)
            if evidence["peer_after"]["state"] != "not-exited":
                raise ValueError("Independent peer did not survive the owned group's cleanup")
            challenge = "after-cleanup-" + os.urandom(16).hex() + "\n"
            encoded = challenge.encode("ascii")
            if peer.stdin.write(encoded) != len(encoded):
                raise ValueError("Incomplete qualification peer challenge write")
            evidence["challenge"] = challenge
            evidence["stdout"]["peer_response"] = read_probe_line(peer.stdout)
            if evidence["stdout"]["peer_response"] != "PEER " + challenge:
                raise ValueError("Independent peer did not answer the fresh post-cleanup challenge")
            peer.stdin.close()
            evidence["peer_exit"] = reap_observed_child(peer, wait_unreaped(peer, 5))
            if evidence["peer_exit"] != 0:
                raise ValueError("Independent peer failed normal EOF shutdown")
            evidence["accepted"] = True
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        evidence["error"] = str(error)
    finally:
        for role, process in (("leader", leader), ("peer", peer)):
            if process is None:
                continue
            if process.returncode is None and role not in cleanup_attempted:
                cleanup_attempted.add(role)
                cleanup = stop_owned_session(process)
                evidence["teardown"].append({"role": role, **cleanup})
                if cleanup["result"] != "terminated-and-reaped":
                    evidence["accepted"] = False
            for pipe in (process.stdin, process.stdout):
                if pipe is not None and not pipe.closed:
                    try:
                        pipe.close()
                    except OSError as error:
                        evidence["accepted"] = False
                        evidence.setdefault("pipe_close_errors", []).append(str(error))
    raw = log.read_bytes() if log.is_file() else b""
    evidence.update({"log_bytes": len(raw), "log_sha256": hashlib.sha256(raw).hexdigest()})
    print(raw.decode("utf-8", errors="replace"), flush=True)
    return evidence


def run_command(label, command, timeout_seconds, environment=None, assessor=None):
    log = ROOT / (label + ".log")
    result = {"case": label, "command": command, "cwd": "repository root",
              "timeout_seconds": timeout_seconds, "command_exit": None, "capture_exit": 0,
              "execution_error": None, "log": log.name, "oracle_accepted": False}
    raw = b""
    try:
        with log.open("wb") as stream:
            process = subprocess.Popen(command, stdout=stream, stderr=subprocess.STDOUT,
                                       env=environment, start_new_session=True)
            cleanup_attempted = False
            try:
                observed = wait_unreaped(process, timeout_seconds)
                if observed is None:
                    result["capture_exit"] = 1
                    result["execution_error"] = "Owned command session exceeded its deadline"
                    cleanup_attempted = True
                    result["cleanup"] = stop_owned_session(process)
                    result["command_exit"] = result["cleanup"]["command_exit"]
                else:
                    result["exit_observation"] = observed
                    result["command_exit"] = reap_observed_child(process, observed)
            finally:
                # Reading returncode does not reap. If a prior cleanup was
                # unresolved, do not repeat it with a potentially stale PGID.
                if process.returncode is None and not cleanup_attempted:
                    result["capture_exit"] = 1
                    cleanup_attempted = True
                    result["cleanup"] = stop_owned_session(process)
                    result["command_exit"] = result["cleanup"]["command_exit"]
        raw = log.read_bytes()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        result["capture_exit"] = 1
        result["execution_error"] = str(error)
        if log.is_file():
            raw = log.read_bytes()
    text = raw.decode("utf-8", errors="replace")
    result.update({"log_bytes": len(raw), "log_sha256": hashlib.sha256(raw).hexdigest()})
    if assessor is not None:
        result.update(assessor(text, result["command_exit"], result["capture_exit"]))
    else:
        result["oracle_accepted"] = result["command_exit"] == 0 and result["capture_exit"] == 0
    print(text, flush=True)
    return result

def bundle_inputs():
    metadata = json.loads(META.read_text(encoding="utf-8"))
    output_entry = None
    for name, detail in metadata["outputs"].items():
        path = pathlib.Path("desktop") / name
        relative, _ = repository_path(path)
        if relative == BUNDLE.as_posix():
            output_entry = detail
    if output_entry is None or "entryPoint" not in output_entry:
        raise ValueError("Bundle metadata does not identify the requested entry")
    entry, _ = repository_path(pathlib.Path("desktop") / output_entry["entryPoint"])
    if entry != ENTRY or output_entry["bytes"] != BUNDLE.stat().st_size:
        raise ValueError("Bundle entry or output byte count differs from the build metadata")
    source_paths, installed = [], {}
    for name in metadata["inputs"]:
        relative, absolute = repository_path(pathlib.Path("desktop") / name)
        if relative.startswith("desktop/node_modules/"):
            installed[relative] = digest(absolute)
        else:
            source_paths.append(relative)
    required = {ENTRY, "source/electron-main/coordinator/coordinator-control-server.ts",
                "source/electron-main/coordinator/coordinator-executors.ts",
                "source/shared/local-exec-process-identity.ts"}
    if not required.issubset(source_paths):
        raise ValueError("Actual identity entry, carrier or consumer is missing from the bundle")
    return {"repository_sources": source_bindings(source_paths),
            "installed_dependencies": installed,
            "dependency_role": "Actual npm-ci installed inputs, bound by desktop/package-lock.json; not Git source or fixed-U dependency closure."}


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Executable verification is GitHub Actions-only.")
    ROOT.mkdir(parents=True, exist_ok=False)
    receipt = {"contract": "LPI-v1", "result": "failed-or-incomplete", "executions": [],
               "scope": "Actual exact-H Coordinator control producer and shipping TypeScript control server/identity consumer with controlled native observations and daemon handle. Includes existing Rust supervisor state machines and all optional-wire combinations. Not real OS inspection, external accounts/devices, HTML provenance, package or global OCRM acceptance."}
    save(receipt)
    try:
        if os.environ["RUNNER_OS"] != "Linux" or sys.platform != "linux":
            raise ValueError("This selected cross-language instrument requires the Linux Actions job")
        head = git("rev-parse", "HEAD").decode("ascii").strip()
        if not re.fullmatch(r"[0-9a-f]{40}", head) or head != os.environ["FABUSHI_TARGET_SHA"]:
            raise ValueError("Exact LPI subject mismatch")
        if os.environ.get("EXPECTED_DISPATCH_SHA") not in (None, "", head):
            raise ValueError("Dispatch ref moved from its frozen subject")
        lock = json.loads(pathlib.Path("projects/openclaw-rust/upstream.lock.json").read_text(encoding="utf-8"))
        upstream = {key: lock["upstream"][key] for key in UPSTREAM}
        if upstream != UPSTREAM:
            raise ValueError("Fixed upstream identity drift")
        rust_paths = git("ls-files", "-z", "--", CRATE + "/src", CRATE + "/build.rs",
                         CRATE + "/.cargo", ".cargo", "source/.cargo",
                         "rust-toolchain", "rust-toolchain.toml").decode("utf-8").split("\0")
        inputs = sorted(set(DIRECT_INPUTS + [path for path in rust_paths if path]))
        if CRATE + "/src/main.rs" not in inputs or CRATE + "/src/local_exec/supervisor.rs" not in inputs:
            raise ValueError("Coordinator source set is incomplete")
        receipt.update({"subject_commit": head,
                        "subject_tree": git("rev-parse", "HEAD^{tree}").decode("ascii").strip(),
                        "upstream": upstream, "run_id": os.environ["GITHUB_RUN_ID"],
                        "run_attempt": os.environ["GITHUB_RUN_ATTEMPT"], "job": os.environ["GITHUB_JOB"],
                        "runner_os": os.environ["RUNNER_OS"], "python": sys.version,
                        "rustc": subprocess.check_output(["rustc", "--version", "--verbose"], text=True).strip(),
                        "node": subprocess.check_output(["node", "--version"], text=True).strip(),
                        "inputs_before": source_bindings(inputs), "oracle_qualification": qualify_oracles(),
                        "environment": {key: os.environ.get(key) for key in (
                            "CARGO_BUILD_JOBS", "CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG",
                            "CARGO_INCREMENTAL", "CARGO_TARGET_DIR")}})
        save(receipt)
        if not receipt["oracle_qualification"]["accepted"]:
            raise ValueError("LPI result oracle qualification failed")
        receipt["active_case"] = "owned-cleanup-qualification"
        save(receipt)
        receipt["cleanup_qualification"] = qualify_owned_cleanup()
        save(receipt)
        if not receipt["cleanup_qualification"]["accepted"]:
            raise ValueError("Actual owned-child cleanup qualification failed")
        cargo = ["cargo", "test", "--locked", "--target-dir", "source/host/target",
                 "--manifest-path", CRATE + "/Cargo.toml"]
        planned = [
            ("rust-wire-and-lifecycle", cargo + ["--test", "local_exec_supervisor_runtime", "--", "--test-threads=1"], 180, rust_assessment),
            ("coordinator-normal-build", ["cargo", "build", "--locked", "--target-dir", "source/host/target",
                                         "--manifest-path", CRATE + "/Cargo.toml", "--bin", "mahayana-node-agent-coordinator"], 180, None),
            ("typescript-bundle", ["node", "desktop/scripts/build-electron-runtime.mjs",
                                  "--local-exec-identity-contracts"], 90, None),
        ]
        receipt["planned_commands"] = [{"case": label, "command": command, "timeout_seconds": timeout}
                                       for label, command, timeout, _ in planned]
        receipt["planned_commands"].append({"case": "actual-control-wire",
            "command": ["node", "--test", "--test-reporter=tap", str(BUNDLE)], "timeout_seconds": 180})
        for label, command, timeout, assessor in planned:
            receipt["active_case"] = label
            save(receipt)
            result = run_command(label, command, timeout, assessor=assessor)
            receipt["executions"].append(result)
            save(receipt)
            if not result["oracle_accepted"]:
                raise ValueError("Required LPI command failed: " + label)
        if not os.access(BINARY, os.X_OK) or BINARY.stat().st_size == 0:
            raise ValueError("The same-H build produced no normal executable")
        producer_paths = [BINARY, BUNDLE, META, pathlib.Path(str(BUNDLE) + ".map")]
        receipt["producer_before"] = {str(path): digest(path) for path in producer_paths}
        receipt["bundled_inputs_before"] = bundle_inputs()
        receipt["bundle_source_binding_phase"] = "After bundle build and before actual control cases; every repository input matches exact H."
        receipt["inputs_before_cases"] = source_bindings(inputs)
        if receipt["inputs_before_cases"] != receipt["inputs_before"]:
            raise ValueError("A build command changed a causal source input")
        receipt["active_case"] = "actual-control-wire"
        binary = str(BINARY.resolve(strict=True))
        receipt["test_environment"] = {"FABUSHI_COORDINATOR_TEST_BIN": BINARY.as_posix()}
        save(receipt)
        result = run_command("actual-control-wire",
            ["node", "--test", "--test-reporter=tap", str(BUNDLE)], 180,
            environment=dict(os.environ, FABUSHI_COORDINATOR_TEST_BIN=binary), assessor=tap_assessment)
        receipt["executions"].append(result)
        receipt["inputs_after"] = source_bindings(inputs)
        receipt["producer_after"] = {str(path): digest(path) for path in producer_paths}
        receipt["bundled_inputs_after"] = bundle_inputs()
        receipt["inputs_unchanged"] = receipt["inputs_before"] == receipt["inputs_after"]
        receipt["producer_unchanged"] = receipt["producer_before"] == receipt["producer_after"]
        receipt["bundled_inputs_unchanged"] = receipt["bundled_inputs_before"] == receipt["bundled_inputs_after"]
        accepted = (receipt["cleanup_qualification"]["accepted"]
                    and len(receipt["executions"]) == 4
                    and all(case["oracle_accepted"] for case in receipt["executions"])
                    and receipt["inputs_unchanged"] and receipt["producer_unchanged"]
                    and receipt["bundled_inputs_unchanged"])
        receipt["result"] = "passed" if accepted else "failed-or-incomplete"
        receipt.pop("active_case", None)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        receipt["capture_error"] = str(error)
    save(receipt)
    print(json.dumps(receipt, indent=2), flush=True)
    return 0 if receipt["result"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
