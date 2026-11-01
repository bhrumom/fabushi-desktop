# Complete read — fixed-U canary Bot API runtime entrypoint

## Identity

- Upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Complete recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, 7,194 entries, non-truncated.
- Source: `.github/telegram-bot-api/entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e`.
- Recursive-tree index: 115 zero-based.
- Git mode/type/size: `100755` / blob / 322 bytes.
- Text shape: 322 ASCII code points, 11 LF, zero CR, trailing newline.

## Exact shell/runtime contract

The script is exactly a POSIX `#!/bin/sh` entrypoint. After comments it runs `set -e` and then one `exec /usr/local/bin/telegram-bot-api` with these arguments in order:

1. `--http-port=8081`
2. `--dir=/data`
3. `--temp-dir=/data/temp`
4. `${TELEGRAM_LOCAL:+--local}`
5. `"$@"`

The executable is an absolute path, so there is no PATH lookup. The successful `exec` replaces the shell; there is no shell supervisor, background process, retry/backoff loop, daemonization, signal proxy, or fallback process. The Bot API process becomes the container lifecycle owner. If `exec` itself fails, `set -e` makes that shell startup fail; if the binary starts and later exits, its status is the container status.

## Environment and argv

The source comments state that `TELEGRAM_API_ID` and `TELEGRAM_API_HASH` are read by the binary. The script never expands, validates, logs, rewrites, or exports those names; they are simply inherited environment.

`TELEGRAM_LOCAL` is interpreted by POSIX `:+` expansion. Unset or empty yields no argument. Any non-empty value yields exactly the literal `--local`; this includes `TELEGRAM_LOCAL=0`. The fixed-U consumer sets it to `1`.

Quoted `"$@"` preserves each caller-supplied argv element and avoids a second shell parse. The source contains no `eval`, `sh -c`, command substitution of caller input, glob expansion of caller argv, or allowlist/semantic validation. Extra arguments are therefore trusted deployment/operator authority. This shell does not establish whether duplicate later options override earlier fixed options; that belongs to the external binary parser and must be proven separately if relied upon.

## Network and data paths

The shell fixes only the server HTTP port number, 8081. It supplies no bind-address, TLS, proxy, firewall, network-namespace, Bot-token, or authorization flags. Exact Dockerfile composition exposes 8081 and fixed-U canary jobs map it and use `BOT_API=http://localhost:8081`, but that is consumer/container behavior rather than a shell-enforced loopback or transport-security guarantee.

Persistent working path is `/data`; temp path is `/data/temp`. The Dockerfile creates/chowns `/data` for non-root user `botapi` UID 999 and sets `WORKDIR /data`; it does not create `/data/temp` or declare a volume. The entrypoint itself performs no mkdir/chown/chmod. Temp-directory creation and data-layout validity are delegated to the binary. The canary service declares no volume, so this source chain provides no durable Fabushi persistence contract.

## Direct callers and consumers

The only direct repository source reference found for `entrypoint.sh` is exact `.github/telegram-bot-api/Dockerfile@a361602441040028a3332c3732c4902d9c880122`, which performs `COPY entrypoint.sh /usr/local/bin/entrypoint.sh` and declares `ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]`. It also composes non-root UID 999, `/data`, and port 8081.

Exact build caller `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9` builds context `.github/telegram-bot-api` from a validated full external Bot API commit and pushes the image. Exact image consumer `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947` requires a GHCR sha256 digest and runs the service in win64/mac/linux publish jobs with runtime `CANARY_API_ID`, `CANARY_API_HASH`, `TELEGRAM_LOCAL=1`, and port 8081 mapping.

Exact downstream `Telegram/build/canary_publish.sh@4eb2d790063bb8f11efc7ffa06977fe1ef79a270` is the readiness/publication consumer. It polls `getMe` up to 30 times with two-second sleeps and fails if readiness never arrives, then sends update/portable files and edits metadata through the local Bot API. The entrypoint itself owns none of this readiness or publication recovery.

## Failure and recovery

Hard/non-passing cases include: missing or non-executable `/usr/local/bin/telegram-bot-api`; failed exec; unusable `/data` or `/data/temp`; invalid/missing runtime API inputs rejected by the binary; binary startup/exit; trusted appended argv that changes behavior incompatibly; process startup without downstream readiness; stale entrypoint/Dockerfile/workflow/image lineage. Recovery is a fresh exact source/context/image chain and successful later Stage-B qualification, not alternate paths, PATH search, root/privileged fallback, third-party image substitution, hidden retry, or acceptance based only on process existence.

## Security boundary

Positive source properties: absolute binary path; no `eval`; quoted `"$@"`; no secret printing by the shell; exact fixed port/data paths; non-root runtime supplied by the Dockerfile; digest-pinned image supplied by the consumer workflow. Negative/non-claims: no credential validation, secret scrubber, TLS, bind restriction, firewall, auth policy, argv allowlist, privilege drop inside the script, or persistence isolation. API credentials stay runtime-only and extra argv must remain a trusted operator boundary.

## Existing-owner-first disposition

Target owner is the existing Fabushi repository build/container/toolchain provenance + dependency-advisory/supply-chain quality + repository process/security governance + shared quality/evidence/release gates. No target entrypoint, Dockerfile, Telegram Bot API runtime, sidecar/provider, second updater/release service, Agent/Coordinator/Host/Runner product component, platform-core service, or second security/evidence truth is added.

Target contract surfaces:
- `AGENTS.md` — Repository canary Bot API runtime-entrypoint authority.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` — Repository canary Bot API runtime-entrypoint qualification.
- `projects/fabushi-communication-platform/quality/evidence-contract.md` — Repository canary Bot API runtime-entrypoint evidence.

## Authored-only verification inventory

Contract cases: `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-SHELL-001`, `...-FIXED-ARGV-001`, `...-LOCAL-MODE-001`, `...-ENV-INHERITANCE-001`, `...-ARGV-FORWARDING-001`, `...-PID1-EXEC-001`, `...-NETWORK-001`, `...-DATA-PATHS-001`, `...-SOURCE-LINEAGE-001`, `...-CONSUMER-001`.

Integration: `INT-TDRP-CANARY-BOT-API-ENTRYPOINT-REPOSITORY-PROCESS-001`.

Fault/recovery: `FAULT-TDRP-CANARY-BOT-API-ENTRYPOINT-EXEC-FAILURE-001`, `...-DATA-PATH-001`, `...-BINARY-STARTUP-001`, `...-READINESS-001`.

Security: `SEC-TDRP-CANARY-BOT-API-ENTRYPOINT-NO-EVAL-001`, `...-SECRET-BOUNDARY-001`, `...-ARGV-TRUST-BOUNDARY-001`, `...-NONROOT-COMPOSITION-001`.

All are authored only. Stage A performs no shell/container execution, Docker/image build, service start, Bot API request, workflow run/query, validator, test/E2E, package/sign/release/deploy/install, or runtime acceptance. Status remains `validation_status=not-run`, `accepted=false`.
