# Attachment content publication — CAP-v1

Status: implementation candidate; independent review and exact-H Actions pending.  
Project: OCRM-001; existing M23/M24 Attachments owner.  
Discovery H: `4c582ea2b1ddcf1dc6a46d6a6bb5aa7575c0b9f0`. Implementation base:
`aa5a448e5758bd2b46aed92e699f56861ad0acdc`. First integration base:
`7eb81e20b7dc2eac5a71516a9a0c78795d252e95`; the attachment owner is unchanged.
The shared workflow preserves the incoming channel, pairing and Automation checks.
Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.

## Problem and decision

At discovery H, `attachments_service.rs::write_content_addressed_file` calls
`exists()` and then writes directly to the final filename. If that write is
interrupted, a later upload of the same bytes sees the partial file and reports
the input's complete hash and length without reading the retained object.
Generated-image persistence has the same failure because it uses that helper.
An existing directory or link can likewise be mistaken for a valid object.

**CAP-DEFECT-01 — FIX / IMPL_BUG**, recorded before implementation: publish
fully written content without overwriting an existing object, and validate the
existing object's exact bytes before reporting deduplication success. Keep
ownership, wire shapes, file naming, byte limits and downstream consumers.

This is a necessary shared persistence contract, not the complete desktop
composer migration. The actual preload/MainEdge staging payload mismatch,
target/account binding, legacy drafts and offline credential questions remain
open. It also does not close `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001`: directory
capabilities, replacement-resistant parent handles, revocation and consumer
acceptance are separate requirements.

## Ownership, interface and actual callers

The registered `uploadAttachment` HTTP method uses the existing Host lane,
`dispatch_gateway_call`, and `AttachmentsService::dispatch_gateway`/`upload`.
Its producer-supplied bytes reach `ingest_attachment_bytes`; existing text,
image, chunk and Box consumers receive the same canonical Agent attachment
path. Ordinary HTTP uploads are serialized by that Host lane; multiple
concurrent HTTP uploads are not asserted as a current production fact.

The actual App generated-image persistence closure and
`AttachmentsService::persist_image_bytes` reach `persist_image_bytes` and the same
publication helper. The generated resource consumer receives its existing
path, file URL, byte count and dimensions only after publication succeeds.
The Host already holds its persistent `host_lock` before constructing owners.
No store, scheduler, account authority, lock owner or runtime is introduced.

The storage boundary is the directory already chosen by these existing owners.
Preserve existing parent-directory and attachment-file creation permissions;
the secret-store's 0700/0600 policy is not an attachment policy. A new helper, if
needed for placement, remains private to the existing Attachments module.

## Required behavior

| ID | Contract |
| --- | --- |
| CAP-01 | Preserve SHA-256 filenames, lowercase/default extensions, paths, file URLs, image dimensions, API/error envelope, empty-input behavior and the existing 25 MiB / 200 MiB admission limits. Valid old objects are reusable without rewriting them. |
| CAP-02 | For an absent final object, create one uniquely named adjacent staging file with `create_new`, write all bytes, sync that file, then publish using an atomic no-overwrite operation. A final name must never expose an incomplete write from this owner. Sync the parent directory where the existing supported platform primitive permits it, including acceptance of a valid existing winner so retry does not bypass a previous failed directory sync. No direct-final-write fallback is allowed. |
| CAP-03 | An already present object, including a winner published by another call, is accepted only when an opened regular non-symlink/non-reparse file has the exact input length and bytes, with bounded comparison and an EOF check. Reject unreadable, truncated, same-length corrupt, oversized, directory, symlink/reparse or otherwise invalid entries. Do not overwrite, unlink, rename or silently repair the pre-existing entry. Errors contain no payload bytes. |
| CAP-04 | Each call cleans up only its own successfully created staging file on success or failure. It never removes a pre-existing final object or another operation's temporary file. Failure after publication is an unknown acknowledgement of an intact object; retry verifies and reuses it. Process interruption before publication can leave only an unpublished temporary object; restart must not mistake that object for a completed final path. No age-based or broad-directory cleanup is added. |
| CAP-05 | Independent calls for identical content converge on one complete final object without truncating the winner. The owner uses bounded comparison storage rather than copying another full attachment into memory. No new global mutex serializes unrelated objects. This portable helper property is distinct from a claim of parallel shipping HTTP dispatch. |
| CAP-06 | Both upload and generated-image persistence propagate corrupt-winner/publication errors through their existing result boundary. Their valid controls still reach actual read/chunk/image consumers with unchanged bytes and metadata. A returned success cannot describe bytes other than those stored at its content-addressed resource. |

