# Existing desktop link-preview mapped-IP admission — LPIP-v1

Status: bounded preservation of the existing Fabushi desktop preview path;
exact-subject Actions and full OCRM acceptance remain required. Target baseline
is `9004481ef1c34e9c99acf9e6cdab50081193179a` (tree
`5153a756ad5308bd64434583589498fe81c45dd9`). Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`.

Integration continues from concurrent parent
`4df79ca181c8daa98080dbea0c8e8143c0407e05` (tree
`f70a3977fa00dcc4f879b75257c74562297978b5`), preserving its checked Automation
delivery implementation, tests, evidence and selections. The LPIP production
and test blobs remain those reviewed against the original baseline; shared
workflow/RTM integration receives a separate focused review.

## Existing owner and defect

The renderer's `DesktopBridge.getLinkMetadata` passes through preload,
Electron `main-edge` and the attachment/coordinator adapter to the existing
Host `AttachmentsService::dispatch_gateway("fetchLinkMetadata")`.
`fetch_link_metadata` validates the URL before reading the existing preview
cache. HTML, preview-image and favicon downloads use the existing
`safe_link_preview_fetch` owner. Its literal-URL parser, DNS answer validator
and redirect parser all consume `is_blocked_link_preview_ip`.

That classifier applies the IPv4 rules only to `IpAddr::V4`. The IPv6 branch
does not inspect the IPv4 part of `::ffff:V4`, so an otherwise blocked
loopback/private/link-local address can pass as mapped IPv6. The same gap can
authorize a cached preview or a mixed DNS answer set. Correct this in the
existing classifier; introduce no new fetch, cache, permission or runtime
owner.

This batch preserves an existing Fabushi capability under OCRM's media and
regression obligations (R24/AC24). It is not evidence that OpenClaw's separate
link-understanding or general-media fetch path is connected or equivalent.
Current-U corroboration is `src/infra/net/ssrf.ts` blob
`664540bba2ec156f9ef5ef57bfd1fb7bd7f93d0d` and
`packages/net-policy/src/ip.ts` blob
`e860d9a558698921798e8d34f652d9fe2b33ee89`, whose private-IP policy examines
embedded IPv4. This is a bounded policy reference, not full network-source
or dependency closure. The applicable target IPv4 rules are the existing
Fabushi preview rules at this baseline.

## Required behavior

| ID | Contract and oracle |
| --- | --- |
| LPIP-01 | An IPv4-mapped IPv6 address receives the same allow/block decision as its embedded IPv4 address. Block loopback, private, link-local, benchmark and other already-blocked IPv4 ranges; retain public mapped and ordinary public IPv6 controls. |
| LPIP-02 | The existing URL and redirect parsers reject mapped prohibited literals with `NonPublicIp`, including dotted and hexadecimal spellings. Existing HTTPS, credential, port and hostname rules remain unchanged. |
| LPIP-03 | The actual connection-target owner rejects the entire DNS answer set if any mapped answer is prohibited, irrespective of answer order. Public mapped answers retain the selected address, port 443, original Host header and TLS server name. Literal URLs do not acquire a DNS lookup. |
| LPIP-04 | The real `fetchLinkMetadata` service call refuses a prohibited mapped URL before serving a fresh cache entry. Valid public and mapped-public cached previews remain usable with unchanged metadata and cache bytes. |

Use `Ipv6Addr`'s mapped-address operation and the same existing IPv4
classification. Do not broaden this batch to IPv4-compatible/other transition
encodings or redefine either existing range list. The classifier is pure; no
new persistent work, cancellation or restart protocol exists. Cache ownership
and download lifetimes stay with their current owners. Reversion must retain
the refusal behavior or disable the affected preview, not re-enable the known
private-target bypass.

## Verification and boundaries

Extend existing `safe_link_preview_fetch_contract` controls for the classifier,
literal parsing, DNS answer selection and redirect parser. Exercise the real
service against fresh on-disk cache records created using ordinary URL
canonicalization, not the policy under test. Negative private records and
distinct positive public records must fail/pass causally without network
access or substituting a fake policy owner. Preserve all original assertions.

Select the affected existing contract target(s) in the current scoped Actions
lane and existing Linux Host job/cache. Require exact H, named positive
results, exit status, log and source hashes in separate bounded receipts.
Do not execute local validation. These fixtures establish admission and the
cache consumer; they do not execute a real HTTP redirect loop, prove complete
DNS pinning, or replace macOS, platform-package, real-provider/channel and
full OCRM acceptance. General-media restrictive host lists, redirect/DNS/proxy
policy, complete link understanding, and all global completion gates remain
open.
