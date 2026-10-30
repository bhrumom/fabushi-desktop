# node.invoke timeout and cancellation propagation — 2026-10-30

The local `node.invoke system.run` path already admitted an invocation through
the generation/idempotency owner, but its parsed `timeoutMs` was not passed to
the existing local-exec owner. A deadline therefore could not request provider
cancellation before the Gateway call returned.

`GatewayLocalExecManager::execute_shell_with_timeout` now preserves the
existing operation journal and uses the registered operation's cancellation
handle when the node deadline expires. It marks the result aborted only when
the cancellation frame was delivered; if the provider does not produce a
terminal frame, the response remains `outcomeUnknown=true`. The legacy
`execute_shell` path delegates with no deadline, so existing consumers retain
their behavior.

`node.invoke` passes the protocol `timeoutMs` into this owner. No second
process controller, cancellation registry, or remote transport was added.
Evidence is limited to source inspection and `git diff --check`; runtime
provider/device acceptance, local OCRM tests/builds, and Actions remain
pending.
