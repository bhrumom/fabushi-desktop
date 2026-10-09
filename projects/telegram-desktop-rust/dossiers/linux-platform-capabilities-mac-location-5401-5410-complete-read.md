# Linux platform capabilities and macOS location 5401-5410 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5401-5410 were read completely from exact blobs.

- **5401 — `Telegram/SourceFiles/platform/linux/text_recognition_linux.h` @ `d2d05be5871f0e17ddd8ed7528c192af1eb05b24`**: Linux OCR availability explicitly false; canonical mapping: canonical text-recognition capability gate.
- **5402 — `Telegram/SourceFiles/platform/linux/translate_provider_linux.cpp` @ `acf8279100d27312292bba30b7fbe0347ac304f8`**: external translation provider discovery, host-process request and typed error result; canonical mapping: canonical translate provider + sandbox/process owner.
- **5403 — `Telegram/SourceFiles/platform/linux/translate_provider_linux.h` @ `8e7ad9a263d9ecb5268ae7a76f88690bf06332cf`**: typed platform translation provider boundary; canonical mapping: canonical translate provider.
- **5404 — `Telegram/SourceFiles/platform/linux/tray_linux.cpp` @ `6b154a49ec2d35a86c14fd6a55ed6c66c8e846ab`**: tray icon theme/counter/muted caching, watcher-driven recreation and menu/click lifecycle; canonical mapping: canonical Electron tray/window/notification owner.
- **5405 — `Telegram/SourceFiles/platform/linux/tray_linux.h` @ `181de22b983710fbec156d7c92094b39af4b0f0f`**: typed tray lifecycle/event boundary; canonical mapping: canonical Electron tray/window owner.
- **5406 — `Telegram/SourceFiles/platform/linux/update_install_linux.cpp` @ `45d81625f43196fb990ef231704ac076be173cc9`**: trusted path/file checks, authenticated package verify, private staging, destination preflight, privileged helper and post-install digest verification; canonical mapping: source/electron-main/update + packaging/signing/security owners.
- **5407 — `Telegram/SourceFiles/platform/linux/update_install_linux.h` @ `e5edf6d427fc66bd423ec1c0334751b7fa536029`**: typed writable/protected/refused updater boundary; canonical mapping: source/electron-main/update + packaging/security owners.
- **5408 — `Telegram/SourceFiles/platform/linux/webauthn_linux.cpp` @ `c24e63bab5b7cb57aa65ef92f93cfc08011d6364`**: passkey registration/login routing to cable or libfido2 security key; canonical mapping: canonical account/auth/passkey + platform security adapter.
- **5409 — `Telegram/SourceFiles/platform/mac/current_geo_location_mac.h` @ `6bb8195430d9cf504d93db967807da5e11ae73eb`**: typed macOS location adapter boundary; canonical mapping: canonical location/permission owner.
- **5410 — `Telegram/SourceFiles/platform/mac/current_geo_location_mac.mm` @ `0ea55446387b2043b3112f75edbdd41af369b241`**: authorization-aware location lifecycle, fail-closed callback and reverse-geocode projection; canonical mapping: canonical location/permission + platform adapter.

The protected updater responsibility is high-risk and remains mapped-open: existing Fabushi update/packaging owners do not receive equivalence credit until trusted-path, authenticated-package, staging/preflight, privilege boundary, post-install verification and failure recovery are proven on the same exact head. Linux OCR is explicitly unavailable upstream and therefore is a capability-delta input, not permission to silently claim a Fabushi OCR implementation. Unknown remains 15,841; omitted remains 0.
