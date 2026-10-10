# Telegram folder resources 901–1000 — complete source read

Upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

Evidence: Fabushi Desktop `1983e056a9b31ab4f515eae2d0355609a6d51be4`; GitHub Actions run `37854668389`; source-authority job `113576044767`; artifact `11583228458`; digest `sha256:1d65039b863e36fdd08c241fd44d9d0fa04801b24c811c2bd20cd734fc81d5c8`.

## Responsibility decomposition
- 901–902 finish the existing-chats filter scale family begun at 900.
- 903–905 are the new-chats filter family.
- 906–992 are one folder/filter icon vocabulary consumed by `ui/filter_icons.style` (with a small secondary reuse such as channels in credits). Their decorative names are not separate product capabilities. Fabushi maps semantic folder metadata to canonical source-neutral icon tokens in Picker/Tabs/Menu.
- 993–1000 begin built-in folder-type predicates (archived, bots, channels). The channels scale family continues at order 1001 and is not artificially closed here.
- Every symbol in 901–1000 has at least one accepted-tree SourceFiles consumer. No reachability-open item is manufactured closed or omitted.

## Accounting
Read-through advances 900 -> 1,000. Unread falls 15,220 -> 15,120. Unknown stays 16,033; omitted stays 0.

## Build-time provenance carried by the same exact-head source artifact
`build-time-github-action-ref-resolutions.json` resolves 88 observed top-level GitHub Action occurrences / 27 unique refs to exact commits, including 15 unique mutable tag/branch refs, and records the observed transitive action graph with zero unresolved action-metadata children. This is a real provenance advance but not overall closure: mutable source references and non-Action package/download/build inputs remain fail-closed.
