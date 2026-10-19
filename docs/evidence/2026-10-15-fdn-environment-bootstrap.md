# FDN environment bootstrap evidence — 2026-10-15

Overall: partial delivery; production activation remains blocked. All executable
verification ran in GitHub Actions. Local workstation/server activity was limited
to discovery, editing, Git/API operations and authorized SSH credential provisioning.
Existing uncommitted product changes in the shared checkout were not changed.

## Integrated source

- Backend PR76 merged:52a2555cbba6a599b19916151821d1adae8cfb4c.
  PR source6a370440a68926461d402e9971555dc9b6107640 passed Actions38663689182
  build-and-verify and38663689170 Headscale configuration. Controller tests include
  74 Node tests,0 failed,0 skipped; Go test/vet/build also passed. Shipping ELF is
  ARM aarch64. Post-merge build-and-verify38670429490 succeeded on merge SHA.
- Desktop PR91 merged:8148bfb9e747d446dd3d97342c3cf04a0ed407a2.
  Final PR source5f08b69d837ae75a8c6ab383bb7f5a599dbd7e96 passed real browser
  38664110533 and all Rust/renderer/platform/pressure jobs38664110764.
  Post-merge real browser38670570956 succeeded on merge SHA:12 executed/12 passed,
  0 failed/skipped,exit0. Real Chromium151.0.7922.34,Playwright1.62.1.
  Shipping driver digest:b53b17183bc970ee72b28c34d925881f026ba99fecc7e777c14a8fcb7d4b319e.

## Backend requirement-to-evidence review

| Requirement | Status | Evidence / remaining boundary |
| --- | --- | --- |
| ENV01/02,AC1 | passed | Four encrypted production FDN SSH inputs provisioned; private material not output or committed; temporary key removed. Pinned real SSH and sudo verified by main Actions38670452945,exit0. |
| ENV03 | passed | Explicit linux/arm64 cross-build and actual ARM aarch64 ELF inspection,38663689182; post-merge38670429490 succeeded. |
| ENV04 | passed | Workflow creates dedicated group/account before installing group-owned env files; exact-head Actions green; actual full install not yet run. |
| ENV05,AC2 | blocked | NETWORK_WORKER_ENV_B64,ENROLLMENT_ENV_B64,OFFICIAL_DEVICE_MCP_ENV_B64 absent; no FDN/Headscale/rootless stack or Core authority has been provisioned. Existing legacy MCP/ingress untouched. |
| ENV06 | blocked | Standard factory now verified live and accepted. Its allowlist currently permits only its two main workflows; device/MCP-created data cleanup remains outside accepted auth-only scope. No legacy TestAccount or isolated token promoted to production. |
| ENV07 | passed | All tests/builds/lint/artifact execution only in Actions. |
| ENV08 | passed | Five real launcher subprocess cases validate canonical32-byte AES keys and reject malformed/missing/short/trailing-junk before external service execution. |
| ENV09 | passed | Protected-main SSH inventory38670452945 confirms architecture,pinned identity,sudo,and all three service env paths not-configured. |
| AC3 | blocked | Real production OAuth/device directory/call/Host,direct/DERP/WSS,cross-account,revoke,response-loss/outcome_unknown acceptance not run. |

The four configured names are FABUSHI_FDN_DEPLOY_HOST,DEPLOY_USER,DEPLOY_SSH_KEY,
DEPLOY_KNOWN_HOSTS (each with FABUSHI_FDN_ prefix). Values are never included here.
Pending service payload names are FABUSHI_FDN_NETWORK_WORKER_ENV_B64,
FABUSHI_FDN_ENROLLMENT_ENV_B64,FABUSHI_FDN_OFFICIAL_DEVICE_MCP_ENV_B64.

## G11 requirement-to-evidence review

