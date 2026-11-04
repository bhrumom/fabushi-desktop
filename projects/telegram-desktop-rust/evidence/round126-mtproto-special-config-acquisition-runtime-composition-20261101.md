# Round 126 — MTProto special-config acquisition → RuntimeClient composition

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 / round 95
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
Stage: A / implementation_status=implemented / validation_status=not-run / accepted=false

## Fixed-U source reread

This round reread the source tree rather than relying on prior chat conclusions.

- SpecialConfigRequest: Telegram/SourceFiles/mtproto/special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88
- ConfigLoader: Telegram/SourceFiles/mtproto/config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194
- Config defaults: Telegram/SourceFiles/mtproto/mtproto_config.cpp@e8fc605ca197d2bc82a9f73da18eeb945967a99f

Confirmed source semantics:
- config-bearing SpecialConfigRequest seeds Google dns.google.com and Mozilla mozilla.cloudflare-dns.com, shuffles that two-attempt group, and outside test mode seeds direct Firestore plus one Firestore camouflage attempt for each DnsDomains entry and shuffles that group separately;
- group ordering remains DoH attempts before Firestore attempts, while exact dispatch remains the existing 800 ms stagger;
- requestFinished exposes a successful HTTP Date header to the global HTTP-time responsibility before finalizing/removing the reply, then consumes the readable response body;
- Google/Mozilla bodies are adapted from TXT Answer data; Firestore uses fields.data.stringValue;
- non-empty normalized bodies go through simple-config decode, expiry/rule/phone-prefix selection, and selected endpoints are emitted without internally fencing the remaining SpecialConfigRequest stagger;
- ConfigLoader deduplicates endpoints against pending and tried collections, selects a random pending endpoint, moves it pending->tried, applies it through a temporary 1000+d request, and continues with remaining candidates; special config changes only dc_options.

## Existing-owner-first implementation

No TelegramRuntime, TelegramProvider, second HTTP product, second DcDirectory, or special-config-only session registry was added.

Platform implementation sequence:
- spec-first composition authorization: bab99fd54de1cf98dd781f335dad666b3fa2b3ab
- telegram-network attempt seeding/shuffle: 2224fd041f99e3153be80c8b667270fbbd6ce3c5
- RuntimeClient acquisition/application composition: 87bb86ae509ca09c46d1b9e26f5177f81bb54830
- source-review endpoint-queue timing correction: 602b67b2c2c4b11f197ec968e8bc964e89af7695
- final platform source-compliance record: 0b24613beea74301bf974dad4e10c56988096c40

Final platform implementation blobs:
- native/telegram-network/src/special_config_acquisition.rs@9c8ef0ee0176a78eaa98266b5f92db1757b5db7c
- native/telegram-network/src/lib.rs@b073f01ab05e692b46afa4fc931cd3e8dd0893cf
- native/telegram-runtime/src/runtime.rs@df8ed7c3cebed632f6591c5809c28753ede9bba0
- native/telegram-runtime/src/lib.rs@9acba52c16de3085a3dbfa61dc3e9f6ba5cdb689
- docs/specs/telegram-special-config-acquisition-application-composition.md@62ec9f061745b1388728c2d60dc061ebaf86e920

Desktop production mirror:
- initial mirror: 19613c2566e0f4969455106f8143707584aeda78
- final source-reviewed mirror: 0c81381ed8b5d3862c9ced13bc7e2320b0e5d510
- the four production blobs and the spec above are byte-identical between platform-core and desktop mirror.

## Production composition

The existing RuntimeClient now exposes telegram.refreshSpecialConfig and the Rust refresh_special_config entrypoint.

The composition is:
1. derive existing account test-mode and fixed-U apv3.stel.com/tapv3.stel.com domain;
2. seed Google/Mozilla and, for production, Firestore attempts in the existing telegram-network owner with source-equivalent group-local shuffle;
3. normalize requests with the existing request_normalization owner;
4. dispatch with existing DirectHttpsRace using SpecialConfig settlement, so one accepted reply does not fence later scheduled attempts;
5. for each finished response, parse/apply a successful HTTP Date before reply finalization/body decoding;
6. adapt the readable body with existing DNS-TXT or Firestore response normalization;
7. decode/select with the existing special-config protocol owner and current HTTP-adjusted time/phone;
8. enqueue endpoints into one source-neutral pending/tried deduplicating queue and begin draining that queue after each decoded response, while independently scheduled HTTP attempts remain live;
9. move the randomly selected endpoint pending->tried before calling the existing apply_resolved_special_config_endpoint;
10. retain the existing temporary 1000+d SessionOwner, per-real-DC auth material, bare help.getConfig, exact 6000 ms owned MTProto attempt, temporary-session teardown, and dc_options-only replacement.

A failed HTTP/decode/endpoint application is recorded and does not silently terminate unrelated remaining candidates.

## Authored contracts — deliberately not executed

telegram-network authored-only contracts cover:
- injected-entropy Google/Mozilla shuffle;
- test-mode suppression of Firestore;
- production DoH-before-Firestore group order;
- attempt-kind request normalization;
- stable zero/one random-index behavior.

RuntimeClient authored-only contracts cover:
- pending/tried endpoint dedupe;
- random-index selection moving an endpoint to tried before application.

Existing direct-carrier, response-normalization, decode/select, temporary-session, bare help.getConfig and dc_options contracts remain the downstream coverage.

No contract/unit test, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, integration/E2E, packaging, deployment, installation, runtime acceptance, GitHub Actions query, commit-status query, or check query was executed.

## Stage-A disposition

Responsibility TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001 is now production-composed and may advance to implementation_status=implemented. Validation remains not-run and accepted=false. Stage-B executable evidence is still mandatory before verified/accepted.

The full migration remains incomplete; this round does not authorize Stage B.
