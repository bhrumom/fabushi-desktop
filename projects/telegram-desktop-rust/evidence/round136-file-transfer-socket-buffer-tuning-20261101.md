# Round 136 — file-transfer native socket-buffer tuning

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 / round 105  
Stage: A — implementation authored, executable validation deferred

## Authority and exact fixed-U behavior

Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Source identities:
- `Telegram/SourceFiles/mtproto/details/mtproto_abstract_socket.h@4283a353dd00db4b12044ef5a1d46cfbde7f659b`
- `Telegram/SourceFiles/mtproto/details/mtproto_tcp_socket.cpp@b3130fd706b7cbfa33f7c9f54ac5e87abe9c3e10`
- `Telegram/SourceFiles/mtproto/details/mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `Telegram/SourceFiles/mtproto/mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`
- `Telegram/SourceFiles/mtproto/facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256`
- `Telegram/SourceFiles/storage/file_upload.cpp@a19470a24b6a62b58c63d8cc21d831641d970260`
- `Telegram/SourceFiles/storage/download_manager_mtproto.cpp@d0007eb17fb0bcfbdffd89b86d8efdb900fcd363`

`kFilesSendBufferSize` and `kFilesReceiveBufferSize` are both exactly `2 * 1024 * 1024`. TcpSocket and TlsSocket request those send/receive socket options only when `protocolForFiles` is true.

The actual SessionPrivate predicate is:
`isMediaClusterDcId(_shiftedDcId) || (_realDcType == DcType::Cdn)`.
The adjacent `isUploadDcId(_shiftedDcId)` line is deliberately commented out. MediaCluster includes download, group-call-stream, export-media and updater shifts. Both upload and download shifted sessions can use file-session threads, so file-thread placement alone is not the tuning predicate.

## Existing-owner decision

- `telegram-network` already owns native TcpStream creation, direct/proxy carrier selection, TLS camouflage, framing, and transport errors.
- `telegram-protocol::DcPurpose` already distinguishes Main/Media/Cdn at that exact composition boundary.
- `telegram-media` owns scheduling/progress/pause/retry/concurrency/integrity and explicitly delegates actual I/O to adapters.
- RuntimeClient owns request/session lifetime, not socket options.

Therefore the existing telegram-network owner is extended. No file/media/network/runtime/provider/settings owner is added.

## Spec-first implementation

Platform spec checkpoint: `c284ba77b7a18477cf8900e6766787986a4c5aee`  
Platform implementation/source-compliance commit: `0ac50a1545c95681213b329e0589683cb11404a7`  
Canonical network blob: `6dece8da8f9a1512dcd17bc76fd1cc9ce43c1ef6`  
Final spec blob: `6fca16349ff56f3493854a4a6a78f37ee932a339`

Desktop production/spec mirror: `23d73b32e301408f9c8aed151cb1f9e442516e13`  
Desktop network blob: `6dece8da8f9a1512dcd17bc76fd1cc9ce43c1ef6`  
Desktop spec blob: `6fca16349ff56f3493854a4a6a78f37ee932a339`

Implementation:
1. `FILES_SEND_BUFFER_BYTES` and `FILES_RECEIVE_BUFFER_BYTES` are distinct exact 2 MiB constants.
2. `NativeSocketBufferProfile::Files` is selected only for `DcPurpose::Media | DcPurpose::Cdn`; Main selects Default.
3. Native direct and MTProto-proxy obfuscated TCP constructors receive the selected profile.
4. Native SOCKS5/HTTP forward-proxy TCP constructor receives the same profile.
5. File-profile native sockets request send and receive buffer sizes immediately after connect and before proxy/TLS/protocol bytes.
6. Native option failures return `io::Error` and are promoted by the existing `NetworkError::Io` conversion.
7. Linux/Android, Apple/BSD family, and Windows adapters are explicit with no new dependency; unsupported native targets fail closed.
8. WebProxy relay/browser stream is excluded from the native TcpStream claim.

## Authored contracts — not executed

- `file_transfer_socket_buffer_contract_tests::fixed_u_file_socket_buffer_requests_are_two_mebibytes_each`
- `file_transfer_socket_buffer_contract_tests::only_media_and_cdn_directory_purposes_select_file_socket_buffers`

No contract was built or run.

## Accounting

Responsibility: `TDRP-R9-MTPROTO-FILE-TRANSFER-SOCKET-BUFFER-001`  
State: `implemented`  
Validation: `not-run`  
Accepted: `false`

Coverage:
- `source_entries_total=16202`
- `unknown=16172`
- `unread=16145`
- `omitted=0`
- `responsibilities_total=124`
- `unmapped_responsibilities=0`

No unknown/unread decrement is claimed because this is an exact responsibility isolated from already-reviewed adjacent source families, not a claim of whole-file source-entry closure.

## Stage-A execution statement

No GitHub Actions/check/status/workflow query or trigger was performed. No build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deploy, install or runtime acceptance was executed.

Stage-B executable validation and broader fixed-U source/implementation closure remain open.