G11-R01/R02 passed: hosted disposable profile,loopback CDP,synthetic localhost fixtures,
exact checkout,literal shipping driver extraction,pinned dependencies and artifact.
G11-R03 passed: navigate/refresh/back/forward rotate generation; missing/stale generation
rejects without navigation. G11-R04 passed: same-URL exact target IDs stay distinct;
closed/reused view rejects old generation. G11-R05 passed: real iframe fill,missing-frame
rejection,download bytes/hash,workspace upload and outside/symlink/oversize rejection.
G11-R06/R08 passed: direct cookie/storage/profile methods and Runtime.evaluate synthetic
cookie export reject under explicit observation-only CDP allowlist. G11-R07 passed:
results JSON records source/driver/browser/runtime/run/commands/counts/exits. G11-R09
passed: exact history-entry dispatch,old generation fenced before dispatch,bounded
observation/disconnect. AC1/AC2/AC3 passed for the managed-browser driver scope.
AC4 remains blocked: outer Rust Host/box RPC,signed extension/native messaging,
packaged acceptance and production MCP are not exercised by this harness.

Actual browser findings required implementation repairs: unrestricted Runtime.evaluate
could read cookies; Playwright history restoration could hang. Earlier runs38663228665
(8/12) and38663478961(11/12) are retained as failures,not accepted results.

## Standard account factory

Latest accepted cloud run38666610725 succeeded on Backend
f6dbe6417b6252fdbbfb01bbd4b093ab2b7ae592. Literal cancelled-client prepare38668206894
and independent observer38668322225 attempt2 also passed:36 live owned auth rows before
expiry,0 afterward. Use ordinary OIDC lease API,one-time credentials held in memory,
owner-scoped cleanup/TTL. SMTP,device/MCP cleanup and production identity provisioning
are not implied by ordinary-auth acceptance. No additional consumer is allowlisted
without a reviewed policy change.

## Next action requiring confirmation

Cloudflare UI is on final token-summary page. Draft only; no new token created:
name fabushi-fdn-production-bootstrap-20261015; current Cloudflare account only;
Workers Scripts:Edit; start2026-10-15,end2026-10-16 (configured China time end23:59).
Purpose: authorized Actions configuration of Core FDN service authority. Proposed
save destination: Backend protected production secret FABUSHI_FDN_CF_BOOTSTRAP_TOKEN.
Creation grants new security-sensitive access and needs action-time confirmation
under the Computer Use tool's confirmation policy. Account-level Workers Scripts
permission is broader than one Worker; no DNS,D1,account-admin or billing permission
has been added. The draft does not resolve Headscale cell/rootless setup or test
fixture device/MCP cleanup by itself.

## Evidence links

- https://github.com/bhrumom/fabushi-backend/pull/76
- https://github.com/bhrumom/fabushi-desktop/pull/91
- https://github.com/bhrumom/fabushi-backend/actions/runs/38670452945
- https://github.com/bhrumom/fabushi-backend/actions/runs/38670429490
- https://github.com/bhrumom/fabushi-desktop/actions/runs/38670570956
- https://github.com/bhrumom/fabushi-backend/actions/runs/38666610725

## Confirmed-token continuation

User explicitly confirmed creation/storage. Cloudflare token created with only
accountWorkers-Scripts:Edit and expiry2026-10-16; encrypted production secret
FABUSHI_FDN_CF_BOOTSTRAP_TOKEN API readback timestamp2026-10-15T02:30:38Z.
No D1,DNS,account-admin or billing permission was added to this token. Never record
its value. Actual Actions38671649756 proves Worker settings readable.

Integrated Backend PR83(44ee2c6f1d7541855425b8242890481dadeb7400),PR84
(c8a0439776fd7d8450fa406fd5ba5ecd5c93fd40),PR85
(0c672da8993d3b58c879d56bc6b6db3102528ea4),PR86
(a6757a92bf5326e3410fdba1c1987f7c01721020).

Real production defects found and corrected:
- Initial FDN route404 prevented any authority mutation. Existing canonical Core
  delivery38671901555 published verifiedCoref781e6c85375ba0945aa4fd4c19ead89e396dfdb.
- One independent network authority was securely staged root-only and published;
  immediate edge response401 was retained as failure38672077824. Reconciliation
 38672189288 used the same staged key without rotation/publication and returned500.
