# Coordinator JSON wire fidelity — CJW-v1

## Scope and authority

Base: `be07ae46b199d712f689fb0b0a67594f327efcad`. Fixed OpenClaw U remains
`1440bb1771372334bea23d7642bb52deb7db1845`; no source or completion state is
promoted. This bounded contract makes the existing unchanged-payload requirement
CECF-4 in `docs/specs/coordinator-gateway-event-channel-fidelity.md` measurable
at the actual Coordinator HTTP reply and SSE decoding boundaries.

The existing `gateway_request_dispatcher::dispatch_http_json_response` parses
the Host response into a `Value`. `gateway_client::stream_http_events` parses
SSE envelopes into a `Value` before the existing event callback. Public
`getAgentTranscriptTail` replies and committed transcript events reach these
owners and the renderer. Supported carrier requests and Host stdout fallback
have additional real parsing entrances; their complete binary-path verification
remains separate. Host inbound channel append and its local Runner handoff do
not universally traverse Coordinator.

Coordinator is a standalone Cargo package with manifest blob
`7205ee428e18bdd4ba82e3f98147698058e07f22` and no committed adjacent lock at this
base. The Host's observed `serde_json 1.0.154` default/std decoding failure at
Actions `41453150181` supplies an independently constructed fixture seed,
not evidence of Coordinator's resolution or runtime behavior. CL-01–05 in
`projects/openclaw-rust/management/tasks/P0-config-settings-authority.md`
remains the authority for observing and freezing actual Coordinator inputs.

## Required behavior and evidence

- **CJW-01 — value contract.** Preserve the supported finite f64 values actually
  encoded by the producer through the public HTTP reply and SSE callback,
  together with the complete object/array/string/integer/boolean/null payload.
  This is fixed-float semantic fidelity. It does not promise arbitrary decimal
  precision, numeric spelling, whitespace, object-key order, or a JavaScript
  negative zero already converted to zero before reaching this wire.
- **CJW-02 — existing owners.** Keep Host Session persistence, Runner execution,
  Coordinator transport/protocol, event-family mapping and thin Electron/UI
  consumers with their existing owners. Preserve method correlation, headers,
  permission, generation, cancellation and error behavior. No schema/protocol
  version change, numeric normalizer or alternative parser is authorized.
- **CJW-03 — deciding baseline.** In the already selected
  `coordinator_contract` target, exercise actual `dispatch_http_json` and
  `stream_http_events` against owned loopback peers. Construct complete
  expected Values directly from fixed float bits; never parse the expected
  value through the decoder under test. Retain synthetic raw response/event
  bytes and exact leaf-bit diagnostics before full equality assertions.
  Include a fractional epoch probe, signed/finite representative values,
  an integral control and nested ordinary JSON. Preserve existing transport,
  bearer/slim-avatar, event and HTTP-error controls.
- **CJW-04 — bounded fixture.** Bound peer acceptance, socket reads/writes and
  synchronization. Normalize accepted sockets to blocking before installing
  timeouts. Release and join the exact owned peer on failure as well as success.
  A barrier may deliver SSE after `on_connected`; a combined socket write does
  not prove initial-buffer branch coverage. Setup/timeout/compile errors are
  non-deciding, never numerical failure evidence.
- **CJW-05 — input provenance.** Keep the manifest, dependencies, features and
  selected Coordinator command unchanged for the initial CL observation.
  The numeric baseline may accompany that first command and its exact raw-lock
  capture: record its real outcome even when it fails. Observe the actual
  standalone `cargo tree --locked --edges features --invert serde_json`
  after successful input capture/upload, so this later observation cannot
  discard an already captured lock. This graph describes the selected
  package/test feature context; it is not separate packaged-build acceptance.
  Review and commit the actual artifact bytes before the subsequent locked
  correction/acceptance run. An unlocked baseline alone approves no frozen graph.
  The initial CL observation has since completed at `e81bb4e812fa0160bd30cdeb2727d3032df2f3cc`:
  its exact artifact lock is adopted under CL-04/05 before this numeric baseline
  executes. Both existing test commands now use `--locked`; the manifest and
  all selected targets remain unchanged. The adoption provenance is recorded
  in the existing P0 configuration task. It establishes no numerical verdict.
- **CJW-06 — conditional correction.** Only a sound failure with the exact
  resolved parser/feature evidence authorizes changing Coordinator's existing
  production serde_json declaration to request its `float_roundtrip` feature,
  after contract-reading that actual version. Keep crate versions, observed
  lock bytes, default features and all exact assertions. Do not enable
  `arbitrary_precision`, round values or normalize expectations. If the
  baseline preserves values, reassess instead of inventing a failing verdict.
- **CJW-07 — corrected gate and limits.** Execute unchanged numerical and
  affected protocol/lifecycle/relay controls at the corrected exact H with
  committed inputs and `--locked`; retain baseline and fixed evidence
  separately. Any claimed carrier-stdin/stdout-fallback coverage additionally
  needs the real binary harness with raw output retained, because its existing
  typed observer also uses serde_json. No real provider/channel/device, platform,
  performance/soak, full source closure or global acceptance follows from these
  transport contracts. Rollback changes dependency configuration only and does
  not repair historical values; dependency-documented parse cost is not a
  measured product performance result.

## Initial implementation boundary

Root owns this contract and the existing workflow's post-capture feature
observation. The test change is confined to
`source/node-agent-coordinator/tests/coordinator_contract.rs`.
No production manifest/parser change is part of the initial baseline.
All executable verification remains GitHub Actions only.

## Deciding baseline and conditional correction

CJW-06-FLOAT-01 is **FIX / IMPL_BUG** after Actions `41470485288`, Host job
`125362470654`, at H `22b10860e909bea9e4ee3840bab310c2d61fcd4d`.
Both actual HTTP and post-connected SSE probes reached their full-Value
comparison and failed: the retained wire value `1793000000000.0015` decoded
as `1793000000000.0017`. Positive and negative expected bits ending `00006`
became `00007`; the other five finite/zero/integral controls retained their bits.
The real standalone feature observation resolved `serde_json 1.0.154` with
`default/std`, without `float_roundtrip`. This is production decoder configuration
evidence, not a fixture, transport or timeout failure.

Enable only `float_roundtrip` in the existing Coordinator serde_json declaration.
Keep default features, version constraint, observed lock bytes, production
parser entrypoints and every independent expectation unchanged. The exact
[1.0.154 feature contract](https://github.com/serde-rs/json/blob/v1.0.154/Cargo.toml)
and [selected number conversion paths](https://github.com/serde-rs/json/blob/v1.0.154/src/de.rs)
support this configuration; they do not establish a measured product cost.
The baseline's 55 passing Coordinator cases do not include the two later relay
targets, which Cargo did not reach after this failure. The corrected exact H
must execute both numerical probes and all three selected targets with `--locked`,
retain the actual feature graph and pass the shared gates. No corrected result,
binary carrier coverage, real service compatibility or global acceptance is
claimed from this source change.
