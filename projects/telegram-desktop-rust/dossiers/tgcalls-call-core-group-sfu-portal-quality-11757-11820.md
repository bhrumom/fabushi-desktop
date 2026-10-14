# tgcalls call-core / Group SFU / xdg portal quality — 11,757–11,820

Mapped-open except explicit repository-only metadata at 11,813–11,816. Source authority: tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6; tgcalls@1c236c09f8d8569fead14bd68000618a52051225; xdg-desktop-portal@23a76c392170dbbd26230f85ef56c3a57e52b857. Exact materialization: 2937161a99e1324032bb492c8af03619fc16e2a7 / 38174934167 / 114575281813 / artifact 11715945600 / sha256:39d60f209321a97b6566d2d19c04c12b801669ebd877e2d58f3fa48502c0fd74.

## Signaling and call-core
11,757–11,759 complete ordered description/candidate translation and strict ICE/DTLS/RTP/media-state parsing. 11,760–11,785 define a single-media-thread, non-reentrant core/host boundary with copied buffers, timer generations, bounded serialization/compression/data-channel sends, perfect negotiation, stats/quality and ICE restart. WAMR/filesystem module behavior is security authority only: upstream production disables filesystem module loading; provenance/range checks and trap/version failures are fail-closed with no native fallback. Fabushi keeps these invariants in existing CallSession/Host/WebRTC owners and does not introduce a second executable call runtime.

## Group and SFU
11,786–11,812 cover deterministic media fixtures, p2p/group/churn runners and the upstream test SFU. Applicable responsibilities are multi-party join/leave/churn; mute/audio-level and directed video assertions; join payloads; SIM/FID simulcast; participant/SSRC registry; publication/subscription routing; per-receiver layer selection; REMB/TWCC; PLI/FIR; RTX/SSRC rewrite; ICE/DTLS/SRTP/SRTCP/SCTP lifecycle; bounded demux/backpressure; and delay/jitter/loss/bandwidth fault injection. Test-only insecure certificate verification is forbidden as shipping behavior.

Existing-owner-first target is the current CallSession/HumanCallControls/Coordinator/Host/native-messaging/backend Human Call chain. The shipping service is authenticated, durable and generation-fenced but still one-to-one; Group/SFU therefore remains a P0 gap. Closure requires durable multi-member roster/session state, authorized mutation, short-lived SFU join lease, publication/subscription identities, simulcast/media routing, generation/reconnect fencing, failure recovery and real >=3-party E2E. The CLI/Go SFU, raw-TCP hub and current 1:1 WebRTC path are not completion evidence.

## xdg-desktop-portal boundary
11,813–11,816 are contributor/funding/issue routing only and are development-only non-applicable. 11,817–11,820 define reproducible Linux portal dependency CI, GCC/Clang ASan, installed tests, Qt DBus annotation checks, dist creation and POTFILES/localization completeness; these map to existing Linux platform/dependency provenance and quality owners.

Accounting: read-through 11,820; unread 4,305; unknown 15,845; omitted 0. Development-only summary becomes 101 only from the four repository-metadata rows; unknown_closed remains 280. Next: 11,821 Telegram/ThirdParty/xdg-desktop-portal::.github/workflows/main.yml@1ea5bf947ae71ab23e326860253d1e5fbd6bc62b.