- Actual PLATFORM_DB inventory showed10 missing canonical migrations0024,0025,
  0027–0034. Reviewed all10 SQL files. Both ACCOUNT_DB/PLATFORM_DB recovery bookmarks
  saved by38672574609. Explicit reviewed migration dispatch38672810857 applied all10
  and deployedCoreversiond14b73b4-32b7-48f5-a03a-1308909b5c06 successfully.
- Final authority reconciliation38672973704:PASS,exit0,unauthenticated401,
  authenticated200,Core/server staged same authority,actionreconcile-existing.
  No ordinary account,device or network node was provisioned by this read-only probe.

### Authority compliance
AB01/02 passed:protected-main Actions,actual approved token/settings,preflight-only
first run. AB03 passed:one random48-byte independent credential securely staged and
published via SSH stdin/HTTPS;no value in artifacts. AB04 passed:existing state
reconciles without rotation or uncertain mutation replay. AB05 passed:real empty
cell snapshot200 and unauthenticated401. AB06 passed:Headscale/identity/three full
payloads/production transport acceptance remain explicitly separate. AB07 passed:
canonical release preserved exact-Core/provider/signing/recovery/provenance gates;
no isolated factory credential fallback. AB08/09 passed:both database recovery
bookmarks and pending inventory precede opt-in canonical PLATFORM_DB migrations;
3 new ordering/isolation contracts plus existing gateway/static delivery suites
pass onPR85 final6006e5c;production apply38672810857 succeeded. The user-approved
limited token's permissions were not broadened; existing production credential
resolution completed the canonical release successfully.

### Rootless infrastructure
Actions38672970418 deployed exactBackenda6757a92bf5326e3410fdba1c1987f7c01721020
for dedicatedfabushi-fdn uid993,private socket/run/user/993/docker.sock. Dedicated
user docker.service andfabushi-fdn-headscale.service areactive;actual container
running at deploy completion,healthstarting (not yet a claimed healthy result).
Pinned image0.29.5 resolves tosha256:edd3681095a33fd0f76b1ee343571d3b5c2f7e263ea39da4d35be136f04faa8c.
Existing rootful workloads were retained. No globalVPN,DNS or firewall change.
Runtime state/policy persists under/var/lib/fabushi-fdn. RB01–04 passed by reviewed
source,Actions configtest and actual install;RB05 includes actual image/source/exit
but later healthy status is pending ingress stage. Public control/DERP acceptance
and account-specific controller enrollment remain blocked.

### Direct ingress preparation
Headscale's project-hosted community deployment guide explicitly states that
Cloudflare Proxy/Tunnel cannot handle the required POST control upgrade. Use a
DNS-onlyA record and direct TLS;do not follow the earlier possible tunnel path.
Reference:https://headscale.net/stable/ref/integration/reverse-proxy/.

Actual server TLS owner:HAProxyTCP443 -> existingApache127.0.0.1:9443. Apache80,
certbot and an existing registeredLet's Encrypt account already exist. New staged
hostname vhosts reuse these owners;no newproxydaemon or existingSNI-route change.
PR87 prepares stage/publish modes,public admin-path denial and minimal access logs.
Synthetic-certificate Apache configtest passed inActions38673321256;full exact-head
controller regression/stage delivery still pending at this entry.

Cloudflare UI has an unsaved draft:A fdn.ombhrum.com ->144.24.17.21,proxyoff
(DNS-only),TTLautomatic. No record was saved. Next confirmation must explicitly
cover DNS publication,real hostname TLS ingress andUDP3478STUN exposure. No public
Headscale admin API/gRPC/metrics should be exposed. This new public network access
requires action-time confirmation under the Computer Use tool policy,distinct
from the already-confirmed limited token creation.

The original three complete encrypted service payloads remain absent;do not mark
7/7 configured or production MCP/direct/DERP/WSS/revoke/recovery accepted yet.

### Direct ingress stage completed (2026-10-15T06:27Z)

