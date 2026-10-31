# Round 31 — fixed-U authority and recursive provenance closure

Date: 2026-10-31
Stage: A (manual Git/API/source evidence only; no CI/Actions status query or execution)
Checkpoint before write: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1@2bab866d21a02c013b11a27889be79a8f8e8d9c6`
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Root authority re-anchoring

`root-mutable-authorities.json` was reread against fixed-U source blobs and normalized to 48 authorities / 67 occurrences. The prior separate Ada v3.2.4/v3.2.9 records converge at fixed U to v3.2.9. Current source changes also include libxcb 1.16.1, libXtst 1.2.5, libXv 1.0.13, libXrandr 1.5.4, libXrender 0.9.12, OpenSSL 3.5.9, Docker xz 5.8.4, the exact OpenAL Windows checkout 00cb9b649c1a6ad1802d88ee4458979164529238, and prepare.py xz checkout 05af863c3166cddeb4ab935829b14e995c8346b4.

All 67 occurrences now bind fixed U plus the exact fixed-U source blob. Forty-seven unique GitHub-readable authority commit objects and their root-tree SHAs were independently fetched. Authorities whose source is still HEAD/branch are retained as live resolution evidence only and are explicitly not represented as immutable U pins.

`root-tool-authorities.json` now binds fixed U and current source blobs. The current candidate set is 16 lines: 14 tool-bootstrap lines and 2 versioned archive lines. The historical Snap rustup direct-download line is absent at fixed U; the Snap uses its Rust plugin/channel instead.

## Recursive build-acquisition child closure

Exact GitHub commit/root-tree reads were obtained for abseil/abseil-cpp, google/crc32c, cisco/libsrtp, and the three crc32c gitlink children google/benchmark, google/glog, and google/googletest. All six recursive trees report truncated=false. CRC32C exposes exactly those three gitlinks; each of their trees exposes zero further gitlinks.

## Generated-input closure

Exact commit/root-tree reads were obtained for desktop-app/codegen@dbd1d53137cd581cfdbeebe93bcaf54abf55ec74, desktop-app/lib_tl@29746d31247c915a9ec870241399a9d103c39f87, and flatpak/xdg-desktop-portal@23a76c392170dbbd26230f85ef56c3a57e52b857. Their recursive trees are non-truncated and have zero gitlinks. Exact consumed blobs were read for codegen/lang/generator.cpp, lib_tl/tl/generate_tl.py, and data/org.freedesktop.portal.Background.xml.

## License identities

The fixed-U root LEGAL, LICENSE and README.md blobs were read directly. LICENSE/COPYING blobs for abseil, crc32c, libsrtp, googletest, glog, benchmark, and xdg-desktop-portal were also read directly at their pinned child commits. This advances identity provenance only; it does not claim that Fabushi has adopted those dependencies or completed copied/derived-content or release legal review.

## Blockers and gates

No new original-host tree/blob evidence was obtained for gitlab.com/mnauw/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776, so git_tree_verified=false and fail-closed-original-host-unread remain authoritative. No Stage A CI or validation was queried or run. baseline_ready=false, accepted=false, validation_status=not-run.
