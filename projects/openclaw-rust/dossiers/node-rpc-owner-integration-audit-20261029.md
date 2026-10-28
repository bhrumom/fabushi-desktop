# OCRM technical dossier: node RPC owner integration audit

Status: production source wiring is present on this branch; executable verification and real transport/device acceptance remain pending.

Project: OCRM-001  
Audit date: 2026-10-29 (Asia/Shanghai)  
Source branch: `codex/openclaw-rust-full-fusion-20261029`

## Scope and source authority

This audit covers the Fabushi shipping `UnifiedGatewayApi` node RPC handlers and their existing native owners. It is a focused integration slice, not a declaration that the full OpenClaw-to-Fabushi migration is complete.

Relevant current source:

- `source/host/app/src/main.rs`, `UnifiedGatewayApi::call_openclaw_node_register`
- `source/host/app/src/main.rs`, `UnifiedGatewayApi::call_openclaw_node_invoke`
- `source/host/src/extensions/local_exec/gateway_local_exec_sand_box.rs`
- `source/host/tests/openclaw_chat_cron_node_contract.rs`
- `docs/specs/node-rpc-remote-presence.md`
- `projects/openclaw-rust/dossiers/crabbox-node-enrollment-owner-audit-20261029.md`

The upstream Crabbox enrollment audit remains pinned to OpenClaw commit `ad8d12f126d29b0db204187adf5092db80294e70` and tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. That audit explicitly keeps Crabbox bootstrap, artifact, plugin, process and desktop-bridge ownership separate from FDN enrollment and Computer execution.

## Integrated production path

The previously prepared candidate changes were reviewed and integrated into this branch as:

- `268b7ecd4` — route `node.register` through the existing device-pairing owner;
- `d278787ef` — route local `node.invoke` `system.run` through the existing local-exec permission and sandbox owners.

The register path now requires a non-empty node identity, preserves the approved local-node fast path, and delegates remote registration to `self.device_pairing.handle_pair_request`. It injects the device identity only when absent and returns the pairing result rather than fabricating a worker registration response. The path does not create a second pairing store or a Crabbox runtime.

The invoke path now requires a node identity, command, and idempotency key, fences remote calls on the existing pairing status, accepts only `system.run`, and delegates execution to `self.local_exec` with `GatewayLocalToolScope` and `self.local_tool_permission`. It returns the native exit code, stdout, stderr, working directory, abort, outcome-unknown and truncation fields. The former fixture-style success response is not present in the shipping handler.

The existing `production_remote_computer_target_service` was also inspected before deciding the remote branch. Its private transport is lease-, generation-, identity- and sensitive-result-fenced, but its native executor deliberately accepts the product `Computer` contract; it does not expose a generic `system.run` node transport. The current remote `node.invoke` error is therefore an explicit transport boundary, not a hidden fallback to local execution. Extending it to remote shell execution requires a separately specified native capability and owner decision.

The remote presence projection is now fail-closed in the current slice: pairing approval makes a remote device visible and `paired`, but `node.list`/`node.status` report `connected=false` and `active=false` and advertise no executable commands until a live node-transport owner supplies those facts. The local `sand-local-node` remains connected and active. This removes the prior contradiction in which `node.list`/`node.status` claimed a live remote node while `node.invoke` reported that transport was unavailable.

This preserves the current ownership boundary:

| Behavior | Owner | Boundary |
| --- | --- | --- |
| Device identity, pairing approval and generation fencing | `DevicePairingOwner` | No new node-registration state or parallel pairing authority |
| Local command authorization and execution | `HostLocalToolPermissionExtension` + `HostLocalExecExtension` | No direct shell bypass and no second executor |
| Remote Computer authorization/result fencing | `remote_computer_target.rs` | Not replaced by node RPC glue |
| Crabbox provider/artifact/node lifecycle | unresolved owner from the Crabbox audit | Not silently folded into node RPC handlers |

## Verification evidence

The focused contract test now includes a source-bound oracle that reads the shipping `main.rs` and asserts:

- `node.register` calls `self.device_pairing.handle_pair_request` and does not use the old synthetic pairing-generation response;
- `node.invoke` constructs `GatewayLocalToolScope`, calls `self.local_exec.execute_shell`, requires `params.command`, rejects unsupported commands, and does not return the old literal `stdout: ok` / `exitCode: 0` response.

This is a static wiring oracle only. It has not been executed locally because repository policy requires executable OCRM verification in GitHub Actions.

The existing pure contract coverage still provides request-shape and security oracles for node register/list/status/invoke, including missing identity, unsupported command, unconnected remote node, idempotency and bounded-result behavior. Those pure tests do not by themselves prove packaged runtime or real device/provider behavior.

## Open gates

- Run the focused contract and relevant host/runtime checks in GitHub Actions against the exact accepted branch revision.
- Confirm the completed Rust host workflow and retain its exact head, job outcomes and logs; an in-progress or unrelated SHA is not acceptance evidence.
- Verify real private transport/device behavior for remote nodes, including approval, reconnect, generation fencing, timeout and outcome-unknown handling.
- Specify and implement the missing remote `system.run` capability only after an owner/contract decision; do not reinterpret the existing Computer-only private transport as node shell execution.
- Add a live node-transport owner before changing remote presence to connected/active or advertising executable remote commands.
- Complete the remaining Crabbox provider, artifact, process, platform bridge and reverse-caller source closure before creating a new owner or claiming OCRM completion.
- Keep all global OCRM completion flags false until the full RTM and AC01–AC24 have direct positive evidence.

## Conclusion

This slice fixes the identified production wiring gap for node registration and local node invocation while reusing existing Fabushi owners. It is an implementation milestone, not full equivalence acceptance. The remaining claims are deliberately left pending until Actions and real-provider/device evidence are available.
