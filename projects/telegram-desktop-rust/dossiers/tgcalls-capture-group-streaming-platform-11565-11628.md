# tgcalls capture / group streaming / platform responsibilities - orders 11,565-11,628 complete read

Exact source-window evidence: parent HEAD `0347515e75b76c2e093e8777c0155f98280620bc`; Rust desktop runtime `38167547405` success; Desktop Chat Parity `38167547407` success; source-authority job `114553216026` success; rust-host `114553215833` success; artifact `11711859551`, digest `sha256:3b80f4844114053ef649c8a8a63a4d24397f8ea38a27d8fbbc84a7a4971cb6d2`. All 64 repository/commit/path/Git-blob identities were materialized exactly.

## Responsibility decomposition

Exact-blob reading identifies TURN/STUN policy; camera/desktop capture lifecycle; group audio/video decode and buffering; group join plus ICE/DTLS/SRTP/RTP/RTCP transport; and Android/Darwin platform adaptation. Existing-owner-first maps these to canonical CallSession, Conversation, Permissions, Computer capture, Resource, WebRTC/media/network and thin platform adapters. No TelegramCallRuntime, tgcalls FFI/sidecar, second call store or second Conversation/Resource store is introduced.

The existing two-party Human Call owner already has generations, device-scoped leases, authenticated remote call/event sync, Host-owned ICE credentials, WebRTC signaling, ICE restart, permission gates, source fencing, rollback and diagnostics. This slice adds capture pause/active projection: selected desktop MediaStreamTrack mute/unmute maps to screenSharePaused/screenShareActive, is fenced to the owned capture, cleared on stop/end, and projected accessibly.

Group streaming is not marked verified. GroupInstance, GroupNetworkManager, GroupJoinPayload, StreamingMediaContext, AudioStreamingPart, VideoStreamingPart and simulcast require multi-party routing/SFU-equivalent service behavior, authz, persistence, timeout/reconnect/recovery and real E2E. The evidenced /api/social/calls, /events and /ice contract is two-party and cannot close group-media responsibility.

Concurrency/lifecycle/security: capture work is kept off the UI path and source/sink lifetime is fenced; Fabushi releases tracks/peer connections/timers at teardown. TURN/ICE credentials stay outside renderer and Host fetches them with bearer auth plus x-fabushi-device-id; malformed ICE URL/TTL lists fail closed. User-media and desktop-share remain behind canonical permission/capture bridges.

## Per-file traceability

