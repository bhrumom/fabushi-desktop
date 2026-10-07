# Dossier — external URL accepted-auth credential boundary

Status: implemented / verification-open  
Project: TDRP-001 Revision 9  
Responsibility: `TDRP-R9-EXTERNAL-URL-AUTH-CONTEXT-001`

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Commit: `72b3b71c3d6e450e5ef94a3112dd750a0168aa0b`
- Primary source path: `Telegram/SourceFiles/boxes/url_auth_box.cpp`
- Blob: `f68ea15d650a07ed52839020fb46711012b288ae`
- Relevant source symbols: `AcceptedUrlContext`, `ActivateButton`, `ActivateUrl`, `RequestButton`, `RequestUrl`
- Related behavior was cross-read in `core/click_handler_types.cpp` and `core/ui_integration.cpp` to understand stripping and dispatch semantics.

This dossier covers only the accepted-auth-context / foreign web-auth token security responsibility. It does **not** mark those complete source files read or migrated.

## Source behavior

The accepted upstream separates ordinary external URL activation from server-accepted login URLs. Ordinary links remove foreign web-login credential parameters before dispatch. A server-returned accepted URL receives a privileged context that permits the server-issued login parameters to survive the final open.

The source recognizes credential parameter names rather than values, handles case-insensitive names, repeatedly percent-decodes the parameter name with a bounded loop, handles leading question marks, and sanitizes query and fragment parameter forms. This prevents an arbitrary message/link from carrying a login credential that could authenticate the user into a sender-controlled account.

## Existing-owner-first resolution

Existing Fabushi owners inspected:

- `source/shared/external-url-policy.ts`
- `source/electron-main/main-edge.ts`
- `source/electron-main/main-production-services.ts`
- `source/product/fabushi/fabushi-account-service.ts`
- `source/product/fabushi/fabushi-account-adapters.ts`

Chosen owners:

- ordinary external URL policy: existing shared external URL policy;
- privileged accepted-auth URL: existing Fabushi account owner after its server-origin validation.

The renderer-accessible MainEdge remains generic and is **not** allowed to mint accepted-auth privilege. No Telegram/Auth URL subsystem was added.

## Fabushi production implementation

Implementation commit: `aa9df7fef3454a81aacf489fecae984099cdeab4`

Targets:

- `source/shared/external-url-policy.ts`
  - `stripForeignWebAuthTokens`
  - `parseAllowedExternalUrl`
  - `parseServerAcceptedAuthExternalUrl`
- `source/product/fabushi/fabushi-account-adapters.ts`
  - `openServerAcceptedFabushiLoginUrl`

Ordinary HTTP/HTTPS opens strip case-insensitive `tgWebAuth*` and `autologin_token` parameters from query and fragment forms, including bounded nested-percent-encoded parameter names. Existing allowlisted non-HTTP schemes retain their existing behavior.

The only preservation path is the existing Fabushi browser-login owner. It validates the server-returned URL against the exact normalized first-party Fabushi API origin, rejects URL userinfo and origin mismatches, then opens the accepted URL through the existing native shell.

## Security/state/lifecycle

- Generic renderer/MainEdge callers cannot request a keep-token flag.
- Accepted-auth privilege is not persisted; it is re-derived from each authenticated server response.
- Account login keeps its existing AbortController/operation-epoch cancellation.
- No new network service is invented; the existing `/api/auth/browser/start` response is the service authority for the login URL.
- No second Identity, Account, URL, MainEdge, or native-shell owner is introduced.

## Requirement / oracle / invariants

- `TDRP-R9-EXTERNAL-URL-AUTH-CONTEXT-001`
- `ORA-TDRP-URL-AUTH-ACCEPTED-CONTEXT-001`: arbitrary external URLs cannot preserve foreign login credentials; an authenticated first-party server URL can preserve its server-issued credentials only through the account owner.
- `INV-TDRP-URL-AUTH-STRIP-FOREIGN-001`: ordinary HTTP/HTTPS opens remove reserved foreign web-auth parameter names from query/fragment before native dispatch.
- `INV-TDRP-URL-AUTH-EXACT-ORIGIN-001`: accepted-auth preservation requires exact normalized first-party origin and no URL userinfo.
- `INV-TDRP-URL-AUTH-NO-RENDERER-UPGRADE-001`: renderer/MainEdge generic input cannot upgrade itself to accepted-auth context.
- `PROP-TDRP-URL-AUTH-ENCODING-001`: casing and bounded nested percent encoding of the reserved parameter name do not bypass sanitization.
- `CONTRACT-TDRP-URL-AUTH-BOUNDARY-001`: privileged parser use remains inside the Fabushi account owner.
- `SEC-TDRP-URL-AUTH-STRIP-001`, `SEC-TDRP-URL-AUTH-EXACT-ORIGIN-001`, `SEC-TDRP-URL-AUTH-PRIVILEGE-001`: focused security cases.
- `REG-TDRP-URL-AUTH-TOKEN-SMUGGLING-001`: prevents reintroduction of foreign login-token smuggling.

## Tests and evidence

Focused test: `desktop/e2e/tdrp-external-url-auth-policy.spec.ts`.

Exact-head workflow: `.github/workflows/tdrp-external-url-auth-responsibility.yml`.

The workflow typechecks shipping TypeScript, runs the focused contract, asserts exact checkout, checks that accepted-auth privilege is composed only through the account owner rather than MainEdge, and publishes exact-head evidence.

The row remains `implemented`, not `verified`, until the current final HEAD obtains successful artifact-bound GitHub Actions evidence and independent review.