PR87 merged at `f644870335ac779d683ff28908fff434c6fb0558` after all applicable
PR checks passed. Protected-main Actions run
https://github.com/bhrumom/fabushi-backend/actions/runs/38692462081 completed
successfully (exit0) with `publish=false`. Actual Apache configuration check ran
only in Actions. Downloaded deployment artifact records the same source SHA,
`mode=stage`, active service, `running healthy` Headscale container, pinned image
`sha256:edd3681095a33fd0f76b1ee343571d3b5c2f7e263ea39da4d35be136f04faa8c`, and
`ingress_staged_only=true`.

Compliance: RB05 local runtime health now passed. IN06 and AC1 stage passed;
IN02/IN04 configuration checks passed, but public routing/denial remains untested.
IN01/IN03/IN05 and AC2 public activation remain pending specific publication
confirmation. No DNS record saved, hostname vhost enabled, certificate requested,
or UDP3478 firewall rule added by this stage. Healthy container does not prove
public control upgrade, STUN, DERP, enrollment, MCP, or the three full payloads.

### Confirmed public ingress delivery (2026-10-15T06:30Z)

User explicitly confirmed the DNS-only A record, HTTPS and UDP3478/STUN publication
after stage review. Cloudflare UI save/readback shows `fdn.ombhrum.com`, A,
`144.24.17.21`, DNS-only, automatic TTL. Protected-main Actions
https://github.com/bhrumom/fabushi-backend/actions/runs/38692737102 completed
successfully (exit0), source `f644870335ac779d683ff28908fff434c6fb0558`,
`publish=true`. Artifact records active service and the same healthy image digest,
successful real Let's Encrypt certificate issuance (expires2027-01-13), both
dedicated Apache hostname sites activated and UDP3478 UFW rules added.

Actions `Verify real public TLS health and administrative denial` passed:
HTTPS `/health` succeeds with normal certificate verification; `/metrics`,
`/api/v1/node`, `/swagger`, `/debug` each return403. No private certificate content
or authority credential is present in the evidence artifact.

IN01–06 and AC1/AC2 passed for the scoped direct-ingress delivery, with source,
Actions configtest, actual install and public TLS evidence above. UDP firewall
allowance alone does not establish external STUN packet reachability; actual
control-protocol upgrade, enrollment, direct/DERP/WSS and full production MCP
remain separate incomplete acceptance. Standard factory README still explicitly
limits cleanup to auth data and forbids production credential fallback; device/MCP
consumer allowlisting and ownership cleanup must be accepted before those runs.

### Continued real-environment work (2026-10-15)

CorePR23 fixes the discovered signature-only ordinary authorization in Remote
Computer by reusing live account-session checks at all ordinary entrypoints and
the ordinary controller helper. Dedicated service/network authority stays separate.
PR headcb8cccef passed broker38694456018, schema/wasm38694455997 and extension
38694456047. Mergec7d0e24b7561c6f5d1bb3c0e975f599643252504 passed postmerge
schema/wasm38694611402 and broker38694611563. Canonical production delivery
38695093688 succeeded; artifact pins Corec7d0e24 and Backend06a87301, existing
Workermahayana-platform. Runtime denial remains a separate real cloud assertion.

BackendPR89 merged06a87301 adds strict external infrastructure probe. Runs
38693700361/38693879484 passed7 HTTP/DNS assertions but UDP timed out; passive
host capture saw0 packets. After the user logged into Oracle, actual shared
security-list inspection showed noUDP allowance. A new dedicated NSG
`fabushi-fdn-bhrum2-stun` was created with one stateful ingress rule,
IPv4 0.0.0.0/0, UDP destination3478, and attached only to bhrum2 primaryVNIC
(private10.0.0.69/public144.24.17.21). Shared subnet list and other VNICs unchanged.
NSG ocid1.networksecuritygroup.oc1.phx.aaaaaaaaoxo2erdvhqngjcifcxzyue77pm75shkykesxjwrl7hcfvsxx3gda.
Actual UI readback confirmed the saved association. Subsequent host counters
show incoming UDP; remaining timeout was localized to missing Tailscale-specific
request SOFTWARE/FINGERPRINT. BackendPR90 mergedb98202b3 and pinned upstream
parser interop passed in Actions38695797048, complete controller38695797144.
Real run38696389447 then received STUN but failed strict public mapped-address
validation. Do not call this public STUN/direct success: installedRootlessKit2.3.4
builtin forwarding rewrites source addresses. PR92 prepares source-preserving
slirp4netns only for the dedicated daemon; main delivery and reprobe pending.

