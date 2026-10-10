# MicroTeX utility boundary — deterministic read 6963–6976

Authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` -> `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.

The exact blobs cover sorted dictionary lookup, typed layout enums, detailed parse/resource exception taxonomy, indexed tables, optional logging, numeric aliases, token/string helpers, UTF conversion, binary search, utility build closure and the tinyxml2 vcpkg dependency manifest.

Two explicit safety obligations are preserved: binary lookup must never read one-past-end, and byte classification must cast to `unsigned char` before C-library ctype calls. More generally malformed Unicode/numeric/resource input must fail locally without corrupting later renders. These invariants map to the existing AssistantMath/KaTeX boundary and Build/Release dependency provenance; the MicroTeX C++ utility layer is not copied.

Accounting: read-through 6,976 / 16,125; unread 9,149; unknown 15,845; omitted 0. First unread is 6,977 `Telegram/ThirdParty/QR::Readme.markdown@66292b4b93bdd968738066827f8cb91e66ffe170`.
