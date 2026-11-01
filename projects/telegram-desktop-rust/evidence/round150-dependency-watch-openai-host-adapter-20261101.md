# Round 150 — fixed-U OpenAI Dependency Watch host adapter

Task round: 119  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`

Live desktop checkpoint was reread at `46241ddf29ca947ce1dae3fafae56e27ba13bf97`; desktop main at `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` was verified. Fixed-U root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was recursively reread `truncated=false` with 7,194 entries. Before closure the unique ledger was `16202 / unknown=16159 / unread=16132 / omitted=0 / responsibilities=134 / unmapped=0`.

The exact target blob was read completely: 260 characters / 4 lines, fixed-U tree index 58. It defines only OpenAI host metadata: `display_name="Dependency Watch"`, `short_description="Track dependency releases and missing upstream fixes"`, and a default prompt that invokes `$dependency-watch`, audits freshly fetched `origin/dev`, and reports pressing updates before the review-someday list. Repository search found no second in-repository parser or durable advisory state owner. Fixed-U code-health, continue, perform-task and process-inbox adapters use the same `interface/{display_name,short_description,default_prompt}` schema.

Existing-owner-first attaches the blob to `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001`; no new responsibility or owner is introduced. Shipping Electron updater, platform-core runtime, Product Tasks/Automations, Agent/Coordinator/Host/Runner, build-time acquisition inventory as decision authority, and a copied source-specific monitor remain rejected.

Spec-first CAS `832ce8cdbfba0fc24522f2e9bc001600d4fa6b20` extends the existing Test Strategy and Evidence Contract and adds `projects/telegram-desktop-rust/dossiers/dependency-watch-openai-host-adapter-complete-read.md`. Authored but not run: `CONTRACT-TDRP-DEPENDENCY-OPENAI-HOST-METADATA-001` and `INT-TDRP-DEPENDENCY-OPENAI-HOST-INVOCATION-001`.

After closure: `source_entries_total=16202 / unknown=16158 / unread=16131 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`. The next independent fixed-U blob is `.agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
