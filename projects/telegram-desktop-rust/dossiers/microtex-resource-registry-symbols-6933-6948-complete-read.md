# MicroTeX resource registry and symbol closure — deterministic read 6933–6948

Authority:
- `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`
- mounted `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`

This batch exact-reads the remaining StMary/mono font definitions, the font-definition macro contract, resource and registry Meson closures, builtin font/symbol registries, AMS/base/StMary/special symbol tables, and the symbol-definition macro contract.

Applicable semantics are deterministic math typography, extensible delimiter/operator assembly, symbol aliases/name→font/code mapping, dependency initialization and build/resource provenance. They remain mapped-open to the existing AssistantMath/KaTeX dependency interface and Build/Release provenance owner. The source registry does not justify a second Fabushi registry or MicroTeX runtime.

No unknown is closed by reading. Current accounting after this batch: read-through 6,948 / 16,125; unread 9,177; unknown 15,845; omitted 0. First unread is order 6,949 `Telegram/ThirdParty/MicroTeX::src/samples/gtkmm_main.cpp` @ `d491af61d05fbcc5d17c48f64e76664f91d5fce8`.