BackendPR91 extends isolated factory cleanup to reviewed platform device/audit/
network and account connection/MCP/delegation resources, with live-session revoke,
persisted ownership, unknown-scope rejection and partial-failure recovery.
Actions38695907531 contracts pass; main cloud deployment/actual lifecycle
38696016535 is still running and not accepted yet. Production credentials and
production D1 are not used by this consumer. Do not promote test identities.

DesktopPR101 adds actual Rust BrowserToolBridge/ProductionBrowserToolExecutor
execution using real Actions Shell/Write/Read resources. Initial direct-driver
mode38696505701 passed12; Rust mode failed to compile due missing wayland-client
development package. The canonical Linux Host native dependency installation is
now added; next exact-head run pending. This remains distinct from remote Box RPC,
signed extension/native messaging and packaged client acceptance.

### Accepted isolated device factory lifecycle

BackendPR91 merged4ccf4f31ce035b19a0b697828f091d320e04b77b. Actual cloud run
38696016535 PASS, exit0:74/74 adversarial contracts and28/28 named actual remote
cloud assertions, no failed/skipped tests. Corec7d0e24b7561c6f5d1bb3c0e975f599643252504
compiled wasmSHA2564b28b6e6e8e4b9e38311326474afeba1404267206dca09e4e38b008368b17ea7.
Actual registration creates two A devices and B/C fixtures through canonical API;
idempotent inventory, cross-account registration/heartbeat/session denial, paired
session reconciliation, refresh/logout and stale signed-token list/register/
heartbeat denial all pass. Explicit cleanup and independent abandoned-device TTL
pass. Independent Cloudflare inspection reports auth remaining0,platform remaining0.
DR01–07/AC1/AC2 passed for this cloud control-plane slice. Live ordinary-session
denial now proves CorePR23 runtime behavior, separate from its production delivery.
This does not prove live account-MCP delegation, target Host execution or production
identity. Account MCP/delegation cleanup has real SQLite FK/ownership contracts.

BackendPR95 merged3373f2ad84ad4df3cacd1c6d0ac28cbcc9c5d2a4. Main-only cancellation
prepare38699748493 created3 ordinary accounts and1 normally registered device,
saved metadata then was actually cancelled. Metadata readback:37 total owned rows,
1 platform row,expiry2026-10-15T07:50:21.685Z. Observer38699994522 is read-only,
checks the actual cancelled run identity and actual D1 state; no explicit cleanup
or sweep is sent. DR08 remains pending observer completion. Factory executable
source remains4ccf4f31,Corec7d0e24; consumer/observer source is recorded separately.

### Network diagnosis refinement

PR92 source-preserving slirp4netns flags deployed38697358454 atdb7e167e.
PR93 disables only dedicated userland proxy; delivery38698589575 failed on missing
br_netfilter and safely restored the prior daemon. PR94 adds the installed official
kernel module prerequisite without changing sysctls, firewall rules or rootful
daemon; delivery38699469619 succeeded at366c5a72. Module loading is an explicit
documented host prerequisite addition; do not describe it as no host change.

Probe38699663486 still failed stun_mapping_invalid. That generic status also
included unsupported response family, so earlier private-address diagnosis was
an unconfirmed inference, not proof. Source review of actualHeadscale0.29.5
serverSTUNListener shows IPv4 may be returned as IPv4-mapped IPv6. PR96 adds correct
unmapping while still requiring a public IPv4, nonzero port and exact source/
cookie/transaction/length; actual pinned Tailscale Response interoperability and
Python adversarial tests pass. Full exact-head CI and new external run pending.

### G11 real Box findings

