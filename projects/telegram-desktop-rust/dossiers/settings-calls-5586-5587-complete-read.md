# Settings Calls complete read — orders 5586-5587

Accepted upstream: `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Exact blobs: `settings_calls.cpp` `99ce45c03b9ade8f04e5f2ebd2b2ca74c1f0ce2a`; header `686dddae0a1d9ec92c51822757827cc3a8e6b0b8`. The implementation and complete header were read under the live authority.

The section projects canonical call-media settings rather than owning a second media runtime: global output/input device selection with default/inactive fallback; call-specific same-vs-separate speaker/microphone preferences; live microphone level testing; camera selection/preview; OS microphone permission request/recovery; OS audio-settings routing; and current-authorization incoming-call enable/disable. Camera preview explicitly yields capture while an active call or group call exists and tears down its temporary capturer.

Failure/fault requirements stay open: device hotplug/inactive fallback, permission denial/regrant, mic tester teardown, camera preview cancellation/current-call exclusion, account switch, server authorization reconciliation, stale capture switching, keyboard/focus/a11y/light-dark/responsive chooser behavior and recovery must receive same-head evidence.

No Telegram Calls settings runtime or source-named control is introduced. Both rows are `mapped-open`; unknown remains 15,844 and omitted remains 0. Deterministic read-through is now 5,587/16,123; first unread is order 5,588 `Telegram/SourceFiles/settings/sections/settings_chat.cpp` (`0f48c2eaa80a66cf280abd50fd6d137e9d8dd7be`).
