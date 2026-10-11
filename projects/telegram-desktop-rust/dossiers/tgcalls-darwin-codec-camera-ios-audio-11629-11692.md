# tgcalls Darwin codec / camera / iOS audio responsibilities — orders 11,629–11,692 complete read

Exact source-window evidence: parent HEAD `9e19096129b9e5d9c87ff91e9a0645544929d155`; Rust desktop runtime `38169448756` completed/success; Desktop Chat Parity `38169449053` completed/success; source-authority job `114558772727` success; rust-host `114558772656` success; artifact `11713142724`, digest `sha256:e9667b498b5ab9ce45e0e73e81c6c8665866f7a739ef745f477aa175fa59720f`. The helper materialized all 64 exact repository/commit/path/Git-blob identities and Revision 9 validation was green before this descendant accounting change.

## Responsibility decomposition

This window covers Darwin H.265 metadata, serialized queue affinity, macOS CoreMediaIO capture, CVPixelBuffer ownership, H.264/H.265 encoder/decoder factories and VideoToolbox lifecycle, camera pause/active/fatal signaling, Metal/sample-buffer rendering, x264/H.265 packetization, then iOS-only audio-session/audio-device/voice-processing internals. Existing-owner-first maps macOS behavior into canonical CallSession, Permissions, Resource, browser WebRTC and the thinnest platform adapter. No native Telegram/tgcalls runtime, FFI codec sidecar or second Call/Resource store is introduced.

The concrete production delta is camera capture lifecycle parity: the canonical camera MediaStreamTrack now projects mute/unmute as `videoPaused`/`videoActive`, fences callbacks by capture generation plus the still-owned local stream/track, and on unexpected `ended` makes one preferred-device reacquire attempt before declaring capture failed. Successful recovery replaces the current video sender only when screen sharing is not the active source and rebinds lifecycle fencing; failed recovery removes the dead track, projects video inactive, reports the failure, and does not loop forever. Call teardown invalidates the capture generation before stopping streams. This mirrors the source fatal-recovery responsibility without copying Darwin capturer classes.

Orders 11,678–11,692 are individually dispositioned iOS-only implementation files, not a directory-level exclusion. Their interruption, route change, audio-session lock, AudioUnit retry/state and deterministic teardown semantics are recorded as platform boundary evidence but are not Desktop shipping code. H.264/H.265/group codec transport remains open where the verified service is only two-party Human Call; this window does not use 1:1 signaling evidence to claim group/SFU closure.

Concurrency/lifecycle/security: queue-bound source/render state must remain serialized and callbacks generation/source fenced; pixel buffers and hardware codec sessions have explicit ownership and teardown; user camera access remains behind canonical permission/media bridges; renderer never owns TURN credentials or server secrets. iOS AudioUnit/session code is not shipped.

## Per-file traceability