DesktopPR101 now exercises shipping Rust BrowserToolBridge/executor, production
Connect protobuf accessor and actual supervised/authenticated box-exec-daemon.
Run38697503722 found framed-request decoding and missing Write support; actual
workspace guard also required moving ephemeral browser runtime into private
/workspace/.sand-browser instead of widening filesystem roots. Repairs add strict
single-envelope decoding, bounded atomic workspace Write with descriptor-anchored
symlink protection, and dedicated filesystem/protocol regressions. Old ephemeral
claims are not migrated. Run38698699437 then found binary PNG reads were lossily
converted to UTF8; binary Data handling/read bounds were repaired. Run38699218529
passed11/12 actual Rust/Box cases, history failing at15s RPC idle deadline while
driver navigation permits25s stages. Latestc57da42 adds bounded120s Exec-only idle
deadline, preserving15s Control/unary deadline, caller cancellation and no retry.
Exact-head browser/full-owner CI pending; no G11 completion claim yet.

Three production payloads still require a real production immutable subject/cell.
An asynchronous question requests an ordinary production account identifier or
selection of a new dedicated production acceptance account. The standard isolated
factory credentials must not be promoted into production Core while awaiting it.


### Verified follow-through / dedicated production account decision

Backend main e35cc868a13f1809ec40337cc02a4692893b31d7 public infrastructure
run38700633492 passes8/8, exit0, skipped0 including actual external STUN binding.
This is not enrolled direct/DERP tool-call evidence.
Factory literal cancellation observer38699994522 passes: initial37 total/1 platform
row, cancelled consumer38699748493; after lease expiry auth0/platform0/total0,
without observer sweep or cleanup. Factory4ccf4f31/Corec7d0e24.
DesktopPR101 exact headc57da42 passes full Rust/renderer/worker/three-OS profiler
workflow38700169192 and browser38700169640:12/12 direct,12/12 Rust/Box,171 real
resource calls, skipped0, exit0. Driver744cb467a8a660383d517b555f7bca161dba5e6ecc10e4d6f0ae943d39cd9e54.
Merged asc33a9495fc1e640557d65f3e2daedc1449be7fcb; exact-main real browser
run38731191530 and full owner38731189125 in progress. External Box deployment,
extension/package and production approval-policy acceptance remain separate.
User explicitly chose a new dedicated production account. Prepared ordinary
canonical browser registration forfdn_acceptance_20261015 with verified operator
Gmail plus alias. Browser tool requires operator entry of new password and action-
time terms acceptance; handoff requested without asking for code/password in chat.
No account yet created; no isolated credentials promoted; three payloads remain
blocked on actual immutable production subject/cell. PA01–05 durable addendum
staged in Backend production environment spec before further configuration.

Exact-main G11 run38731191530 atc33a9495fc1e640557d65f3e2daedc1449be7fcb
completed successfully. Downloaded both artifacts:12/12 direct and12/12 Rust/Box,
zero failures/skips, exit0. Full main owner run38731189125 still pending at last
inspection. Backend docsPR97 mergedeaf2960a source as3d7fe854d21d0a328b1b37cc326e7bdadfa03c8b.
Production registration code delivery visually confirmed; credential/terms handoff
still pending. No production account or final three secrets claimed configured.


### Production registration mail repaired

User explicitly confirmed existing OraclePhoenix mail domain/sender, dedicated
SMTP-only identity, exact sender policy, TLS587 relay and required SPF/DKIM updates.
Oracle domainombhrum.com created; senderamitabha@ombhrum.com approved; DKIMselector
fabushi-20261015 nowactive. Cloudflare DNS-onlyCNAME points tofabushi-20261015.ombhrum.com.dkim.phx1.oracleemaildelivery.com;
SPF retainsip4:144.24.17.21 andCloudflare include, addingrp.oracleemaildelivery.com.
IAMgroupFabushiRegistrationMail has useapproved-senders only where target.approved-sender.emailaddress equals the one approved address.
Serviceuserfabushi-registration-mail has SMTP capability only; password/API/token/
OAuth/customer-key/database capabilities disabled. SMTPcredential configured in
protected production FABUSHI_MAIL_RELAY_CREDENTIALS_B64; no credential values here.
BackendPR99 merged7110dc6c29466d9ffa7d500caadb0bf348a65b92 afterexact-head6tests+
entrypointcompileCI38741461689. Productiondelivery38741571017 succeeded and real
mail API submission200; actualGmailinbox row subjectFabushi production mail
verification38741571017 confirms receipt. A subsequent canonical production code
mail1a13fb8df599b368 also arrived. Queue acceptance alone was not used as proof.
InstalledAPIappSHAa87b23e97041ac664ed7350fd6da4f5bcf18980929f3ad66bce9aef59d570a28;
rollbackmanifest/backup /var/backups/fabushi-mail/20261015T131844.662057Z.
No unrelated queues flushed. ExistingAPI now sets configured envelope sender,
Postfix uses exactsender TLS/auth maps; globalrelayhost unchanged.

