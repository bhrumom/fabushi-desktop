# tgcalls render / codec / networking / signaling — 11,693–11,756

Mapped-open. Source: tdesktop@6fed91ff -> tgcalls@1c236c09. Predecessor exact materialization: 072c76d9 / 38170930654 / 114563229488 / artifact 11714083420 / sha256 2a20ec87ef17d88a815faeceef8ebb7f8d463959f25a4dae9fb2b5bc0acf1b84.

Canonical owners: durable `call_session.rs`; shipping `production.rs`; real HTTPS `native_messaging.rs`; renderer `human-call-media.tsx`; thin Electron `call-media.ts`; backend `bhrumom/fabushi-backend@9c56a77c...`. Independent native signaling hub is not shipping proof.

11,693–11,716 map render/codec/camera/capture lifecycle into existing WebRTC/media/platform owners. 11,717–11,720 map serialization/bounded compression into existing security/network owners. 11,721–11,756 map negotiation/data-channel/ICE-DTLS-RTP/signaling/reconnect/relay into CallSession + authenticated Fabushi backend + WebRTC. Telegram MTProto/reflector/crypto mechanics are replacement-only.

Order 11,678 call-tone is implemented at ca885458 using Fabushi-owned Int16 samples, 48 kHz sample rate, finite loops, offset accounting, ringing start, transition/terminal stop, teardown, blocked-audio alert and explicit retry. unknown_closed remains false pending current-head and packaged audible/restart evidence.

Group/SFU remains blocked in production: shipping Host/backend only accept one-to-one Human calls. No mock, 1:1 path, standalone hub or uncomposed code is accepted as closure.

Accounting: read-through 11,756; unread 4,369; unknown 15,845; omitted 0; next 11,757 SignalingTranslator.h@8136fc59ff35176f06e25539ede24e64f8af7dfe.
