# Memory citation formatting

Status: active implementation slice; production verification pending  
Owner: existing Host content-search/memory owner

## Context and goal

The fixed OpenClaw source formats memory search citations as `path#L<start>`
or `path#L<start>-L<end>`, and suppresses automatic citations for channel and
group sessions. Fabushi's current transcript search results contain snippets,
entry IDs and timestamps, but do not contain verified file path/line
provenance. This spec defines the safe Rust policy boundary without inventing
line numbers.

## Non-goals

- No second search index, memory store, transcript owner, or channel policy.
- No citation derived from snippet offsets, entry order, timestamps, or guessed
  file names.
- No claim that the policy is production-wired until a real memory producer
  supplies provenance and a real result consumer renders it.

## Requirements

- `MCF-01`: format a single-line citation as `path#L<start>`.
- `MCF-02`: format a range as `path#L<start>-L<end>`.
- `MCF-03`: reject empty paths and non-positive line numbers.
- `MCF-04`: resolve mode `on`, `off`, and `auto`; auto suppresses citations
  when the session key contains `channel` or `group` tokens.
- `MCF-05`: keep the formatter pure and preserve the existing content-search
  owner boundary.

## Current and target state

Current Rust search results have no path/line provenance and therefore cannot
be rendered as memory citations. The target adds a pure policy module that can
format only an explicit provenance record and resolve the mode; the producer
and consumer remain separate follow-up wiring work.

## Failure modes and safety

Missing provenance, blank paths, zero/negative lines, or malformed session keys
must fail closed: return no citation or use the conservative auto default.

## Verification and Definition of Done

Focused unit tests must cover single lines, ranges, invalid provenance, `on`,
`off`, auto suppression for channel/group keys, and auto inclusion for ordinary
keys. Static source review must show no inference from snippets or timestamps.
Exact-head Actions execution and a real producer→formatter→consumer path are
required before verification or acceptance is promoted.

## References

- Fixed OpenClaw `extensions/memory-core/src/tools.citations.ts` at commit
  `ad8d12f126d29b0db204187adf5092db80294e70`.
- `source/host/src/extensions/content_search/agent_content_search.rs`.
- `projects/openclaw-rust/dossiers/memory-knowledge-retrieval-scoped.md`.