Gmail deliberately withholds OTP as[one-time authentication code withheld]. No
raw/UI/queue workaround attempted. BackendPR100 prepares encrypted private OTP
input and automatic ordinary account/password administration; actual account still
not created pending that input. Normal runtimeacceptance remains separate from
account administration. Temporary localSMTPclipboard copies and obsolete local
browserattempt files removed; protected production/server credentials retained.


### Encrypted OTP account bootstrap ready

BackendPR100 mergedb8d82eb8446619cafb37cb3ff02d016448ed7b6d; exact-head598a5b0a
CI38743326051 passed7 contracts and syntax. Main verify38743432966 passed; protected
prepare38743434930 passed. Canonical new registration challenge prepared; sanitized
artifact statusawaiting_private_otp, expiresAt1792071717. Strong generated password
and challenge state retained only root-private on controller. No account created yet.
Unified-device private cardchallengeb805cff1-58e2-4f86-a901-d55db8e33e72 created:
E2E OTP input writes owner-private file then normalAPIconsumer completes. Tool says
pause device control awaiting user; no OTP is requested in chat or exposed to model.
Cardexpiry1792071473691. Remaining three service payloads and production MCP/network
acceptance are still pending the actual immutable ordinary production subject.


### Private-input callback reconciliation

At13:39UTC app callback claimed a private submission. Authoritative device readback
showed credentials absent, pendingphaseawaiting_private_otp, noOTPfile. Tool audit
showed noOTPwrite or registrationconsumer invocation. Oldcardexpired1792071473691
before callback; cancellation confirmed no value sent. No sensitive value read or
recovered from logs. Fresh prepare38744966468 succeeded atsameverifiedb8d82eb8;
newCorechallengeexpiresAt1792072358. New encrypted card issued,5minutevalidity
explicit in title/help; registration remains pending actual device completion.

## Canonical production identity created
Private card b4ec3fc1-3b7c-49dc-8fd2-cdeb9a61ea46 completed canonical registration/login/profile validation. Ordinary accountId218013940318977; cell cell-7e9RJhdDu69TuF2cmD7OFcRqrdwjAugh. Root-private credentials/session retained; pending/OTP removed. Source b8d82eb8446619cafb37cb3ff02d016448ed7b6d. This is not production MCP acceptance.

### Controller service payloads and failed first activation

All three real service payloads were sealed on bhrum2 with the GitHub production
public key and written as environment secrets; plaintext never appeared in chat.
Readback timestamps2026-10-15T13:53:35Z/36Z/37Z. Separate broker, enrollment AES256
and MCP vault keys retained root-private; ordinary account tokens excluded.
Controller administration created the exact Core-derived Headscale user and a
nonreusable nonephemeral300s key after the build artifact passed. This does not
bypass target-device grant redemption. Mainb8d82eb8 run38746114659 attempt2 passed
all seven inputs/exact Core gates, but deployment activation failed (exit1), not pass.

Read-only reconciliation on2026-10-16 found no Headscale nodes and no controller
persisted tailscaled.state. Enrollment ProtectHome blocked rootless socket; MCP
endpoint config lacked fixed /v1/fdn/private-request path. Network worker generic
startup error hid the reason; upstream netmon needs AF_NETLINK, absent in unit.
New services/timer stopped to halt restart loops; legacy public MCP still active.
Stale transient /tmp envs removed; cleanup ordering repair in BackendPR101.