The destination selected by the existing owner and its ancestors are assumed
stable during this bounded operation; this is not a newly established
authorization or directory-identity capability. The no-overwrite operation may
report a filesystem/platform error rather than
silently degrade to a replacing rename or direct write. Leaf-file no-follow
checks do not claim safety against an attacker replacing the selected parent
directory or later modifying the accepted file. General read-scope authority,
resource GC and full power-loss/packaged-filesystem acceptance remain open.
Valid read-only legacy objects remain reusable; historical file-data durability,
new ancestor-directory durability and Windows directory-flush guarantees are
not retroactively established by that reuse.

## Implementation and verification

Reuse the existing native Rust filesystem pattern demonstrated by
`host_secret_store::{stage_config_revision_key,publish_config_revision_key}`:
owned `create_new` temporary, `write_all`/`sync_all`, non-replacing `hard_link`,
then actual winner validation. Reuse the existing UUID, standard IO and libc
dependencies. Do not call or copy the LocalExec JSON sandbox API; its root
admission and version protocol belong to another owner. Avoid extracting a
generic cross-owner framework for this bounded correction.

Allowed product paths are the existing `attachments_service.rs`, an optional
private helper under the same Attachments module with its module declaration,
and existing `attachments_service_contract.rs` / focused owner tests. Root
serializes this spec, the existing media dossier, and selection/receipts in
`.github/workflows/rust-desktop-runtime.yml`. Existing ledger/RTM responsibility
identities and all completion/read facts remain unchanged; evidence stays
under their existing dossier/test references, without a parallel ledger.

Actions oracles must exercise real files and the actual owner, not merely
source-string matching: seed interrupted and same-length corrupt final files;
refuse them without mutation through `uploadAttachment` and image persistence;
reopen the owner and retry; accept an intact pre-existing object; reject a
directory and platform-supported link; preserve unrelated temporary files;
and coordinate competing publishers while retaining each caller's result.
Use an actual non-root Unix permission fixture for unreadable existing objects
and an unwritable destination, paired with successful readable/writable
controls. Root execution cannot decide these permission negatives and must
fail their setup rather than silently count a skipped or bypassed case.
Include a causal staging/publication boundary control which proves the final
name is absent until the bytes have been completely staged, without a timing
assumption. Retain existing upload→read/chunk/Box and image metadata controls.
No test-only alternate persistence implementation may serve as the oracle.

Freeze the complete change with exact base/tree, full diff/status and prior
round deltas, then obtain independent general/security and
`gpt-6.1-sol` reuse/structure/performance review at xhigh effort. Only GitHub
Actions may execute compilation, tests or other validation. Reuse the existing
media suite/cache and require all new named positives plus original controls;
record actual commands/exits/log identities and the exact candidate H. Missing
platform runners remain blocked. A source review or authored test is not a pass.

## Migration, rollback and acceptance

No stored format, canonical resource filename, account/session state or public
wire schema changes. Existing correct files survive unchanged. Corrupt old
objects produce an explicit error and remain intact for later owner-guided
recovery; the batch does not delete user data to make retries succeed.
Rollback must retain complete-object verification and no-partial publication,
or disable the affected write entry rather than restore silent corruption.

Completion for CAP-v1 requires implementation in the existing owner, both
consumer paths preserved, independent source convergence and exact-H affected
Actions evidence. This does not confer global verified/accepted status,
upstream whole-file read credit, full source/dependency closure, real
provider/channel/device acceptance, composer parity or packaged platform
acceptance. All G01–G12/AC01–AC24 and eight global flags retain their existing
requirements/state.

## References

- OCRM primary spec §17, SOURCE_OF_TRUTH and module-map M23/M24.
- FBCP canonical resource/storage/language/migration contracts; no TDRP
  source or acceptance evidence is reused.
- `source/host/src/extensions/attachments/attachments_service.rs` and its
  current service/generation callers; `source/host/src/host_secret_store.rs`.
- Existing `media-generation-attachments-scoped.md` and
  `media-store-read-scope.md`; their historical source identities and broader
  open responsibilities are retained.
- Standard-library API contract references, observed as Rust 1.99.1 docs:
  [hard_link](https://doc.rust-lang.org/std/fs/fn.hard_link.html),
  [OpenOptions](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html) and
  [File::sync_all](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all).
  These describe API behavior, not new dependency-source or platform acceptance.