| Order | Exact path | Disposition | Canonical owner |
|---:|---|---|---|
| 11565 | `tgcalls/TurnCustomizerImpl.h` | `mapped-open` | CallSession / WebRTC network security |
| 11566 | `tgcalls/VideoCaptureInterface.cpp` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11567 | `tgcalls/VideoCaptureInterface.h` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11568 | `tgcalls/VideoCaptureInterfaceImpl.cpp` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11569 | `tgcalls/VideoCaptureInterfaceImpl.h` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11570 | `tgcalls/VideoCapturerInterface.h` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11571 | `tgcalls/desktop_capturer/DesktopCaptureSource.cpp` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11572 | `tgcalls/desktop_capturer/DesktopCaptureSource.h` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11573 | `tgcalls/desktop_capturer/DesktopCaptureSourceHelper.cpp` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11574 | `tgcalls/desktop_capturer/DesktopCaptureSourceHelper.h` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11575 | `tgcalls/desktop_capturer/DesktopCaptureSourceManager.cpp` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11576 | `tgcalls/desktop_capturer/DesktopCaptureSourceManager.h` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Computer capture / Resource |
| 11577 | `tgcalls/group/AVIOContextImpl.cpp` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11578 | `tgcalls/group/AVIOContextImpl.h` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11579 | `tgcalls/group/AudioStreamingPart.cpp` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11580 | `tgcalls/group/AudioStreamingPart.h` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11581 | `tgcalls/group/AudioStreamingPartInternal.cpp` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11582 | `tgcalls/group/AudioStreamingPartInternal.h` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11583 | `tgcalls/group/AudioStreamingPartPersistentDecoder.cpp` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11584 | `tgcalls/group/AudioStreamingPartPersistentDecoder.h` | `applicable-group-video-call-production-gap` | CallSession / Resource / group media streaming |
| 11585 | `tgcalls/group/GroupInstanceCustomImpl.cpp` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11586 | `tgcalls/group/GroupInstanceCustomImpl.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11587 | `tgcalls/group/GroupInstanceImpl.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11588 | `tgcalls/group/GroupInstanceReferenceImpl.cpp` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11589 | `tgcalls/group/GroupInstanceReferenceImpl.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11590 | `tgcalls/group/GroupJoinPayload.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11591 | `tgcalls/group/GroupJoinPayloadInternal.cpp` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11592 | `tgcalls/group/GroupJoinPayloadInternal.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11593 | `tgcalls/group/GroupNetworkManager.cpp` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11594 | `tgcalls/group/GroupNetworkManager.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11595 | `tgcalls/group/StreamingAudioRenderer.cpp` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11596 | `tgcalls/group/StreamingAudioRenderer.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11597 | `tgcalls/group/StreamingAudioRendererTest.cpp` | `mapped-open-test-harness` | Quality / CallSession media |
| 11598 | `tgcalls/group/StreamingMediaContext.cpp` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11599 | `tgcalls/group/StreamingMediaContext.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11600 | `tgcalls/group/VideoStreamingPart.cpp` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11601 | `tgcalls/group/VideoStreamingPart.h` | `applicable-group-video-call-production-gap` | CallSession / Conversation / WebRTC media-network |
| 11602 | `tgcalls/platform/PlatformInterface.h` | `mapped-open-canonical-platform-adapter-replacement` | Platform adapters / CallSession media |
| 11603 | `tgcalls/platform/android/AndroidContext.cpp` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11604 | `tgcalls/platform/android/AndroidContext.h` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11605 | `tgcalls/platform/android/AndroidInterface.cpp` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11606 | `tgcalls/platform/android/AndroidInterface.h` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11607 | `tgcalls/platform/android/VideoCameraCapturer.cpp` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11608 | `tgcalls/platform/android/VideoCameraCapturer.h` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11609 | `tgcalls/platform/android/VideoCapturerInterfaceImpl.cpp` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11610 | `tgcalls/platform/android/VideoCapturerInterfaceImpl.h` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11611 | `tgcalls/platform/darwin/AudioDeviceModuleIOS.h` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11612 | `tgcalls/platform/darwin/AudioDeviceModuleMacos.h` | `mapped-open-canonical-platform-adapter-replacement` | Platform adapter / CallSession audio |
| 11613 | `tgcalls/platform/darwin/CustomExternalCapturer.h` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11614 | `tgcalls/platform/darwin/CustomExternalCapturer.mm` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11615 | `tgcalls/platform/darwin/CustomSimulcastEncoderAdapter.cpp` | `applicable-group-video-call-production-gap` | CallSession / WebRTC video encoding |
| 11616 | `tgcalls/platform/darwin/CustomSimulcastEncoderAdapter.h` | `applicable-group-video-call-production-gap` | CallSession / WebRTC video encoding |
| 11617 | `tgcalls/platform/darwin/DarwinFFMpeg.h` | `mapped-open-canonical-platform-adapter-replacement` | macOS platform adapter / CallSession video |
| 11618 | `tgcalls/platform/darwin/DarwinFFMpeg.mm` | `mapped-open-canonical-platform-adapter-replacement` | macOS platform adapter / CallSession video |
| 11619 | `tgcalls/platform/darwin/DarwinInterface.h` | `mapped-open-canonical-platform-adapter-replacement` | macOS platform adapter / CallSession video |
| 11620 | `tgcalls/platform/darwin/DarwinInterface.mm` | `mapped-open-canonical-platform-adapter-replacement` | macOS platform adapter / CallSession video |
| 11621 | `tgcalls/platform/darwin/DarwinVideoSource.h` | `mapped-open-canonical-platform-adapter-replacement` | macOS platform adapter / CallSession video |
| 11622 | `tgcalls/platform/darwin/DarwinVideoSource.mm` | `mapped-open-canonical-platform-adapter-replacement` | macOS platform adapter / CallSession video |
| 11623 | `tgcalls/platform/darwin/DesktopCaptureSourceViewMac.h` | `mapped-open-canonical-platform-adapter-replacement` | Computer capture / Permissions / CallSession video |
| 11624 | `tgcalls/platform/darwin/DesktopCaptureSourceViewMac.mm` | `mapped-open-canonical-platform-adapter-replacement` | Computer capture / Permissions / CallSession video |
| 11625 | `tgcalls/platform/darwin/DesktopSharingCapturer.h` | `mapped-open-canonical-platform-adapter-replacement` | Computer capture / Permissions / CallSession video |
| 11626 | `tgcalls/platform/darwin/DesktopSharingCapturer.mm` | `mapped-open-canonical-platform-adapter-replacement` | Computer capture / Permissions / CallSession video |
| 11627 | `tgcalls/platform/darwin/ExtractCVPixelBuffer.h` | `mapped-open-canonical-platform-adapter-replacement` | Computer capture / Permissions / CallSession video |
| 11628 | `tgcalls/platform/darwin/ExtractCVPixelBuffer.mm` | `mapped-open-canonical-platform-adapter-replacement` | Computer capture / Permissions / CallSession video |

## Accounting

Exact blob read/decomposition advances to **11,628 / 16,125**, leaving **4,497 unread**, **15,845 unknown**, **0 omitted**. Reading does not reduce unknown. Next is **11,629**, `Telegram/ThirdParty/tgcalls::tgcalls/platform/darwin/RTCCodecSpecificInfoH265+Private.h@a22d3594f557f7b75ff67f9427bb461b0c9ba5ce`. Production/source/release closure remains open.
