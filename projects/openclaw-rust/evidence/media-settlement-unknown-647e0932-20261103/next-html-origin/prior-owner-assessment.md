# Generated HTML persisted provenance — next-owner source assessment

## Decision and exact scope

**The generated-origin persistence responsibility remains open at the exact target.** The current product already implements explicit HTML byte validation, generic attachment publication, Session media custody, and fresh-owner recovery. Those behaviors do not preserve a trusted generated-HTML origin across staging and later reads. The finding is based on the shipping materializer, its message/custody representations, the actual loader, durable receipt/schema and fixed-upstream producer/consumer, not solely on an unused helper or a ledger status.

This is preparation for the root's next production batch, not acceptance, implementation permission, a new canonical spec, or a review of MSU R1. Actual configured model and inherited **xhigh** retained because the scope crosses provenance authority, persistence, custody, replay and recovery.

| Subject | Exact identity |
| --- | --- |
| Target repository/base | `bhrumom/fabushi-desktop@d70f86f33b5e3c67545954b270373e1548809f4b` |
| Target tree | `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c` |
| Fixed upstream | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` |
| Upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

All production sources were obtained through exact-ref or exact-blob GitHub reads. The supplied target tree was used only for identity accounting. No local repository working tree or immutable local Git source was needed. Current captured blobs match that tree; byte identities and SHA-256 hashes are in `media-html-provenance-next-inputs/source-identities.json`. Capturing a whole file is not a claim of whole-file semantic review; assessed ranges are recorded below.

## What the current production chain actually does

### Shipping SendMessage producer and persistence

The real first-party tool constructs an attachment from `SendMessageInput.url`, calls the sink resolver, and records only its returned URL and filename, plus ordinary alt/dimension fields (R2 L281–298). `ResolvedAttachmentSource` itself has only `url` and `file_name` (R2 L35–39).

The App installs `ProductionSendMessageSink` for a captured Agent/session (R1 L11568–11590). Its resolver checks Session admission, then calls `resolve_production_attachment_source` (R1 L880–905). That producer reads a local file or the existing Box download fallback and persists bytes under the existing Agent media directory (R1 L639–705; R18 L242–267). HTML is ordinary non-image attachment data on this path. `persist_runner_media_bytes` uses the Session admitted filesystem mutation boundary before `persist_agent_media_bytes` (R1 L618–637). The latter creates a UUID-named Agent asset/attachment; it has no generated-origin parameter or durable marker write (R3 L236–255).

Session custody is real and must be reused. It captures the media work ID, lifecycle revision and materialized paths; the App sends through the existing transcript settlement facade, then publishes the transcript event and optional channel delivery (R1 L906–1144). Those identities establish ownership/settlement. They do not attest HTML origin: `SessionWorkAdmissionRecord` stores paths, while the outgoing settlement receipt stores paths and transcript entry IDs without content hash, size, kind/version or generated-source evidence (R9 L555–588, L6384–6406). The existing first-open recovery uses that receipt/fence to retain or clean media; it does not reconstruct origin from bytes or message history (R8 L1757–1820).

### Attachments/media-store and subsequent reads

There are **two existing storage contexts**, which must not be conflated:

| Context | Current owner and output | Provenance implication |
| --- | --- | --- |
| Agent message media | App → Session admission → `attachment_paths`; paths under Agent assets/attachments, with existing message custody | A successful send or recovery receipt is not a generated-HTML grant |
| Generic media store | Attachments `saveMediaBuffer` → `media_store::save_media_buffer`; global `<sand-root>/media/<subdir>`, including `outbound` | `SavedMedia` has ID/path/size/content type, with no generated-origin field or marker operation |

The actual generic service dispatch accepts buffer bytes, filename, MIME and subdirectory and calls the store (R4 L1856–1882). Upload likewise goes through content-addressed attachment ingestion (R4 L1406–1425, L290–319). Neither route is evidence of generated origin. Ordinary `readMediaBuffer` returns bytes/path, and ordinary attachment preview is a separate text/binary preview contract (R4 L597–631, L1883–1906; R5 L744–770).

Explicit HTML is already implemented in the actual loader/service chain. `AttachmentsService::dispatch_gateway("loadWebMedia")` forwards `htmlPublication` to `web_media::load_web_media` (R4 L2006–2026). After the existing local-path/root/size checks and bounded read, exact `"explicit"` plus HTML/HTM extension selects byte sniffing with no filename hint and the existing HTML policy. Success returns original bytes as `text/html`/`document` with generated trust absent (R6 L388–465). This is implemented behavior and should not be duplicated or upgraded into generated origin.

A material consumption gap remains: `WebMediaOptions.host_read_capability` exists (R6 L75), but the actual local-loader body never branches on it. Outside explicit mode it follows ordinary image probing/result construction and returns `trusted_generated_html_source: None` (R6 L468–489). There is no marker lookup in this path. The direct assertion service also supplies `trusted_generated_html_path: None` and `explicit_html_publication: false` (R4 L2057–2077).

The App's ordinary gateway dispatcher reaches the Attachments service (R1 L15089–15104), but this review does not establish `loadWebMedia` as a registered external user command or as the shipping SendMessage consumer. The SendMessage path above uses its own materializer. Therefore this is a concrete missing loader/product integration contract; it is not evidence that every current generic HTML send is an unauthorized operation.

## Fixed U establishes a real production responsibility

Fixed U has an actual producer chain:

1. Product message actions and reply-media preparation call `resolveOutboundAttachmentFromUrl` (U9 L529–583; U10 L250–320).
2. The loader determines origin itself. HTML host reads either have the secure temp-root classification or read a persisted outbound marker. A regular, non-symlink, single-link file and canonical scope are required; outbound takes precedence even if it sits beneath the trusted temp root (U2 L188–224).
3. The content policy checks the actual bytes and HTML shape. Outbound trust requires exact hash and size. Explicit publication authorizes content without becoming generated origin (U3 L118–143; U2 L664–751).
4. URL staging saves the captured bytes and calls the marker writer only when the loader returned `trustedGeneratedHtmlSource`. Ordinary buffer staging does not mark origin (U1 L17–43, L46–64).
5. Marker publication captures its state owner, computes hash/size before the filesystem await, confines the destination to outbound, and submits an owned upsert (U2 L227–253). Read/list/upsert/prune use the existing state DB worker, not caller SQL (U4–U6).
6. Later reads require a marker and current bytes. Cleanup invokes provenance pruning; pruning deletes only the full row snapshot still matching the filesystem observation, preserving a newer marker (U6 L60–87; U8 L290–307).

This is business runtime behavior, not HTML source-report generation or source-accounting tooling. The worker/contract files participate directly in runtime SQL and trust decisions. Repository source manifests, reading records and generated audit reports are evidence artifacts, not generated-document authority.

The exact private temp-root boundary matters. U11 selects a secure/private root via `@openclaw/fs-safe/secure-temp-root` and checks POSIX ancestor safety. The current Rust local-root list includes ordinary `std::env::temp_dir()` (R13 L61–85). An allowed read root is not equivalent to that generated-source boundary. No safe implementation decision follows from setting `HostReadHtmlTrust::TempRoot` for every file under the current generic temp/workspace allowlist.

## Concrete missing contract and bounded counterexample

**GHP-NEXT-01 — runtime generated-origin staging/reload is incomplete.**

A source-derived direct-loader counterexample is concrete enough to become an Actions test:

1. Save valid HTML bytes through the real generic media store into an owned `outbound` directory, without a trusted-origin producer.
2. Call the actual `load_web_media` on that path with valid local roots, a sufficient byte cap, `host_read_capability=true`, `optimize_images=false`, and no explicit-publication mode.
3. Current code returns the ordinary result: original bytes, image kind, no generated trust. The flag is not consulted. Fixed U's host-read contract refuses an unmarked outbound HTML file, even under a trusted temp ancestor.

This is an unexecuted source assessment, not a reproduced test. It should be paired with the real generated positive after its authority is defined.

The shipping counterpart is information loss: generated report bytes and identical generic uploaded/local bytes enter the same URL/filename-only materializer and persist without an origin marker. A later reader or resend cannot distinguish them using the existing Session fence or receipt. Supplying a fabricated `HostReadHtmlTrust::Outbound` to the policy, or constructing a row in a unit test, cannot repair that production gap. The existing provenance admission test does precisely that narrower policy exercise (R14 L100–155).

A new positive cannot honestly start from a caller boolean or an arbitrary HTML file labelled “generated.” **The exact legitimate Rust producer/source grant remains a design dependency.** The inspected production image generator is image-specific (R17); it is not an HTML producer. No trusted HTML authorizer was identified in the actual SendMessage chain reviewed here. This conclusion is bounded to the inspected chain and is not a claim that every unrelated repository subsystem has been exhaustively searched.

## Existing owner allocation and narrow next behavior contract

The current responsibility remains `OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001`, blocker `BL-MEDIA-HTML-PROVENANCE-CHAT-WIRING-001`. The current ledger row identifies only policy types/validation and explicitly leaves the producer, durable state and consumers open. Its `implementation=implemented` refers to that existing helper scope, not an implemented end-to-end provenance store. The adjacent host-read consumer blocker remains distinct. Module-map M23 assigns media/provenance to existing Attachments; M07 retains Session/Transcript history ownership.

For the **Agent/session chat batch**, the suitable allocation is:

- **Attachments** owns content validation and interpretation of generated-source evidence.
- **Existing Session/SandAgentDb** owns durable state and mutation admission for the Agent-owned files already carried by its media fence and settlement receipt.
- **App/Runner composition** supplies the captured producer/session and consumes the result through the existing sink.
- **Existing media lifecycle/recovery** owns cleanup. No parallel provenance runtime, session DB, cache-as-authority or cleanup worker is needed.

That allocation is a next-spec recommendation grounded in current ownership, not an already implemented marker API. Global unscoped media-store files need an explicit owner mapping if included; they must not silently be rebound to whichever Agent is active. A narrowly scoped chat implementation must leave global-store provenance equivalence open until that mapping and consumer exist.

The root's durable contract should define these behaviors before implementation:

1. **Origin admission:** name one real trusted producer and the immutable identity it passes to Attachments. The grant must come from an existing owner, be scoped to the captured Agent/session/generation and exact source object, and remain separate from ordinary file-read/delivery authority. Paths, extensions, MIME, user/tool JSON, uploads, explicit publication and successful sends do not mint it.
2. **Staging and durable identity:** capture and validate the actual buffer once; preserve origin when that exact buffer is materialized into the existing owned location. Persist canonical scope/path, marker kind/version, hash, byte size and a mutation identity sufficient for conditional replacement/removal. Use the current Session writer for chat media. Choose and document the acceptance boundary with transcript/media settlement; adding an independent marker commit after a reported successful send would leave an unowned failure window.
3. **Later read/resend:** the real consumer must resolve marker state through the captured owner, recheck current file kind/scope and actual hash/size, then invoke the existing HTML byte policy. Missing, malformed, stale or unavailable provenance narrows trust. Explicit mode stays explicit and trust-free. Preserve ordinary non-host-read media behavior.
4. **Failure and recovery:** known precommit refusal, unknown commit outcome, cancellation/revocation, owner closure and fresh-owner recovery need distinct dispositions. Do not replay an unknown marker mutation, invent success, delete possibly committed bytes, or move to a successor owner. Reuse the finalized MSU settlement/error/custody contract after root integration; this task has not reviewed that frozen candidate.
5. **Cleanup:** filesystem inspection stays in the existing media owner; the admitted DB owner conditionally removes the exact observed row snapshot. Missing/invalid/link cases may be selected; permission/transient inspection failure retains the row without authorizing a read. No unconditional delete by pathname may erase a concurrently replaced marker.
6. **Legacy and scope:** old files remain ordinary attachments until a real source grant proves origin. No blanket backfill from HTML shape, content hash alone, existing transcript entries or “already sent” status. Preserve HHP-v2 byte policy, CAP publication, LFA admission and ordinary upload/image controls.

## Required causal oracles in existing fixtures

All execution remains Actions-only. Authored or passing helper tests cannot substitute for the real producer/owner/consumer.

| Oracle | Required observation and existing fixture anchor |
| --- | --- |
| Trusted publish → reload → resend | A real owner-generated grant produces a stored marker for the bytes actually materialized; the shipping App sink commits the message; drop every owner, reopen twice, and read/resend through the actual consumer. Assert exact bytes, marker identity, document MIME/kind and retained origin. Reuse R16 `LocalMediaAppFixture`, its actual sink observer (L71–158), construction (L181–241) and `reopen` (L291–304). |
| Ordinary/explicit controls | Identical generic uploaded HTML and explicit HTML remain untrusted; unmarked outbound HTML fails when host-read mode is selected. The existing explicit byte/service test and omitted/unknown-mode test in R15 retain their current assertions. Ordinary non-host-read behavior remains separately tested. |
| Tamper/scope negatives | Same-size changed bytes, changed length, renamed/rebound file, wrong kind/version, link/multilink and another Agent/session cannot reuse the marker. Prove a legitimate positive before tampering, then repair with an authorized new publication and prove recovery. |
| Captured source/owner lifetime | Pause at an existing owned boundary after capture; change source bytes, session generation or owner routing. Assert that no successor store/Agent gets a marker or read grant. Do not use a global fault flag or a synthetic marker-only producer. |
| Real persistence outcome | Exercise confirmed rollback and actual commit-with-lost-ack/unknown using the existing finalized DB fault boundary. Observe actual durable marker/message/file state, no duplicate publication/replay/success, and later recovery. A mocked error from the policy is not SQL evidence. |
| Conditional prune | Start from a genuinely published marker. Remove/expire the file, inspect a stale snapshot, publish a newer marker before deletion, and prove the newer row survives. For the actual stale-row deletion, recreate identical bytes at the old path and prove origin does not resurrect. Repeat after fresh owner reopen. |
| Inspection refusal | Real missing and invalid-file controls remove only their matching markers; actual permission/transient inspection error retains the complete row. A retained row still cannot grant trust when the read cannot validate bytes. Reuse R14's existing filesystem selector controls without claiming those alone test DB mutation. |

The baseline App fixture currently asserts an empty pending map (R16 L278–284). Root's accepted MSU terminal semantics may change that fixture; adapt to the finalized active-custody assertion rather than copying this baseline assertion into the next batch. No active work-tree fixture was read.

## Remaining implementation dependencies and non-claims

The source work establishes a necessary runtime gap and an existing owner path. It does **not** settle these remaining inputs:

- The authoritative HTML producer/source-grant boundary and its supported local/Box path. This is required before minting any marker.
- Whether the first batch covers only Session-owned chat artifacts or also global media-store outbound files, with exact owner/routing for the latter.
- The finalized MSU settlement API, typed unknown disposition and fault-port composition. No new MSU R1 report or candidate source was read.
- Platform regular-file/link/root identity details and the actual pinned secure-temp dependency contract if the upstream temp-root model is chosen. U11 was read, but its fs-safe implementation was not re-reviewed here.
- Exact actions, input/name accounting and independent review for the new candidate. No current test execution or packaged/provider/channel acceptance is inferred.

Do not recreate the existing explicit HTML validator or content publisher merely to fill the ledger. The next coherent unit is one real trusted generated-document publication and later consumption path, with durable identity and negative controls.

## Source references and literal read scope

The following are immutable captured inputs, with original-source line numbers. Whole-file text searches were used to locate callers/fields; only the listed ranges are claimed as semantic reads. Other fetched documents/helper files remain context or discovery-only.

| ID | Path | Exact Git blob | Read ranges | Role |
| --- | --- | --- | --- | --- |
| R1 | [source/host/app/src/main.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/app/src/main.rs) | `4831ae8cf002ce64e6409f2d695b48aaa1914c45` | 618–769; 880–1145; 11549–11602; 15089–15124 | Shipping materializer, custody/send consumer, constructor, Attachments dispatch |
| R2 | [source/host/src/runner/tools/send_message_tool.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/runner/tools/send_message_tool.rs) | `9e12d7d2b457715f0fac168e5e4f262e6ef15985` | 35–96; 247–349; 384–440 | URL/filename carrier and actual first-party attachment construction |
| R3 | [source/host/src/attachment_paths.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/attachment_paths.rs) | `c66a2d1ce8dfb70532a16857a0fe02d31788ca62` | 1–258 | Existing custody, Agent media roots, buffer staging and byte persistence |
| R4 | [source/host/src/extensions/attachments/attachments_service.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/attachments/attachments_service.rs) | `75a49490ee80e7416df66df2431d2f3013f1b17c` | 241–319; 597–631; 1329–1455; 1856–1925; 2006–2077 | Upload, ordinary preview, global media store and explicit HTML dispatch |
| R5 | [source/host/src/extensions/attachments/media_store.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/attachments/media_store.rs) | `8ebfaf9056b9d8c516cc794faacaf6351ed7dfd1` | 1–143; 391–522; 707–789; 904–956 | Store metadata, byte publication/read/delete and lifecycle entry |
| R6 | [source/host/src/extensions/attachments/web_media.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/attachments/web_media.rs) | `bf7a38dfa100fadb6c0cc15b6bf699884c67299b` | 1–96; 327–491 | Actual local loader and HTML result |
| R7 | [source/host/src/extensions/attachments/host_read_media_policy.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/attachments/host_read_media_policy.rs) | `9087b941ce253ac261a453df3d8b0054e64804e2` | 1–200; 230–447 | Content policy, trust types, marker validation and stale selection |
| R8 | [source/host/src/extensions/session/production.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/session/production.rs) | `f585fae13720e08f34323bed82759253590a3753` | 1757–1824; 2285–2336; 2681–2811 | Existing recovery, admitted filesystem boundary and media fence |
| R9 | [source/host/src/extensions/session/agent_db.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/session/agent_db.rs) | `78740768f7f1b0307dfd6756c4d50c368a91b721` | 555–588; 4660–4705; 4865–4919; 6384–6406 | Admission and actual settlement/receipt shapes; file text searched for HTML/provenance names |
| R10 | [source/host/src/extensions/session/agent_db_schema.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/session/agent_db_schema.rs) | `279583c0e0ccb37a7188bfa29b5467f8a1e24b3e` | 1–203 | Existing schema and SQL constants |
| R11 | [source/host/src/extensions/attachments/load_options.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/attachments/load_options.rs) | `4427dbc7b214cabdcbbb7b181c2c5338b6198233` | 1–190 | Current read-capability option construction |
| R12 | [source/host/src/extensions/attachments/read_capability.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/attachments/read_capability.rs) | `e75be65d0ad78318adad66c3e8f901f92c1d2c6b` | 1–233 | Current agent/read-policy input shapes |
| R13 | [source/host/src/extensions/attachments/local_roots.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/extensions/attachments/local_roots.rs) | `02d06b110886deebd256addd20282b6f5f3c1651` | 1–246 | Allowed roots; generic OS temp is not a generated-origin grant |
| R14 | [source/host/tests/openclaw_html_provenance_contract.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/tests/openclaw_html_provenance_contract.rs) | `64f7c72e44d904a4ba84c524577c24688c80c278` | 100–155; test declarations located elsewhere | Existing manually supplied trust oracle |
| R15 | [source/host/tests/openclaw_web_media_contract.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/tests/openclaw_web_media_contract.rs) | `315cda26d1e62b0616881857d6f9047fc883abd1` | 567–638; 728–764; adjacent explicit test declarations located | Actual explicit loader/service controls |
| R16 | [source/host/app/src/tests/local_media_file_url_tests.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/app/src/tests/local_media_file_url_tests.rs) | `5682e951f1eac8f107efd3a3948b0748870912bd` | 1–322; 459–524 | Actual App/Session fixture and fresh-owner reopen |
| R17 | [source/host/src/runner/host_generate_image_dependencies.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/runner/host_generate_image_dependencies.rs) | `f4047f4dc4f944a37728a13ccb03ecc03fb66245` | 1–49 | Actual image generation bridge, not an HTML origin producer |
| R18 | [source/host/src/runner/tools/send_message_encoding.rs](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/current/source/host/src/runner/tools/send_message_encoding.rs) | `22e2700f4ea2f894b92f3233d0a399a296f6fa5b` | 151–267 | Existing Box download/persist fallback |
| U1 | [src/media/outbound-attachment.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/outbound-attachment.ts) | `d2c00c2905323763fb58cb61b4e3aeb5bf37a250` | 1–65 | Real URL staging producer versus ordinary buffer staging |
| U2 | [src/media/web-media.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/web-media.ts) | `f48bca504f6fb77c3d094864eb656cb2b1bf25e2` | 1–110; 141–259; 530–600; 650–756 | Generated-source classification, marker publication and later read |
| U3 | [src/media/host-read-media-policy.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/host-read-media-policy.ts) | `dfe3731a67d3f2d320c0bb67c3aa747cabdee909` | 1–234 | Byte/content authorization policy |
| U4 | [src/media/generated-html-provenance.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/generated-html-provenance.ts) | `d00a635b7ee764126236a7f7c4c32969acf3f893` | 1–97 | Captured read/write owner and stale inspection |
| U5 | [src/media/generated-html-provenance.worker-contract.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/generated-html-provenance.worker-contract.ts) | `3d1ad1f65212e1b7dd166e647c426c38fe4283d6` | 1–23 | Persisted row and operation types |
| U6 | [src/media/generated-html-provenance.worker.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/generated-html-provenance.worker.ts) | `0dd1b22b82eb26d4c53e00a8bc8bf9ae62b035fa` | 1–90 | Actual upsert/read/list and conditional prune SQL |
| U7 | [src/media/generated-html-provenance.test.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/generated-html-provenance.test.ts) | `e49c01be45ac77f6275498eeeece0b83feb648a1` | 1–293 | Actual staged reload, authority, receipt/unknown and prune-race oracles |
| U8 | [src/media/store.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/media/store.ts) | `d4342f8f5a4de1834346f131db70d2c6b6e80426` | 290–310; other symbol positions located only | Existing lifecycle calls provenance pruning |
| U9 | [src/infra/outbound/message-action-send.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/infra/outbound/message-action-send.ts) | `2f03e468a9531433af016cb2279ad117924a974c` | 529–590 | Product message action calls outbound staging |
| U10 | [src/auto-reply/reply/reply-media-paths.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/auto-reply/reply/reply-media-paths.ts) | `59f2c3e7542da5f1168f87813f461eae1165839d` | 250–324 | Actual reply-media preparation calls outbound staging |
| U11 | [src/infra/tmp-openclaw-dir.ts](sandbox:/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/upstream/src/infra/tmp-openclaw-dir.ts) | `cf059c7a37029556ba07f40f73563dcea9b39adb` | 1–128 | Private secure temp-root selection and dependency boundary |

Governance/context read: exact root `AGENTS.md` (`e4682748087759b2d4bb2b3a47916b6148e8a6f2`, relevant discovery/review/OCRM sections); SOT (`3748d734b10c9ed5b2d948467fbaa9483f1dd306`, L1–53); OCRM main spec (`69c9667af2328e20a1fba09ebf80c5aab4d5c336`, R04/R05/R08/R13/R14 and §17 L671–720); module-map (`15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3`, M07/M23/M24 and unique-owner rule); complete HHP-v2 (`f0892088bfa3448441f7b92df6344692ac535932`), media-store read-scope (`61ef64b9d225dab1deeb2dd9384a5ae2e6823b62`) and attachment-content publication (`c74551f6b0f05d771f1b324f3c535b82c3121eff`) specs. Relevant current ledger rows were read from blob `1e56a25479e1390cb98c36d130b68f09946dc0bc`; no ledger or source-review state was changed.

Default-branch GitHub code search was used only to discover the upstream outbound caller paths. All source facts cited above were subsequently read at fixed U. No floating search result is used as semantic evidence.

## Write/execution scope

Written only this report and immutable raw source/identity captures under `/workspace/scratch/7d27ea239237/media-html-provenance-next-inputs/`. No production, spec, ledger, workflow, shared candidate or test edits; no local/htch executable validation, source parser, build, test, test listing, repository capture script, commit/push, Actions dispatch/rerun or agent spawning. Local computation was limited to reading the supplied JSON tree and hashing saved source bytes. No failure is claimed reproduced and no acceptance flag is promoted.

