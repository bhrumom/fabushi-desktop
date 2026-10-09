# Revision 9 upstream rebaseline dossier — b0d1fe5e order 5166

Previous upstream: `811b83a1cc5f61bd4238ab3ebfcbee6302078014`
Live upstream: `b0d1fe5e08c963402b8b71648c70a2bf68e6fab2`
Live root tree: `e279373aec283ebfcc6e3bb7385e51205acd84a2`
Changed path count: 1
Changed deterministic order: 5166
Path: `Telegram/SourceFiles/media/view/media_view_overlay_widget.cpp`
Old blob: `03138e6eeb429ca38a1b8edafa0a9a7d9742cb1a`
New blob: `e35f67119323850a3cea450548566186c8a2231c`

The path set is unchanged, so root non-directory remains 6,653 and recursive non-directory remains 16,125. Direct/nested gitlinks and build-time acquisition paths are unchanged.

The product delta removes the `updateControls()` branch that, for a message with `forbidsSaving()`, hid document controls and automatically invoked `DocumentSaveClickHandler::Save(fileOrigin(), document, Mode::ToCacheOrFile)` when the media was not loaded. The live responsibility is therefore stricter: opening/rendering/refreshing a protected document in the media viewer must not itself persist the bytes to disk/cache-file. This is a storage/privacy/security product invariant, not a test-only change.

The existing order-5166 MediaViewer lifecycle responsibility remains mapped-open, not verified. Its exact blob identity and consumer evidence were replaced with the live blob and the no-auto-persist invariant. No unknown, omission, implementation or release counter is falsely closed.

Because this changed blob lies inside the existing deterministic prefix, it was explicitly re-read before retaining prefix closure. Orders 5806–5813 were separately read under the unchanged path order, advancing read-through to 5,813. Fresh exact-head GitHub Actions evidence is required for the descendant target HEAD.
