# Upstream rebaseline 22b352e -> 36a0c87 exact changed-source read

- Upstream: `telegramdesktop/tdesktop`
- From: `22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`
- To: `36a0c87ca096c48ccf6193aa2c31707fdcafcc7c` / tree `94e009f981d886ee55cd0450f0a5cc9b38305ba8`
- Ahead by: 5 commits
- Top-level changed authority paths: 5
- Recursive path-set effect: none. `cmake` advances from `12cdedd007b6b3c36df2349dae1996384e535ede` to `699262e441dbc6270f0ba6cc74f2e7a8e4c7e422`; that child changes only `external/ada/CMakeLists.txt`, so the existing 16,120 non-directory recursive denominator remains path-stable.

## Changed responsibilities

1. `Telegram/SourceFiles/editor/editor_trim_timeline.cpp` (`3de795c6...`, order 4112): Qt 5 native-gesture coordinate compatibility changes `globalPosition().toPoint()` to `globalPos()`. The existing media editor trim/zoom/seek lifecycle, cancellation, rendering and teardown responsibility is unchanged. This entry was re-read and its exact blob rebound; implementation/release state remains open.
2. `Telegram/SourceFiles/iv/markdown/iv_markdown_article.cpp` (`c219f0b7...`): two aggregate initializations become explicit `Ui::BubbleRounding` construction for compiler compatibility. Existing IV Article content and interaction responsibilities remain mapped/pending; no new read-through credit is created by this rebaseline.
3. `Telegram/SourceFiles/ui/chat/chat_theme_readability.cpp` (`29dd9596...`): adds explicit `<QtCore/QtMath>` dependency. Theme readability semantics are unchanged. This path is outside the current 1-4500 source-disposition prefix.
4. `Telegram/build/docker/centos_env/Dockerfile` (`e6d22fce...`): adds `perl-Time-Piece` to Linux build prerequisites. This is build/tooling provenance, not shipping UI/product behavior, and it adds no new download/clone/ref candidate.
5. `cmake` gitlink -> `desktop-app/cmake_helpers@699262e...`: Windows ada linkage switches from the singleheader artifact to standard `out/src` libraries and `${libs_loc}/ada/include`. No child paths or nested gitlinks are added/removed; build-time authority is rebound and remains fail-closed pending current-head Actions evidence.

## Accounting

- read-through: 4,500 (unchanged)
- unread: 11,620 (unchanged)
- unknown: 15,841 (unchanged)
- unknown-closed: 279 (unchanged)
- omitted: 0
- implementation: 21 implemented / 34 mapped (unchanged)
- coverage matrix: 55 pending
- release gate: 55 pending

No changed source was used to promote `implemented`, `verified`, coverage, or release status. The prior 22b352e Source authority artifact is historical only; a new exact-head GitHub Actions Source authority artifact is required after this rebaseline commit.
