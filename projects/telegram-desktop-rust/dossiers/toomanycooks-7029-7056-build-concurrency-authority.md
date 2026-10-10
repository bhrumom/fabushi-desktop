# TooManyCooks — deterministic read 7,029–7,056

Authority: `telegramdesktop/tdesktop@6fed91ff...` -> `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` (Boost-1.0).

This batch exact-read/decomposes formatting, CI matrices, project authority, build/package configuration and README contracts before the runtime headers. The CI authority covers ASan/TSan/UBSan, clang/gcc, 32/64-bit, macOS/Linux/Windows, hwloc on/off, container CPU quota/cpuset, CORO/FUNCORO, coverage and channel fuzzing. The project authority adds cold-task semantics, executor/priority inheritance, resume affinity, mandatory await/detach for linear awaitables, fork/join lifetime, coroutine-lambda capture UAF hazards, work-stealing CPU executor, optional Asio, topology-aware placement and configuration consistency across consumers.

Existing-owner-first maps these responsibilities into Fabushi Coordinator/Host/Runner/ConversationActor/CapabilityBroker runtime owners plus Build/Release quality gates. It does **not** create a TMC/TooManyCooks runtime, provider or second conversation/message state system. Production equivalence is mapped-open; reading closes no unknown.

Accounting after this batch: **7,056/16,125 read; 9,069 unread; 15,845 unknown; 0 omitted**. First unread: order **7,057** `Telegram/ThirdParty/TooManyCooks::include/tmc/all_headers.hpp@9f14633b57c5d4b319d1309b4bc6db643bc1822d`.
