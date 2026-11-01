# Round 191 evidence — repository canary Bot API runtime entrypoint

Task round 160. Spec-first commit `8b36ad8f578a175f068f35fd1646ad586c1ef16a`. Fixed-U source `.github/telegram-bot-api/entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e`, recursive-tree index 115 zero-based, mode 100755, 322 Git bytes/code points, 11 LF, 0 CR, trailing newline.

Complete semantic read confirms POSIX `/bin/sh`, `set -e`, absolute-path `exec`, fixed port/data/temp argv, non-empty `TELEGRAM_LOCAL` presence semantics, inherited binary-owned API ID/hash, and exact quoted argv forwarding. Successful exec removes the shell supervisor and makes the Bot API binary PID1. No shell-owned retry/backoff/readiness, bind-address/TLS/auth, secret scrubber, argv allowlist, directory creation or privilege drop is invented.

Exact direct caller is `.github/telegram-bot-api/Dockerfile@a361602441040028a3332c3732c4902d9c880122`; exact build caller is `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9`; exact digest-pinned image consumer is `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`; exact downstream readiness/publication consumer is `Telegram/build/canary_publish.sh@4eb2d790063bb8f11efc7ffa06977fe1ef79a270`. Repository source search found no other direct `entrypoint.sh` reference.

Responsibility `TDRP-R9-REPOSITORY-CANARY-BOT-API-ENTRYPOINT-001` maps existing-owner-first to repository build/container/toolchain provenance + supply-chain/process/security + shared quality/evidence/release gates. Ten contract, one integration, four fault-recovery and four security cases are authored, not run; `accepted=false`. One source entry closes: unknown 16118->16117, unread 16091->16090, responsibilities 160->161, omitted 0, unmapped 0. Next independent source entry is `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9` at index 117.

No Actions/check/status/workflow-run query or trigger, shell/container execution, Docker/image build, registry operation, service start, Bot API call, validator, test/E2E, package, signing, release, deployment, installation or runtime acceptance was executed.
