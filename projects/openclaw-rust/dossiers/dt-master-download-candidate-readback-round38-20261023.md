# Round 38 — Historical DefinitelyTyped master download path, 93 candidate Blob readbacks, and Sand external-contract bounds

Task: `3dd43a37-9f36-4b7a-adac-dfcc87584523`; round 38; audit date 2026-10-23. This is a **new read-only evidence dossier**, not a production change, test result, source closure, or acceptance. Prior dossier: `dt-fs-selected-candidates-round37-20261023.md` at audit branch carrier `0151fe753f817423ab6d3fc3dd1abb2559f72b61`.

Subject PR: [bhrumom/fabushi-desktop#169](https://github.com/bhrumom/fabushi-desktop/pull/169) **open, unmerged**, HEAD `b5db35fa272c83833685d70a730b5f8285fa1176`; live main `7394f6dc7687f9b986ee5589d40aa680ce23b90a` (tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`). Frozen [OpenClaw source](https://github.com/openclaw/openclaw/commit/ad8d12f126d29b0db204187adf5092db80294e70) commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. These identities do not mean the PR is current main, and historical green jobs do not verify this new carrier.

## New closure of historical download-to-FS call path

At both historical microsoft/DefinitelyTyped-tools publisher commits [8b9e12c](https://github.com/microsoft/DefinitelyTyped-tools/commit/8b9e12cd2b5680c748838204995618eeb3e3b58a) and [f746498](https://github.com/microsoft/DefinitelyTyped-tools/commit/f746498624d9c12ff5191309fe28a429eef458d5), the exact code Blob identities are **identical**:
- `packages/definitions-parser/src/get-definitely-typed.ts@8345bf70e94ef12838226e29a18d1aa5db980285`: `getDefinitelyTyped(options, log)`, when `definitelyTypedPath === undefined`, creates the data directory and returns `downloadAndExtractFile(definitelyTypedZipUrl, log)` (lines 20–25); when a local path is provided it instead checks clean `git diff --name-only` and returns `DiskFS` (27–38).
- `packages/definitions-parser/src/lib/settings.ts@fbd76917658c79fdcde693eff277cc5716b50e54`: `sourceBranch = "master"` and `definitelyTypedZipUrl = https://codeload.github.com/DefinitelyTyped/DefinitelyTyped/tar.gz/master`; **mutable URL, not an immutable commit archive**.
- `packages/utils/src/index.ts@e81d16213148ec05a41ddcee9879e92e98099f7c` exports `./io` and `./fs`. `packages/utils/src/io.ts@7b8d374de9632bb5620aa91a22c6832fd834a030`: `downloadAndExtractFile()` performs HTTPS GET, accepts status 200, gunzips into `tarStream.extract()`, strips `DefinitelyTyped-master/`, collects file entry string content into `Dir`, skips directory entry bodies, and **on extract finish resolves `new InMemoryFS(root.finish(), "/")`** (lines 175–233). There is no exact commit/digest capture or archive response identity in this function.
- `packages/utils/src/fs.ts@6b8e87ec19d9b164d89b0c417749c0d902324cd8`: `Dir.finish()` recursively sorts the per-directory map keys (69–75), `InMemoryFS.readdir()` returns those keys in insertion order (147–149); alternative `DiskFS.readdir()` explicitly sorts (190–195). Therefore extraction order is canonicalized *before* `getFiles` traverses paths in these historical publisher commits.
- `packages/definitions-parser/src/lib/definition-parser.ts@1ffd0766ae5bef7e99ddf7e3a16cfb3ff03b1bc5` `getFiles` recurses `FS.readdir()`, includes matching declarations, and excludes root older-version folders (261–300). `packages/utils/src/miscellany.ts@bbbda8e40cbea1963195ce78ac0ee31ad34abc37` uses `**/*.d.{ts,cts,mts,*.ts}` matcher and normalizes line endings for SHA256. `packages/definitions-parser/src/packages.ts@36399d9b3619980982a4a62522bc79a7f943d19b` `getContentHash()` includes ordered `getFiles()` plus normalized `package.json` (546–551). **This is not npm tarball SHA512.**

Official successful historical publisher [pg run 32063758027/job 95490787861](https://github.com/microsoft/DefinitelyTyped-tools/actions/runs/32063758027) log requested mutable master at **2026-08-17 20:04:47 UTC**, finished at 20:04:55 UTC, and logged `Need to publish: @types/pg@8.23.1`; publisher `head_sha=8b9e12c...`. [node run 37974941076/job 113970739568](https://github.com/microsoft/DefinitelyTyped-tools/actions/runs/37974941076) requested master at **2026-10-09 18:41:56 UTC**, finished at 18:42:04 UTC and logged `Need to publish: @types/node@26.6.5`; publisher `head_sha=f746498...`. No observed log line binds the downloaded tar response to DefinitelyTyped commit/tree or archive digest. The later `commits?path=types/pg` / `types/node` request cannot retroactively attest the earlier archive response. **Historical downloaded master identity remains unproven (0/2); neither published typesPublisherContentHash has been independently reproduced.**

## Immutable candidate source evidence, not historic download proof

- [DefinitelyTyped pg candidate](https://github.com/DefinitelyTyped/DefinitelyTyped/commit/97ba786e647c0899a2dd6d1b6807896762e725fa): commit `97ba786e647c0899a2dd6d1b6807896762e725fa`, tree `022e9d61402858052bb76a773c093249f50435a3`, timestamp 2026-08-17 19:46:36 UTC, predates pg download. Latest declarations **4/4** read using GitHub Contents path/ref and Git Blob SHA; for all 4, independently fetched `fetch_blob` UTF-8 text equaled path content and path SHA matched the frozen tree.
- [DefinitelyTyped node candidate](https://github.com/DefinitelyTyped/DefinitelyTyped/commit/a92efe8ec26d5931fbefff5e6daa5d6b2cc1fd06): commit `a92efe8ec26d5931fbefff5e6daa5d6b2cc1fd06`, tree `6d321f29b7d4e42f314d21d28d81b18855546177`, timestamp 2026-10-09 18:38:54 UTC, predates node download. Root has old `v22/v24/v25` folders excluded from latest by parser rule, but `ts5.6` and `ts5.7` are **included** (4+2 declarations). Re-read all 17 other child subtrees with GitHub tree API; none truncated. Subtree matched declarations: .vscode:0, assert:1, dns:1, fs:1, inspector:1, node-tests:0, path:2, readline:1, scripts:0, stream:4, test:1, timers:1, ts5.6:4, ts5.7:2, util:1, web-globals:16, zlib:1. Total **89/89** latest candidate declaration paths retrieved individually by GitHub Contents at immutable ref; all returned Blob SHAs match frozen tree paths. First two node files `assert.d.ts` and `assert/strict.d.ts` were also fetched separately by Blob SHA and compared text-identically. A short manually typed expected SHA for `readline.d.ts` lacked the last `b` character; the actual `a9ad29df4cf698a6928ac6f79363d20984059a5b` matches the tree and is **not a source mismatch**.
- All 93 names are eligible *candidate path identities*, not verified members of the historical publisher's mutable downloaded master response. Git file SHA agreement is not equivalent to an independently witnessed tarball commit binding. **Published @types/pg@8.23.1 typesPublisherContentHash `4c3006364fd06c962ce8e915a3dcf8d0d87bfec39f509aa95b270ad620cad104` and @types/node@26.6.5 `3196021f34db9bcb68a729bd29ff60434d3b4c5f49619270db25e7dbd6dc651c` remain unverified 0/2.**

### Complete selected candidate path → immutable Blob SHA readback (93 rows)

| Family | Candidate path | Git Blob SHA | This round read method |
| --- | --- | --- | --- |
| pg | `types/pg/index.d.mts` | `a56545bab04faafe8dccdc59c0f0847aa1e101b5` | Contents+Blob equality |
| pg | `types/pg/index.d.ts` | `9700c723c78a9ffc211445b012896962a7338950` | Contents+Blob equality |
| pg | `types/pg/lib/connection-parameters.d.ts` | `951d48139a6eeab891e83753792000a34092edd3` | Contents+Blob equality |
| pg | `types/pg/lib/type-overrides.d.ts` | `972b5a984c3c6f064c76f84dc9723ee44d71dde8` | Contents+Blob equality |
| node | `types/node/assert.d.ts` | `72cc91de95e987a304bfcd4221d898b653a0f2e9` | Contents path+SHA |
| node | `types/node/assert/strict.d.ts` | `d4ea39083f7259f7db129b55a543ea9115e1a397` | Contents path+SHA |
| node | `types/node/async_hooks.d.ts` | `d38fcf096e45b9d3a7426fc9df6e0e6b26583d20` | Contents path+SHA |
| node | `types/node/buffer.buffer.d.ts` | `47f1fac97132f6e7f4cb10d2b46447dfb669795a` | Contents path+SHA |
| node | `types/node/buffer.d.ts` | `59f8a4532982d7ac124ab3fc519131b38665367a` | Contents path+SHA |
| node | `types/node/child_process.d.ts` | `f89e415241dc5db410d2f8eb0bdcadf1ab5aee9b` | Contents path+SHA |
| node | `types/node/cluster.d.ts` | `b9dbc8c110099c67c41192a355f82be8844b8505` | Contents path+SHA |
| node | `types/node/console.d.ts` | `5ba70b3f4b322dfdd252d6a215eb0a0350058856` | Contents path+SHA |
| node | `types/node/constants.d.ts` | `a271f9aaad19442e9a9b8c187cd861af485e8b1d` | Contents path+SHA |
| node | `types/node/crypto.d.ts` | `c628382fb3022fccf02e3ae1bfe7fbc5a340b0fb` | Contents path+SHA |
| node | `types/node/dgram.d.ts` | `ca8586a3af8780dc03a9b4928d6f2a8d4c250a92` | Contents path+SHA |
| node | `types/node/diagnostics_channel.d.ts` | `3dee66a1f7421e0d0962946b6e8968d1a68cfed2` | Contents path+SHA |
| node | `types/node/dns.d.ts` | `319ee9696ab5ca81c7c1f5ef921444e03499d184` | Contents path+SHA |
| node | `types/node/dns/promises.d.ts` | `9e7e1c9a862ab7528e57fbaba4af9d7c562a2f7e` | Contents path+SHA |
| node | `types/node/domain.d.ts` | `8e03a7d6db6ea23480d6ae3b1962d39e30c2bf1d` | Contents path+SHA |
| node | `types/node/events.d.ts` | `3191046e5298b33d8d9098978480e92037c1e08b` | Contents path+SHA |
| node | `types/node/ffi.d.ts` | `6b9932023882c6a945a09b0492a85609ab652806` | Contents path+SHA |
| node | `types/node/fs.d.ts` | `e2626e67e2bc003fb4059781675a0e7034d678ee` | Contents path+SHA |
| node | `types/node/fs/promises.d.ts` | `9366b971ab7f77fd7a49b11a135da985d80c7e8b` | Contents path+SHA |
| node | `types/node/globals.d.ts` | `0c2f0b09ebbea9dd382666a0fbf79aad4842ca5e` | Contents path+SHA |
| node | `types/node/globals.typedarray.d.ts` | `e69dd0cd879a924ea177d4142614513d6d904057` | Contents path+SHA |
| node | `types/node/http.d.ts` | `73a7024da159eac7eb26f248b52b7df869290b07` | Contents path+SHA |
| node | `types/node/http2.d.ts` | `1899b0ef6e9f6a2f8b041d5d12e64f33e9928139` | Contents path+SHA |
| node | `types/node/https.d.ts` | `b10aad0c28c5744f64ede6a2f7bfdb35ee7cbcd7` | Contents path+SHA |
| node | `types/node/index.d.ts` | `c8cdf6bd97f16b01741661df110b8972402b5565` | Contents path+SHA |
| node | `types/node/inspector.d.ts` | `0dd4ca9a44a01423149df486a9a74141b8ec1787` | Contents path+SHA |
| node | `types/node/inspector.generated.d.ts` | `0e2e4416daaf5efd3142dde814d2734acf16d19c` | Contents path+SHA |
| node | `types/node/inspector/promises.d.ts` | `e75ff20a3549576182e767383b3c7a8ce74fc43c` | Contents path+SHA |
| node | `types/node/module.d.ts` | `792a0bd5def12861af3bfd016f5a26c5fddaf137` | Contents path+SHA |
| node | `types/node/net.d.ts` | `3f104c5835eca13728bac0836368232f53ab730e` | Contents path+SHA |
| node | `types/node/os.d.ts` | `751d4b41e8f656e5b51b3de576847d08e383f17d` | Contents path+SHA |
| node | `types/node/path.d.ts` | `ae8fd973fd96a6cb8503f7066ea32dc65ee84635` | Contents path+SHA |
| node | `types/node/path/posix.d.ts` | `d60f629fb7775140b210b3c1c68a668ae8e7feb9` | Contents path+SHA |
| node | `types/node/path/win32.d.ts` | `e6aa9fa610c86c7af3ad3be20025f7578c1cd871` | Contents path+SHA |
| node | `types/node/perf_hooks.d.ts` | `e4bea8f3bf616706ed712ea10c57155570602dc5` | Contents path+SHA |
| node | `types/node/process.d.ts` | `51a3d6172b0ec76603e6c8ee87357fd8cf2599d3` | Contents path+SHA |
| node | `types/node/punycode.d.ts` | `87fdec970362b006d0cff41f52ea51f807227aa1` | Contents path+SHA |
| node | `types/node/querystring.d.ts` | `6d821a4e06be5881a726d1c891ac7cc1b7ef3db4` | Contents path+SHA |
| node | `types/node/quic.d.ts` | `e48ea2757d00508b146e2b7491e74c6085d2a190` | Contents path+SHA |
| node | `types/node/readline.d.ts` | `a9ad29df4cf698a6928ac6f79363d20984059a5b` | Contents path+SHA |
| node | `types/node/readline/promises.d.ts` | `ccee49037635bf3776aefa479b0ccebb83148370` | Contents path+SHA |
| node | `types/node/repl.d.ts` | `33e2fe97a2d695dfe1289bc5492bf375c6b44e46` | Contents path+SHA |
| node | `types/node/sea.d.ts` | `85ab108b8869683cfba65b7f4092dded0866e204` | Contents path+SHA |
| node | `types/node/sqlite.d.ts` | `70493f065c5175d55b19b284b5398802421b5d36` | Contents path+SHA |
| node | `types/node/stream.d.ts` | `c9b0811e86479ac5529c9c8bf2c33f25bc0fe306` | Contents path+SHA |
| node | `types/node/stream/consumers.d.ts` | `27a474c5e18b8ca161c77c0ca80edcd942e8c2f6` | Contents path+SHA |
| node | `types/node/stream/iter.d.ts` | `f42fb5276b000352c2e7ae3a55de7279ded18d13` | Contents path+SHA |
| node | `types/node/stream/promises.d.ts` | `c57fe1e3416afd19286b7758ad0bd25bd926c2e3` | Contents path+SHA |
| node | `types/node/stream/web.d.ts` | `e3a61252756bb062e524a9e1d3ebea3ed189551e` | Contents path+SHA |
| node | `types/node/string_decoder.d.ts` | `568e3f654fad09753fca86f4f5688df6c064228a` | Contents path+SHA |
| node | `types/node/test.d.ts` | `22acc430b6a6c7a13bc035fd22d12700293f516b` | Contents path+SHA |
| node | `types/node/test/reporters.d.ts` | `36ca28fc8dc1daa6323d50ab3df9a4784d139047` | Contents path+SHA |
| node | `types/node/timers.d.ts` | `5b74b894f8ae0903901e5dd639684da783eba569` | Contents path+SHA |
| node | `types/node/timers/promises.d.ts` | `8e178eafe6cb61a719ae77b3cbea00281e1001e6` | Contents path+SHA |
| node | `types/node/tls.d.ts` | `773a69f0352e1b5912145e8b0560975e1e70ad4b` | Contents path+SHA |
| node | `types/node/trace_events.d.ts` | `5fa7a3957a9884d90280915eb6862a313118d51f` | Contents path+SHA |
| node | `types/node/ts5.6/buffer.buffer.d.ts` | `626ec28c1eb70a726be5aaff882939fc85f4502f` | Contents path+SHA |
| node | `types/node/ts5.6/compatibility/float16array.d.ts` | `f148cc4f85b83107f8e8bc4fe3fcf6b642b294fe` | Contents path+SHA |
| node | `types/node/ts5.6/globals.typedarray.d.ts` | `57a1ab4f2c93361e8694aab8527b67862f667a15` | Contents path+SHA |
| node | `types/node/ts5.6/index.d.ts` | `6d093bd318d9a8bcc279eb9d518492607fa656ed` | Contents path+SHA |
| node | `types/node/ts5.7/compatibility/float16array.d.ts` | `110b1ebb19cb26cfac04deff6f9280ce796089d5` | Contents path+SHA |
| node | `types/node/ts5.7/index.d.ts` | `a9ee9e95a44a89a2cb7ae0d0dcc15ca34edb1b9b` | Contents path+SHA |
| node | `types/node/tty.d.ts` | `0569bef5b498a8a6b0d3362aa2852738415239a7` | Contents path+SHA |
| node | `types/node/url.d.ts` | `d6ea8c59e7faf4072ba7488fe03c31899dbc8a18` | Contents path+SHA |
| node | `types/node/util.d.ts` | `d756206d9b505ffc421592b49c52e595387972b1` | Contents path+SHA |
| node | `types/node/util/types.d.ts` | `174c0fa0e818085bfa66dca8939f035f235184df` | Contents path+SHA |
| node | `types/node/v8.d.ts` | `ed5c7b18e2832034f721dc5b370a8cd50db1a9cf` | Contents path+SHA |
| node | `types/node/vfs.d.ts` | `1bcae3300e2b3ff4e557b7ceb3e175857fe1f34d` | Contents path+SHA |
| node | `types/node/vm.d.ts` | `526c31160546913f4062e598d863fd584a41cd0b` | Contents path+SHA |
| node | `types/node/wasi.d.ts` | `a0d56cf89643f2306e1ba744352215c91b53f618` | Contents path+SHA |
| node | `types/node/web-globals/abortcontroller.d.ts` | `ad753c16ac0617a51913d24081e8f15a5c0121ed` | Contents path+SHA |
| node | `types/node/web-globals/blob.d.ts` | `04ff440aa0ab69667494064bc589006419449e33` | Contents path+SHA |
| node | `types/node/web-globals/console.d.ts` | `5492de33e4d6c2f0d5b76cfa9d0b1b13a92251a8` | Contents path+SHA |
| node | `types/node/web-globals/crypto.d.ts` | `f038a439203b73cdcc942842b1542af2fa5d78c5` | Contents path+SHA |
| node | `types/node/web-globals/domexception.d.ts` | `1908bd45284667626586c8327767b75e9c49f56b` | Contents path+SHA |
| node | `types/node/web-globals/encoding.d.ts` | `de5fa4f7bdb54b70aaf1232a9318341f2c907558` | Contents path+SHA |
| node | `types/node/web-globals/events.d.ts` | `cdbcc690cdf17c1a696b8625401231c8c77ef239` | Contents path+SHA |
| node | `types/node/web-globals/fetch.d.ts` | `0aa958833f288875dc4717f48878ccdf87a7cd80` | Contents path+SHA |
| node | `types/node/web-globals/importmeta.d.ts` | `33deba14a7662d4f19f25851cc167f26c118bac2` | Contents path+SHA |
| node | `types/node/web-globals/messaging.d.ts` | `c914582b98ee57ecd87856feeec5c8ff1c324ad4` | Contents path+SHA |
| node | `types/node/web-globals/navigator.d.ts` | `9b45b2678ade745b6ac3f70739467d4c65752ea8` | Contents path+SHA |
| node | `types/node/web-globals/performance.d.ts` | `b8f4e627a82a99b9929142279cc72612fc412911` | Contents path+SHA |
| node | `types/node/web-globals/storage.d.ts` | `fd61cfc83bd81851e06d63f0a96c9b0226dae775` | Contents path+SHA |
| node | `types/node/web-globals/streams.d.ts` | `96d2868d72c0d5e550cf66918b294d608b977baf` | Contents path+SHA |
| node | `types/node/web-globals/timers.d.ts` | `9f84a3e9ed985b8c66481e79edb8a601a9aff91a` | Contents path+SHA |
| node | `types/node/web-globals/url.d.ts` | `d30208ba1ba900966875d5ed0e6caba06f033bef` | Contents path+SHA |
| node | `types/node/worker_threads.d.ts` | `f7977aedfae47c97b9e9754e39bc344918e2db17` | Contents path+SHA |
| node | `types/node/zlib.d.ts` | `ddfbfd96e520e3ebdecf8ab04609acbba6e7a606` | Contents path+SHA |
| node | `types/node/zlib/iter.d.ts` | `085bea541ab94a3f77713885ad4ce142877cbe91` | Contents path+SHA |

## npm exact-version published-byte evidence

Actual read-only attempts: `https://registry.npmjs.org/@types%2Fpg/8.23.1`, `https://registry.npmjs.org/@types%2Fnode/26.6.5`, `https://registry.npmjs.org/pg/8.16.3` were inaccessible to the available web source, and the exact-version npmjs.com pages returned 403. GitHub connector does not permit registry.npmjs.org hosts. npm's general signing documentation does not supply these exact versions. No official exact-version `dist.integrity` or tarball bytes were independently obtained: **0/14 Registry exact-version evidence; 0/14 independently hashed npm published tarballs; 0/14 published archive LICENSE/NOTICE inspections**. Do not promote a pnpm-lock hash or GitHub source archive into an independent npm publication receipt. Other official package endpoints were not claimed checked.

## Sand service owner / lifecycle: source-observed boundaries and unproven contract

Actual Fabushi candidate source Blobs (GitHub read-only, PR subject scope):
- `backend_transport.rs@6dc4477c72a38179d71ab33013c9b7e2c3b89991` lines 65–101 obtains `HostAuthExtension::get_access_token` **per POST**, sends bearer and Sand client headers, rejects non-2xx; **even HTTP 204 enters `response.json::<Value>()`** and thus may be a decode error if response body is empty. This is a source-level transport concern, not demonstrated deployed behavior.
- `sand_automation_fire_consumer.rs@b38b1614a95a34bbfe34acff39b8c3cdfaae943e` lines 135–163 maintains process-local `BTreeMap<runUuid, Running|Completed|Reported>`; `stop()` does not persist or generation-fence completion. Lines 187–207 and 336–371 post completion and classify HTTP 404 or 409 as reported/settled. Lines 267–309 ACK reported UUIDs on subsequent `/sand/automation-events/poll`, then dispatch unseen UUIDs. It is not proof of remote service's correct 404/409 semantics.
- `automation_store.rs@8ffd2175c7afb65611f65a1d56f3368b79881220` `FileAutomationStore.begin_run()` (line 43) returns an existing persisted UUID run unchanged if found, including `status=running`; `automation_run_path.rs@7bedae399879f7269cfded2c5bf629ad9204a0b1` lines 198–231 calls `begin_run`, takes its ID, then calls execution callback **without checking previously persisted run status**; 240–275 finishes. This establishes a concrete crash/replay *risk path* on redelivery, not a measured duplicate side effect.
- `production.rs@3927e9736bebd27302cd9c4c2139b583e80b7aa4` lines 181–201 couples dispatch completion to the process-local fire state. The formal account-generation fence and the first-party deployed server owner remain unverified.

New **comparison-only / non-authoritative** source: public [Spaire-Tech/Claidor @ 87081b78](https://github.com/Spaire-Tech/Claidor/commit/87081b78e76353b787ddc8b24d6b1007315c1882) contains `server/simeon/sand/listeners_relay.py@3003b3535687c7155b936f34887052ebe5fd8233` lines 110–134: `POST /sand/automation-runs/complete` obtains `get_desktop_or_box_session`, validates `runUuid` and `status=succeeded|failed` (else 400), calls `listeners.complete_fire(session, caller.user_id, ...)`, returns 404 if run missing and 200 JSON `{}` otherwise. Its `listeners_service.py@05ece3c5b4c2d8fc67c1f93dc8fafd5d5803f152` lines 468–532 ACKs/list-polls by user, and returns success for an already `completed`/`acked` UUID; its `server/tests/sand/test_listeners.py@406e412ab071dae34ef838e85bb9a618e40f7ad1` checks completed run 200, subsequent ACK disappearance, missing UUID 404. Auth code `server/simeon/desktop/auth.py@6554a4a28f2143135b269250687b85087aa9929b` checks a live bearer session. **Claidor is a separate public repository/product; there is NO proof its implementation is Fabushi's currently deployed authoritative Sand service. Treat this only as a hypothesis for comparison and oracle design.** Its route does not affirm formal 204 or 409 semantics for Fabushi, and does not settle revoked-session, account-generation or lost-response production contracts.

Unexecuted falsifiers remain: `AUTO-RUN-AUTH-LATE-01`: fire under account A, suspend its completion after stop/revoke/switch to B, require no A completion or ACK ever uses B token; an HTTP 200 under a new account is not proof of authorization. `AUTO-RUN-CRASH-01`: persist `runUuid` running, perform one externally countable mutation, crash before completion reply; server redelivers same UUID; restore must query/reconcile the ambiguous effect before retry or ACK and MUST NOT execute the mutation twice. Include 200-with-JSON vs empty-204 decoding, 401 revoked auth, 404 unknown/cross-account, 409 duplicate/rejected only after formal server owner contract; missing/late HTTP response is outcome-unknown, not safe implicit retry.

## Gate discipline and next evidence action

Source candidate file reads and third-party server comparison **do not** confer scoped G01/G02, upstream master/archive identity, official npm tarball integrity, production Sand backend authority, full owner/oracle approval or independent review. No validator/workflow, target runtime, pairing production writer, ledger, RTM, lock or acceptance flag modified. No local or Actions build/test/generator invoked. No agent delegation tool was available to claim independent subagent review. **Not accepted; OCRM-AC01..AC24 and all full-product predicates remain open.** Next actions: seek immutable codeload-master download run receipt (actual archive commit/hash), official Registry API via permitted infrastructure, and formal deployed Sand service owner tests/contracts; only after explicit scoped G01/G02 and independent reviewer positive evidence may a validator be implemented; pairing writer requires independent ACCEPT.
