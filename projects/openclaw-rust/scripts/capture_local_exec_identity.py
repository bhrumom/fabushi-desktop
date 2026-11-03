#!/usr/bin/env python3
"""Capture LPI-v1 and the selected LDB-v1 daemon on their exact Actions subject."""

import hashlib
import json
import os
import pathlib
import re
import select
import signal
import shutil
import subprocess
import sys
import tarfile
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
HOST_BINARY = pathlib.Path("source/host/target/debug/mahayana-app-host")
DAEMON_ENTRY = "source/local-exec-daemon/main.ts"
DAEMON_BUNDLE = pathlib.Path("desktop/dist/local-exec-daemon/main.cjs")
DAEMON_META = pathlib.Path("desktop/dist/local-exec-daemon/build-metafile.json")
DAEMON_MANIFEST = pathlib.Path("desktop/dist/local-exec-daemon/build-manifest.json")
STARTUP_ENTRY = "source/local-exec-daemon/startup.contract.test.ts"
STARTUP_BUNDLE = OUT / "startup.contract.test.cjs"
STARTUP_META = OUT / "startup-build-metafile.json"
DEPENDENCY_TEST = "desktop/scripts/local-exec-dependencies.contract.test.mjs"
LDB_SPEC = "projects/openclaw-rust/docs/specs/local-exec-outcome-ownership.md"
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
NATIVE_NAMES = [
    "LDB real Node and Electron stages load the parser under their actual runtimes",
    "LDB structural checking accepts complete synthetic stages without installed dependencies",
    "LDB structural checking refuses wrong-domain stages without fallback",
    "LDB structural checking refuses absent native outputs even with a rewritten receipt",
    "LDB structural checking detects changed native bytes",
    "LDB structural checking rejects undeclared stage members",
    "LDB structural checking rejects a symlink or junction escape",
    "LDB structural checking rejects traversal and duplicate manifest paths",
    "LDB structural checking rejects stale lock and builder identities",
    "LDB structural checking rejects missing causal source inputs",
    "LDB structural checking detects changed package source behind rewritten output hashes",
    "LDB structural checking rejects missing package license artifacts",
    "LDB structural checking rejects unqualified native fallback locations",
    "LDB structural checking rejects altered official archive checksums",
    "LDB structural checking rejects mismatched runtime header ABI",
    "LDB structural checking rejects a failed parser load receipt",
    "LDB stock-only C++ patch checks both exact input and output identities",
    "LDB owned child runner rejects spawn errors and nonzero close",
    "LDB owned child deadline cleanup leaves an unrelated process alive",
    "LDB owned child runner waits for close without signaling an exited child",
    "LDB owned native build failure retains only its bound work root"
]
STARTUP_NAMES = [
    "ldb Host resolver preserves packaged development and explicit override mappings",
    "ldb emitted daemon transfers bytes through real provider and retires owned discovery",
    "ldb emitted daemon refuses mismatched generation before registration",
    "ldb emitted daemon refuses missing configured Node parser domain without fallback",
    "ldb emitted daemon refuses wrong-domain parser manifest before registration",
    "ldb emitted daemon refuses mismatched parser runtime before registration",
    "ldb Node event adapter preserves normalization reporter replacement and clearing"
]
DIRECT_INPUTS = [
    ".github/workflows/rust-desktop-runtime.yml", SPEC, SCRIPT,
    "projects/openclaw-rust/upstream.lock.json",
    CRATE + "/Cargo.toml", CRATE + "/Cargo.lock",
    CRATE + "/tests/local_exec_supervisor_runtime.rs",
    ENTRY, DAEMON_ENTRY, STARTUP_ENTRY, LDB_SPEC, DEPENDENCY_TEST,
    "desktop/package.json", "desktop/package-lock.json",
    "source/host/app/Cargo.toml", "source/host/app/Cargo.lock",
    "source/host/Cargo.toml", "source/host/Cargo.lock", "source/host/build.rs",
    "source/host/src/extensions/local_exec/mod.rs", "source/host/src/extensions/local_exec/native_file_sandbox.rs",
    "desktop/scripts/build-local-exec-dependencies.mjs",
    "desktop/scripts/local-exec-dependencies.fixture.mjs",
    "desktop/scripts/check-electron-runtime-bundle.mjs",
    "desktop/scripts/openclaw-notice.contract.test.mjs",
    "desktop/scripts/build-electron-runtime.mjs",
    "source/electron-main/coordinator/coordinator-control-server.ts",
    "source/electron-main/coordinator/coordinator-executors.ts",
    "source/shared/rpc/coordinator-port.ts",
    "source/shared/local-exec-process-identity.ts",
    "source/electron-main/local-exec/local-exec-native.ts",
    "source/electron-main/app-host-executable.ts",
    "source/electron-main/coordinator/rust-coordinator-carrier.ts",
    "source/packages/shell-exec/shell-parser.ts",
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


def tap_assessment(text, command_exit, capture_exit, required_names=TAP_NAMES):
    text = text.replace("\r\n", "\n")
    positives = re.findall(r"^ok (\d+) - ([^\n]+)$", text, re.MULTILINE)
    observed = [name for _, name in positives]
    numbers = [int(number) for number, _ in positives]
    count = len(required_names)
    expected = {"tests": count, "suites": 0, "pass": count, "fail": 0,
                "cancelled": 0, "skipped": 0, "todo": 0}
    summaries = {key: [int(value) for value in re.findall(
        r"^# " + key + r" (\d+)$", text, re.MULTILINE)] for key in expected}
    plans = [int(value) for value in re.findall(r"^1\.\.(\d+)$", text, re.MULTILINE)]
    accepted = (command_exit == 0 and capture_exit == 0
                and count > 0 and len(set(required_names)) == count
                and sorted(observed) == sorted(required_names)
                and numbers == list(range(1, count + 1)) and plans == [count]
                and re.findall(r"^TAP version 13$", text, re.MULTILINE) == ["TAP version 13"]
                and all(summaries[key] == [value] for key, value in expected.items())
                and not re.search(r"^\s*(?:not ok\b|Bail out!)|#\s*(?:SKIP|TODO)\b", text, re.MULTILINE | re.IGNORECASE))
    return {"required_positive_names": required_names, "observed_positive_names": observed,
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
    for label, names in (("native", NATIVE_NAMES), ("startup", STARTUP_NAMES)):
        count = len(names)
        positive = "TAP version 13\n" + "".join(
            "ok " + str(number) + " - " + name + "\n" for number, name in enumerate(names, 1))
        positive += ("1.." + str(count) + "\n# tests " + str(count) + "\n# suites 0\n# pass "
                     + str(count) + "\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n")
        for suffix, value, code, capture, expected in [
            ("positive", positive, 0, 0, True),
            ("crlf", positive.replace("\n", "\r\n"), 0, 0, True),
            ("empty", "", 0, 0, False),
            ("duplicate", positive.replace(names[1], names[0]), 0, 0, False),
            ("missing", positive.replace("ok 1 - " + names[0] + "\n", ""), 0, 0, False),
            ("unexpected", positive.replace(names[0], "undeclared positive"), 0, 0, False),
            ("wrong-plan", positive.replace("1.." + str(count), "1..0"), 0, 0, False),
            ("skipped", positive.replace("# skipped 0", "# skipped 1"), 0, 0, False),
            ("cancelled", positive.replace("# cancelled 0", "# cancelled 1"), 0, 0, False),
            ("todo", positive.replace(names[0], names[0] + " # TODO"), 0, 0, False),
            ("duplicate-summary", positive + "# pass " + str(count) + "\n", 0, 0, False),
            ("failed", positive + "not ok 99 - unexpected failure\n", 0, 0, False),
            ("nonzero", positive, 7, 0, False),
            ("capture-error", positive, 0, 1, False),
        ]:
            controls.append({"control": label + "-" + suffix, "expected": expected,
                             "observed": tap_assessment(value, code, capture, names)["oracle_accepted"]})
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

def bundle_inputs(bundle=BUNDLE, meta=META, expected_entry=ENTRY, required=None):
    metadata = json.loads(meta.read_text(encoding="utf-8"))
    output_entry = None
    for name, detail in metadata["outputs"].items():
        relative, _ = repository_path(pathlib.Path("desktop") / name)
        if relative == bundle.as_posix():
            output_entry = detail
    if output_entry is None or "entryPoint" not in output_entry:
        raise ValueError("Bundle metadata does not identify the requested entry")
    entry, _ = repository_path(pathlib.Path("desktop") / output_entry["entryPoint"])
    if entry != expected_entry or output_entry["bytes"] != bundle.stat().st_size:
        raise ValueError("Bundle entry or output byte count differs from the build metadata")
    source_paths, installed = [], {}
    for name in metadata["inputs"]:
        relative, absolute = repository_path(pathlib.Path("desktop") / name)
        if relative.startswith("desktop/node_modules/"):
            installed[relative] = digest(absolute)
        else:
            source_paths.append(relative)
    if required is None:
        required = {ENTRY, "source/electron-main/coordinator/coordinator-control-server.ts",
                    "source/electron-main/coordinator/coordinator-executors.ts",
                    "source/shared/local-exec-process-identity.ts"}
    if not set(required).issubset(source_paths):
        raise ValueError("An actual entry, carrier or consumer is missing from the bundle")
    return {"repository_sources": source_bindings(source_paths),
            "installed_dependencies": installed,
            "dependency_role": "Actual npm-ci installed inputs, bound by desktop/package-lock.json; not Git source or fixed-U dependency closure."}


def all_bundle_inputs():
    return {
        "identity": bundle_inputs(),
        "daemon": bundle_inputs(DAEMON_BUNDLE, DAEMON_META, DAEMON_ENTRY, {
            DAEMON_ENTRY, "source/local-exec-daemon/production-executor.ts",
            "source/host/local-exec/local-exec-daemon.ts",
            "source/host/local-exec/local-exec-provider.ts",
            "source/host/local-exec/local-exec-native-file-access.ts",
            "source/packages/shell-exec/shell-parser.ts",
        }),
        "startup": bundle_inputs(STARTUP_BUNDLE, STARTUP_META, STARTUP_ENTRY, {
            STARTUP_ENTRY, "source/electron-main/local-exec/local-exec-native.ts",
            "source/electron-main/app-host-executable.ts",
        }),
    }


def host_source_paths(metadata):
    """Bind all tracked members of the actually resolved local Host packages."""
    packages, paths = [], set()
    repository = pathlib.Path.cwd().resolve()
    for package in metadata["packages"]:
        if package["source"] is not None:
            continue
        manifest, absolute = repository_path(package["manifest_path"])
        directory = absolute.parent
        relative = directory.relative_to(repository).as_posix()
        tracked = git("ls-files", "-z", "--", relative).decode("utf-8").split("\0")
        members = [item for item in tracked if item]
        if manifest not in members:
            raise ValueError("Resolved local package manifest is not tracked at H")
        paths.update(members)
        packages.append({"name": package["name"], "version": package["version"],
                         "manifest": manifest, "tracked_input_members": len(members)})
        while directory == repository or repository in directory.parents:
            for candidate in (directory / "Cargo.toml", directory / "Cargo.lock",
                              directory / ".cargo/config", directory / ".cargo/config.toml"):
                if candidate.is_file():
                    item, _ = repository_path(candidate)
                    if git("ls-files", "--", item).strip():
                        paths.add(item)
            if directory == repository:
                break
            directory = directory.parent
    required = {"source/host/app/src/main.rs", "source/host/src/extensions/local_exec/native_file_sandbox.rs",
                "source/host/src/lib.rs", "source/host/build.rs"}
    if not required.issubset(paths):
        raise ValueError("Same-H Host native source closure is incomplete")
    return sorted(paths), sorted(packages, key=lambda item: item["manifest"])


def staged_inputs():
    """Observe all declared outputs; the native contract separately proves load."""
    result = {"domains": {}, "files": {}}
    for domain, directory in (("node", "node-deps"), ("electron", "deps")):
        root = pathlib.Path("desktop/dist") / directory
        manifest = root / "local-exec-dependencies.json"
        data = json.loads(manifest.read_text(encoding="utf-8"))
        if data["contract"] != "LDB-v1" or data["domain"] != domain:
            raise ValueError("Native dependency stage domain mismatch")
        names = [entry["path"] for entry in data["stage_files"]]
        if len(names) != len(set(names)):
            raise ValueError("Duplicate dependency output")
        actual = []
        for member in root.rglob("*"):
            if member.is_symlink():
                raise ValueError("Symbolic native stage member")
            if member.is_file() and member != manifest:
                actual.append(member.relative_to(root).as_posix())
            elif not member.is_dir() and member != manifest:
                raise ValueError("Non-regular native stage member")
        if sorted(actual) != sorted(names):
            raise ValueError("Native stage output membership differs")
        for entry in data["stage_files"]:
            path = root / entry["path"]
            value = digest(path)
            if value["bytes"] != entry["bytes"] or value["sha256"] != entry["sha256"]:
                raise ValueError("Native stage bytes differ: " + value["path"])
            result["files"][value["path"]] = value
        result["files"][manifest.as_posix()] = digest(manifest)
        result["domains"][domain] = data
    manifest = json.loads(DAEMON_MANIFEST.read_text(encoding="utf-8"))
    for key in ("external_library", "legal_inputs"):
        for entry in manifest[key]["files"]:
            value = digest(pathlib.Path("desktop") / entry["path"])
            if value["bytes"] != entry["bytes"] or value["sha256"] != entry["sha256"]:
                raise ValueError("Retained library/source artifact differs")
            result["files"][value["path"]] = value
    return result


def bind_native_build_work(evidence_root):
    """Create and bind one invocation-owned native work root before any build."""
    root = evidence_root / "native-build-work"
    root.mkdir(parents=False, exist_ok=False)
    absolute = root.resolve(strict=True)
    if root.is_symlink() or pathlib.Path(os.path.abspath(root)) != absolute:
        raise ValueError("Native build work root traverses a symbolic link")
    info = root.stat()
    return {"path": root.as_posix(), "absolute_path": str(absolute),
            "device": info.st_dev, "inode": info.st_ino}


def retain_native_build_work(binding, evidence_root):
    """Archive exactly the pre-bound root, including hidden headers, without cleanup."""
    root = pathlib.Path(binding["absolute_path"])
    if root.is_symlink() or root.resolve(strict=True) != root:
        raise ValueError("Native build work root was redirected")
    info = root.stat()
    if (info.st_dev, info.st_ino) != (binding["device"], binding["inode"]) or not root.is_dir():
        raise ValueError("Native build work root identity changed")
    expected = (evidence_root / "native-build-work").resolve(strict=True)
    if root != expected:
        raise ValueError("Native build work root belongs to another invocation")
    archive = evidence_root / "native-build-work.tar"
    def admitted(member):
        if not (member.isfile() or member.isdir() or member.issym() or member.islnk()):
            raise ValueError("Unsupported native work member: " + member.name)
        return member
    # tar stores links as links and never reads their target. No broad glob or
    # directory deletion is involved, even if command settlement is unresolved.
    with tarfile.open(archive, "x", dereference=False) as retained:
        retained.add(root, arcname="native-build-work", recursive=True, filter=admitted)
    with tarfile.open(archive, "r") as retained:
        members = len(retained.getmembers())
    return {"binding": binding, "archive": digest(archive), "members": members,
            "scope": "Snapshot of the exact owned work root, including hidden headers and retained prior stages. No child quiescence or successful native build is inferred from archival."}


def qualify_native_build_retention():
    """Qualify the archive instrument on real files, including a hidden member."""
    evidence_root = ROOT / "native-retention-control"
    evidence_root.mkdir(exist_ok=False)
    binding = bind_native_build_work(evidence_root)
    work = pathlib.Path(binding["absolute_path"])
    hidden = work / "candidate" / ".headers" / "archive.tar.gz"
    hidden.parent.mkdir(parents=True)
    hidden.write_bytes(b"controlled header archive bytes")
    prior = work / "previous" / "runtime.node"
    prior.parent.mkdir()
    prior.write_bytes(b"controlled previous native bytes")
    peer = evidence_root / "unrelated-marker"
    peer.write_bytes(b"unrelated retained peer")
    retained = retain_native_build_work(binding, evidence_root)
    with tarfile.open(evidence_root / "native-build-work.tar", "r") as archive:
        if (archive.extractfile("native-build-work/candidate/.headers/archive.tar.gz").read() != hidden.read_bytes()
                or archive.extractfile("native-build-work/previous/runtime.node").read() != prior.read_bytes()
                or any("unrelated-marker" in member.name for member in archive.getmembers())):
            raise ValueError("Owned native work archive lost or widened its members")
    refused = False
    try:
        retain_native_build_work({**binding, "inode": binding["inode"] + 1}, evidence_root)
    except ValueError:
        refused = True
    if not refused or peer.read_bytes() != b"unrelated retained peer" or not hidden.is_file() or not prior.is_file():
        raise ValueError("Native work retention lost ownership or removed fixture bytes")
    return {"accepted": True, "archive": retained, "wrong_identity_refused": refused,
            "scope": "Instrument qualification only; no native compilation or product behavior claim."}


def retain_selected_products():
    """Keep actual emitted bytes and native receipts; no synthetic replacement."""
    products = [DAEMON_BUNDLE, DAEMON_META, DAEMON_MANIFEST,
                BUNDLE, META, STARTUP_BUNDLE, STARTUP_META]
    for directory in ("node-deps", "deps"):
        root = pathlib.Path("desktop/dist") / directory
        data = json.loads((root / "local-exec-dependencies.json").read_text(encoding="utf-8"))
        products.append(root / "local-exec-dependencies.json")
        products.extend(root / item["path"] for item in data["native_outputs"])
        products.extend(root / item["path"] for item in data["headers"]["files"]
                        if item["path"].endswith((".tar.gz", "SHASUMS256.txt", "downloads.json")))
    retained = {}
    for path in products:
        identity = digest(path)
        destination = ROOT / "build-products" / identity["path"]
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
        if destination.read_bytes() != path.read_bytes():
            raise ValueError("Retained build product differs")
        retained[identity["path"]] = {**identity, "retained": destination.relative_to(ROOT).as_posix()}
    return retained


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Executable verification is GitHub Actions-only.")
    ROOT.mkdir(parents=True, exist_ok=False)
    receipt = {"contract": "LPI-v1/LDB-v1", "result": "failed-or-incomplete", "executions": [],
               "scope": "Actual exact-H Rust Coordinator producer and TypeScript identity consumer, separately qualified Node/Electron parser stages, and emitted packaged-Node daemon through its real provider/executor and same-H Rust Host file helper. Controlled loopback transport exercises actual upload/download bytes, generation refusal, parser-domain refusal, discovery retirement and Node event adapter. Native fixture cases are structural evidence only. The actual native launch adapter is exercised under ordinary Node with a controlled Electron resource context and the same-H staged Host. It does not establish real installed Electron launch end-to-end, full shell/LS parity, physical providers/devices, final dependency legal acceptance, package/upgrade/rollback or global OCRM acceptance."}
    save(receipt)
    try:
        selected_scope = os.environ.get("OCRM_LOCAL_EXEC_SCOPE", "full-lane-local-exec")
        if selected_scope not in ("full-lane-local-exec", "ocrm-local-exec-runtime"):
            raise ValueError("Unknown LocalExec execution selection")
        receipt["validation_scope"] = selected_scope
        receipt["selection_limit"] = "Only the declared LPI/LDB commands are assessed. Unselected historical failures and unrelated full-workflow gates remain unaccepted."
        if os.environ["RUNNER_OS"] != "Linux" or sys.platform != "linux":
            raise ValueError("This selected cross-language instrument requires the Linux Actions job")
        head = git("rev-parse", "HEAD").decode("ascii").strip()
        if not re.fullmatch(r"[0-9a-f]{40}", head) or head != os.environ["FABUSHI_TARGET_SHA"]:
            raise ValueError("Exact LPI/LDB subject mismatch")
        if os.environ.get("EXPECTED_DISPATCH_SHA") not in (None, "", head):
            raise ValueError("Dispatch ref moved from its frozen subject")
        lock = json.loads(pathlib.Path("projects/openclaw-rust/upstream.lock.json").read_text(encoding="utf-8"))
        upstream = {key: lock["upstream"][key] for key in UPSTREAM}
        if upstream != UPSTREAM:
            raise ValueError("Fixed upstream identity drift")
        rust_paths = git("ls-files", "-z", "--", CRATE + "/src", CRATE + "/build.rs",
                         CRATE + "/.cargo", ".cargo", "source/.cargo",
                         "rust-toolchain", "rust-toolchain.toml",
                         "desktop/resources/licenses/local-exec").decode("utf-8").split("\0")
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
            raise ValueError("LPI/LDB result oracle qualification failed")
        receipt["active_case"] = "native-build-retention-qualification"
        save(receipt)
        receipt["native_build_retention_qualification"] = qualify_native_build_retention()
        receipt["native_build_work"] = bind_native_build_work(ROOT)
        save(receipt)
        receipt["active_case"] = "owned-cleanup-qualification"
        save(receipt)
        receipt["cleanup_qualification"] = qualify_owned_cleanup()
        save(receipt)
        if not receipt["cleanup_qualification"]["accepted"]:
            raise ValueError("Actual owned-child cleanup qualification failed")
        cargo = ["cargo", "test", "--locked", "--target-dir", "source/host/target",
                 "--manifest-path", CRATE + "/Cargo.toml"]
        planned = [
            ("host-dependency-graph", ["cargo", "metadata", "--locked", "--format-version", "1",
                                      "--manifest-path", "source/host/app/Cargo.toml"], 120, None),
            ("typescript-bundle", ["node", "desktop/scripts/build-electron-runtime.mjs",
                                  "--local-exec-identity-contracts"], 900, None),
            ("rust-wire-and-lifecycle", cargo + ["--test", "local_exec_supervisor_runtime", "--", "--test-threads=1"], 180, rust_assessment),
            ("coordinator-normal-build", ["cargo", "build", "--locked", "--target-dir", "source/host/target",
                                         "--manifest-path", CRATE + "/Cargo.toml", "--bin", "mahayana-node-agent-coordinator"], 180, None),
            ("host-normal-build", ["cargo", "build", "--locked", "--target-dir", "source/host/target",
                                  "--manifest-path", "source/host/app/Cargo.toml", "--bin", "mahayana-app-host"], 600, None),
        ]
        behavior_cases = [
            ("actual-native-domains", ["node", "--test", "--test-reporter=tap", DEPENDENCY_TEST], 180, NATIVE_NAMES),
            ("actual-control-wire", ["node", "--test", "--test-reporter=tap", str(BUNDLE)], 180, TAP_NAMES),
            ("actual-emitted-daemon", ["node", "--test", "--test-reporter=tap", str(STARTUP_BUNDLE)], 180, STARTUP_NAMES),
        ]
        receipt["planned_commands"] = [{"case": label, "command": command, "timeout_seconds": timeout}
                                       for label, command, timeout, _ in planned + behavior_cases]
        host_paths = []
        for label, command, timeout, assessor in planned:
            receipt["active_case"] = label
            save(receipt)
            command_environment = None
            if label == "typescript-bundle":
                command_environment = dict(os.environ,
                    FABUSHI_LOCAL_EXEC_BUILD_WORK_ROOT=receipt["native_build_work"]["absolute_path"])
            result = run_command(label, command, timeout, environment=command_environment, assessor=assessor)
            receipt["executions"].append(result)
            save(receipt)
            if not result["oracle_accepted"]:
                raise ValueError("Required LPI/LDB command failed: " + label)
            if label == "host-dependency-graph":
                metadata = json.loads((ROOT / result["log"]).read_text(encoding="utf-8"))
                host_paths, packages = host_source_paths(metadata)
                receipt["resolved_local_host_packages"] = packages
                receipt["host_sources_before"] = source_bindings(host_paths)
                receipt["host_source_scope"] = "All tracked members of each resolved local Host package and tracked Cargo/config ancestors; includes a superset of compiled source. Registry packages remain bound by Cargo.lock and actual metadata/build."
                save(receipt)
        for binary in (BINARY, HOST_BINARY):
            if not os.access(binary, os.X_OK) or binary.stat().st_size == 0:
                raise ValueError("The same-H build produced no normal executable: " + str(binary))
        producer_paths = [BINARY, HOST_BINARY, BUNDLE, META, pathlib.Path(str(BUNDLE) + ".map"),
                          DAEMON_BUNDLE, DAEMON_META, DAEMON_MANIFEST, pathlib.Path(str(DAEMON_BUNDLE) + ".map"),
                          STARTUP_BUNDLE, STARTUP_META, pathlib.Path(str(STARTUP_BUNDLE) + ".map")]
        receipt["producer_before"] = {str(path): digest(path) for path in producer_paths}
        receipt["bundled_inputs_before"] = all_bundle_inputs()
        receipt["staged_inputs_before"] = staged_inputs()
        receipt["retained_build_products"] = retain_selected_products()
        receipt["bundle_source_binding_phase"] = "After build and before actual cases; every repository input matches exact H; installed/native/generated identities retain their separate origins."
        receipt["inputs_before_cases"] = source_bindings(inputs)
        receipt["host_sources_before_cases"] = source_bindings(host_paths)
        if (receipt["inputs_before_cases"] != receipt["inputs_before"]
                or receipt["host_sources_before_cases"] != receipt["host_sources_before"]):
            raise ValueError("A build command changed a causal source input")
        environment = dict(os.environ,
                           FABUSHI_COORDINATOR_TEST_BIN=str(BINARY.resolve(strict=True)),
                           FABUSHI_LOCAL_EXEC_HOST_TEST_BIN=str(HOST_BINARY.resolve(strict=True)),
                           FABUSHI_LOCAL_EXEC_DAEMON_TEST_ENTRY=str(DAEMON_BUNDLE.resolve(strict=True)),
                           SAND_TREE_SITTER_NODE_DEPS=str(pathlib.Path("desktop/dist/node-deps").resolve(strict=True)))
        for key in ("ELECTRON_RUN_AS_NODE", "TREE_SITTER_PREBUILD", "TREE_SITTER_BASH_PREBUILD",
                    "PREBUILDS_ONLY", "NODE_OPTIONS", "NODE_PATH", "MAHAYANA_APP_HOST_BIN",
                    "SAND_SENTRY_ENVIRONMENT", "SAND_SENTRY_RELEASE"):
            environment.pop(key, None)
        receipt["test_environment"] = {
            "FABUSHI_COORDINATOR_TEST_BIN": BINARY.as_posix(),
            "FABUSHI_LOCAL_EXEC_HOST_TEST_BIN": HOST_BINARY.as_posix(),
            "FABUSHI_LOCAL_EXEC_DAEMON_TEST_ENTRY": DAEMON_BUNDLE.as_posix(),
            "SAND_TREE_SITTER_NODE_DEPS": "desktop/dist/node-deps",
            "cleared": ["ELECTRON_RUN_AS_NODE", "TREE_SITTER_PREBUILD", "TREE_SITTER_BASH_PREBUILD",
                        "PREBUILDS_ONLY", "NODE_OPTIONS", "NODE_PATH", "MAHAYANA_APP_HOST_BIN",
                        "SAND_SENTRY_ENVIRONMENT", "SAND_SENTRY_RELEASE"],
        }
        for label, command, timeout, names in behavior_cases:
            receipt["active_case"] = label
            save(receipt)
            result = run_command(label, command, timeout, environment=environment,
                                 assessor=lambda text, code, capture, names=names:
                                     tap_assessment(text, code, capture, names))
            receipt["executions"].append(result)
            save(receipt)
        receipt["inputs_after"] = source_bindings(inputs)
        receipt["host_sources_after"] = source_bindings(host_paths)
        receipt["producer_after"] = {str(path): digest(path) for path in producer_paths}
        receipt["bundled_inputs_after"] = all_bundle_inputs()
        receipt["staged_inputs_after"] = staged_inputs()
        receipt["inputs_unchanged"] = receipt["inputs_before"] == receipt["inputs_after"]
        receipt["host_sources_unchanged"] = receipt["host_sources_before"] == receipt["host_sources_after"]
        receipt["producer_unchanged"] = receipt["producer_before"] == receipt["producer_after"]
        receipt["bundled_inputs_unchanged"] = receipt["bundled_inputs_before"] == receipt["bundled_inputs_after"]
        receipt["staged_inputs_unchanged"] = receipt["staged_inputs_before"] == receipt["staged_inputs_after"]
        accepted = (receipt["cleanup_qualification"]["accepted"]
                    and len(receipt["executions"]) == len(planned) + len(behavior_cases)
                    and all(case["oracle_accepted"] for case in receipt["executions"])
                    and receipt["inputs_unchanged"] and receipt["host_sources_unchanged"]
                    and receipt["producer_unchanged"] and receipt["bundled_inputs_unchanged"]
                    and receipt["staged_inputs_unchanged"]
                    and receipt["native_build_retention_qualification"]["accepted"])
        receipt["result"] = "passed" if accepted else "failed-or-incomplete"
        receipt.pop("active_case", None)
    except (OSError, ValueError, KeyError, TypeError, tarfile.TarError, subprocess.SubprocessError) as error:
        receipt["capture_error"] = str(error)
    finally:
        if "native_build_work" in receipt:
            try:
                receipt["retained_native_build_work"] = retain_native_build_work(receipt["native_build_work"], ROOT)
            except (OSError, ValueError, KeyError, TypeError, tarfile.TarError) as error:
                receipt["native_build_retention_error"] = str(error)
                receipt["result"] = "failed-or-incomplete"
    save(receipt)
    print(json.dumps(receipt, indent=2), flush=True)
    return 0 if receipt["result"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
