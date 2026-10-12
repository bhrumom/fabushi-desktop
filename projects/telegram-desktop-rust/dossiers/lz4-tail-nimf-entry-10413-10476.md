# LZ4 tail and Nimf entry — deterministic source read 10,413–10,476

Evidence: Fabushi `362099f6a830c1de3a493d4e718f1fe850bec4a1`, Rust runtime `38128655830`, source-authority job `114438220656`, artifact `11698361416`, digest `sha256:bac2f13bc1492b139d1670a2c106151be410618636049e852999cb7fb4f248ee`.

This range finishes `lz4/lz4@5ff839680134437dbf4678f3d0c7b371d84f4964` and starts `hamonikr/nimf@498ec7ffab3ac140c2469638a14451788f03e798`.

LZ4 tail responsibilities include adversarial undersized-buffer safety, safe full/partial/dictionary/stream decoding, exact round trips, frame/skippable/concatenation semantics, file/stdin/stdout overwrite/remove/sparse behavior, ABI/version compatibility, install/uninstall, freestanding/custom allocator/custom block sizes, performance regression and GPL license boundaries. Fabushi's current gzip pack/resource owner already has bounded compressed/uncompressed sizes, checked arithmetic, path containment, disk/resource admission, SHA-256+size verification and cleanup. No LZ4 runtime or second archive owner is added.

The first six Nimf rows are support/release/provenance metadata only: structured issue reporting, tag-triggered Debian package build/release, generated-file exclusions and authorship. They remain mapped-open until the actual IME/product source is read.

All 64 rows are read/decomposed, `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting: **10,476 read; 5,649 unread; 15,845 unknown; 0 omitted**. Next: 10,477 `Telegram/ThirdParty/nimf::BUILD.md@fbea94fb3836ea4d04faf23734f741891731393b`.