Spec extension CB01–05/ST01–04 recorded before changes. BackendPR101 latest
003fe6e0996edf66a2fe92e7c6f77cb9ac374059 adds exact socket-only bind exception,
AF_NETLINK and fixed sanitized diagnostics, bounded readiness, early-error rollback,
same-source release preservation and cleanup identity ordering. Latest official-MCP
38828838166 passed44 MCP contracts and Go worker checks; coordinator38828838143
passed. Deployment build38828838237 pending completion. No production health or
network acceptance claimed. MCP payload corrected without rotating durable keys.

### Production controller and real service health accepted (2026-10-16)

BackendPR101 merged118c4a24706a35469fcc1a861782c97f9576f379; exact-head003fe6e
38828838237 passed four startup/reinstall/rollback/foreign-socket contracts plus
existing coordinator/Go/MCP gates. Actual118c4a2 deployment38829239481 established
controller100.88.207.67/Core ingress generation1 and MCP8794, but whole install
failed because enrollment's symlink direct-entry guard exited0 before listening.
Rollback stopped new services as designed. Persistent network identity retained.

BackendPR102 merged4213b49d88a8441968477ca1de59e7033f1b4bdd; exact-headf831fb9
38830016942 passed six service contracts including actual symlink listener401 and
invalid config nonzero exit. Production38830333777 passed build+deploy at4213b49,
Corec7d0e24, bundle086e9a2f065825e10c40ee6bd5144be44805a87d68c8b2f602ad3ebb3a363da9.
No new node or key minted after the prior successful enrollment.

BackendPR103 mergedb06a45d795c7e56fd1a88f8ae63d25d970ae1088 (operational inventory/spec).
Protected read-only inventory38830843065 passed installed runtime4213b49 provenance,
three active services+ingress timer, actual workerReady/MCP health, enrollment401,
and successful Core ingress binder. No credential/private state copied to artifact.
Artifacts local /tmp/fdn-completion/controller-health/accepted and
/tmp/fdn-completion/enrollment-symlink/deployed. Service liveness is not full E2E.

Existing canonical account API delivery38830720805 passed at4213b49 on actual ARM64:
full142 tests before and142 after production prune, immutable release installed,
public Registry health passed. Existing data/env/signing ownership preserved.
Additive private proxy env uses broker URL/token and trusted Headscale control URL.
The control URL addition awaits next canonical release activation (no direct restart).
Host defaults api.ombhrum.com; read-only gateway source inspection found the two
/api/fdn/network-enrollment routes unowned. BackendPR104 governs narrow routing,
no retry, bearer-only whitelist and Actions public401/owner evidence. Still pending.

G11 current Desktopmainc33a9495fc1e640557d65f3e2daedc1449be7fcb reverified through
GitHub:38731191530 and fullowner38731189125 success. Real direct+Rust/Box12+12 pass,
zero skipped. This does not claim installed public MCP-to-native Host acceptance.

### Public enrollment gateway and final environment review

BackendPR104 merged3c195ff138715cc5040bb8b36ddd1e4153363070. Exactheadba6d23c
38831399822 passed7 account-proxy+4 gateway contracts; static38831399637 passed.
Canonical Worker38831459832 passed security preflight/bookmark/migration/deploy,
generalhealth and public POST start/complete each401 with existing-account-api marker.
Canonical Node38831456633 passed ARM64 full142+142 and immutable deployment/public
health atsame3c195ff. Read-only active process inventory confirmed broker loopback,
strong broker credential and trusted public Headscale origin configured (no values
output). Existing originalenv/data/signing configuration preserved.

Two initial prerequisites now passed: seven real production FDN inputs/healthy
controller stack and real G11 browser/Box12+12. AC-3/G05/G16 remain blocked pending
actual production OAuth-to-shipping-Host, native permission/execution, direct/DERP/
WSS, live cross-account/revoke and response-loss/outcome_unknown evidence. No public
legacy MCP ingress cutover or complete production E2E is claimed. Backenddocs PR106
records per-criterion passed/blocked evidence in canonical owning specs.