| Order | Exact path | Representative symbols | Disposition | Canonical owner |
|---:|---|---|---|---|
| 11629 | `tgcalls/platform/darwin/RTCCodecSpecificInfoH265+Private.h` | `RTCCodecSpecificInfoH265 private bridge` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC codec negotiation / platform adapter |
| 11630 | `tgcalls/platform/darwin/RTCCodecSpecificInfoH265.h` | `RTCCodecSpecificInfoH265` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC codec negotiation / platform adapter |
| 11631 | `tgcalls/platform/darwin/RTCCodecSpecificInfoH265.mm` | `RTCCodecSpecificInfoH265 implementation` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC codec negotiation / platform adapter |
| 11632 | `tgcalls/platform/darwin/RTCH265ProfileLevelId.h` | `RTCH265ProfileLevelId` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC codec negotiation / platform adapter |
| 11633 | `tgcalls/platform/darwin/RTCH265ProfileLevelId.mm` | `RTCH265ProfileLevelId implementation` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC codec negotiation / platform adapter |
| 11634 | `tgcalls/platform/darwin/SQueue.h` | `SQueue` | `mapped-open-canonical-platform-adapter-replacement` | Platform concurrency / CallSession media |
| 11635 | `tgcalls/platform/darwin/SQueue.m` | `SQueue dispatch implementation` | `mapped-open-canonical-platform-adapter-replacement` | Platform concurrency / CallSession media |
| 11636 | `tgcalls/platform/darwin/SQueueLocalObject.h` | `SQueueLocalObject` | `mapped-open-canonical-platform-adapter-replacement` | Platform concurrency / CallSession media |
| 11637 | `tgcalls/platform/darwin/SQueueLocalObject.m` | `SQueueLocalObject lifecycle` | `mapped-open-canonical-platform-adapter-replacement` | Platform concurrency / CallSession media |
| 11638 | `tgcalls/platform/darwin/TGCMIOCapturer.h` | `TGCMIOCapturer` | `mapped-open-canonical-platform-adapter-replacement` | Permissions / CallSession / Computer capture / platform adapter |
| 11639 | `tgcalls/platform/darwin/TGCMIOCapturer.m` | `TGCMIOCapturer implementation` | `mapped-open-canonical-platform-adapter-replacement` | Permissions / CallSession / Computer capture / platform adapter |
| 11640 | `tgcalls/platform/darwin/TGCMIODevice.h` | `TGCMIODevice` | `mapped-open-canonical-platform-adapter-replacement` | Permissions / CallSession / Computer capture / platform adapter |
| 11641 | `tgcalls/platform/darwin/TGCMIODevice.mm` | `TGCMIODevice start/stop/device enumeration` | `mapped-open-canonical-platform-adapter-replacement` | Permissions / CallSession / Computer capture / platform adapter |
| 11642 | `tgcalls/platform/darwin/TGRTCCVPixelBuffer.h` | `TGRTCCVPixelBuffer` | `mapped-open-canonical-platform-adapter-replacement` | Permissions / CallSession / Computer capture / platform adapter |
| 11643 | `tgcalls/platform/darwin/TGRTCCVPixelBuffer.mm` | `TGRTCCVPixelBuffer implementation` | `mapped-open-canonical-platform-adapter-replacement` | Permissions / CallSession / Computer capture / platform adapter |
| 11644 | `tgcalls/platform/darwin/TGRTCDefaultVideoDecoderFactory.h` | `TGRTCDefaultVideoDecoderFactory` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11645 | `tgcalls/platform/darwin/TGRTCDefaultVideoDecoderFactory.mm` | `supportedCodecs/createDecoder` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11646 | `tgcalls/platform/darwin/TGRTCDefaultVideoEncoderFactory.h` | `TGRTCDefaultVideoEncoderFactory` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11647 | `tgcalls/platform/darwin/TGRTCDefaultVideoEncoderFactory.mm` | `supportedCodecs/createEncoder` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11648 | `tgcalls/platform/darwin/TGRTCVideoDecoderH264.h` | `TGRTCVideoDecoderH264` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11649 | `tgcalls/platform/darwin/TGRTCVideoDecoderH264.mm` | `VideoToolbox H264 decode lifecycle` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11650 | `tgcalls/platform/darwin/TGRTCVideoDecoderH265.h` | `TGRTCVideoDecoderH265` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11651 | `tgcalls/platform/darwin/TGRTCVideoDecoderH265.mm` | `VideoToolbox H265 decode lifecycle` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11652 | `tgcalls/platform/darwin/TGRTCVideoEncoderH264.h` | `TGRTCVideoEncoderH264` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11653 | `tgcalls/platform/darwin/TGRTCVideoEncoderH264.mm` | `H264 encoder session/bitrate/keyframe lifecycle` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11654 | `tgcalls/platform/darwin/TGRTCVideoEncoderH265.h` | `TGRTCVideoEncoderH265` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11655 | `tgcalls/platform/darwin/TGRTCVideoEncoderH265.mm` | `H265 encoder session/bitrate/keyframe lifecycle` | `mapped-open-canonical-platform-adapter-replacement` | CallSession / WebRTC video codec / platform adapter |
| 11656 | `tgcalls/platform/darwin/VideoCMIOCapture.h` | `VideoCMIOCapture` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11657 | `tgcalls/platform/darwin/VideoCMIOCapture.mm` | `VideoCMIOCapture start/stop/frame callback` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11658 | `tgcalls/platform/darwin/VideoCameraCapturer.h` | `VideoCameraCapturer` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11659 | `tgcalls/platform/darwin/VideoCameraCapturer.mm` | `VideoCameraCapturer state/pause/active/fatal` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11660 | `tgcalls/platform/darwin/VideoCameraCapturerMac.h` | `VideoCameraCapturerMac` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11661 | `tgcalls/platform/darwin/VideoCameraCapturerMac.mm` | `VideoCameraCapturerMac state/pause/active/fatal` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11662 | `tgcalls/platform/darwin/VideoCaptureView.h` | `VideoCaptureView` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11663 | `tgcalls/platform/darwin/VideoCaptureView.mm` | `VideoCaptureView frame/view lifecycle` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11664 | `tgcalls/platform/darwin/VideoCapturerInterfaceImpl.h` | `VideoCapturerInterfaceImpl` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11665 | `tgcalls/platform/darwin/VideoCapturerInterfaceImpl.mm` | `VideoCapturerInterfaceImpl source/device/state lifecycle` | `applicable-human-call-core-presentation-production-partial` | CallSession / Permissions / Resource / platform adapter |
| 11666 | `tgcalls/platform/darwin/VideoMetalView.h` | `VideoMetalView` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11667 | `tgcalls/platform/darwin/VideoMetalView.mm` | `VideoRendererAdapterImpl, VideoMetalView` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11668 | `tgcalls/platform/darwin/VideoMetalViewMac.h` | `VideoMetalViewMac` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11669 | `tgcalls/platform/darwin/VideoMetalViewMac.mm` | `VideoRendererAdapterImpl, VideoMetalView macOS` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11670 | `tgcalls/platform/darwin/VideoSampleBufferView.h` | `VideoSampleBufferView` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11671 | `tgcalls/platform/darwin/VideoSampleBufferView.mm` | `VideoRendererAdapterImpl, CopyVideoFrameToNV12PixelBuffer` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11672 | `tgcalls/platform/darwin/VideoSampleBufferViewMac.h` | `VideoSampleBufferViewMac` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11673 | `tgcalls/platform/darwin/VideoSampleBufferViewMac.mm` | `VideoRendererAdapterImpl, VideoSampleBufferView macOS` | `mapped-open-canonical-platform-adapter-replacement` | Transcript/CallSession media presentation / platform adapter |
| 11674 | `tgcalls/platform/darwin/h264_encoder_impl.cpp` | `H264EncoderX264Impl, NumberOfThreads` | `applicable-group-video-call-production-gap` | CallSession / WebRTC media-network |
| 11675 | `tgcalls/platform/darwin/h264_encoder_impl.h` | `H264EncoderX264Impl` | `applicable-group-video-call-production-gap` | CallSession / WebRTC media-network |
| 11676 | `tgcalls/platform/darwin/h265_nalu_rewriter.cc` | `H265AnnexBBufferReader, H265AvccBufferWriter` | `applicable-group-video-call-production-gap` | CallSession / WebRTC media-network |
| 11677 | `tgcalls/platform/darwin/h265_nalu_rewriter.h` | `H265 NALU rewrite API` | `applicable-group-video-call-production-gap` | CallSession / WebRTC media-network |
| 11678 | `tgcalls/platform/darwin/iOS/CallAudioTone.h` | `CallAudioTone` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11679 | `tgcalls/platform/darwin/iOS/RTCAudioSession+Configuration.mm` | `RTCAudioSession configuration category` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11680 | `tgcalls/platform/darwin/iOS/RTCAudioSession+Private.h` | `RTCAudioSession private contract` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11681 | `tgcalls/platform/darwin/iOS/RTCAudioSession.h` | `RTCAudioSession` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11682 | `tgcalls/platform/darwin/iOS/RTCAudioSession.mm` | `RTCAudioSession singleton/delegate/lock lifecycle` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11683 | `tgcalls/platform/darwin/iOS/RTCAudioSessionConfiguration.h` | `RTCAudioSessionConfiguration` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11684 | `tgcalls/platform/darwin/iOS/RTCAudioSessionConfiguration.mm` | `RTCAudioSessionConfiguration defaults` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11685 | `tgcalls/platform/darwin/iOS/RTCNativeAudioSessionDelegateAdapter.h` | `RTCNativeAudioSessionDelegateAdapter` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11686 | `tgcalls/platform/darwin/iOS/RTCNativeAudioSessionDelegateAdapter.mm` | `RTCNativeAudioSessionDelegateAdapter implementation` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11687 | `tgcalls/platform/darwin/iOS/tgcalls_audio_device_ios.h` | `AudioDeviceIOS` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11688 | `tgcalls/platform/darwin/iOS/tgcalls_audio_device_ios.mm` | `AudioDeviceIOS interruption/route/play-record lifecycle` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11689 | `tgcalls/platform/darwin/iOS/tgcalls_audio_device_module_ios.h` | `AudioDeviceModuleIOS` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11690 | `tgcalls/platform/darwin/iOS/tgcalls_audio_device_module_ios.mm` | `CreateAudioDeviceModule, AudioDeviceModuleIOS` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11691 | `tgcalls/platform/darwin/iOS/tgcalls_voice_processing_audio_unit.h` | `VoiceProcessingAudioUnit, State` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |
| 11692 | `tgcalls/platform/darwin/iOS/tgcalls_voice_processing_audio_unit.mm` | `VoiceProcessingAudioUnit initialize/start/stop/retry callbacks` | `development-only-non-applicable-to-shipping-product` | Platform boundary evidence only |

## Accounting

Exact blob read/decomposition advances to **11,692 / 16,125**, leaving **4,433 unread**, **15,845 unknown**, **0 omitted**. Reading does not reduce unknown. Next is **11,693**, `Telegram/ThirdParty/tgcalls::tgcalls/platform/darwin/macOS/TGRTCMTLI420Renderer.h@ceb4c410a5a890583f649ca87aaf4b6affef6ced`. Source/product/release closure remains open.
