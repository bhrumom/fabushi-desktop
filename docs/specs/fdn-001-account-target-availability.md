# FDN-001 account and Remote Computer target availability

Status: implementation contract.
Owner: Fabushi Desktop product account service.
Parent: FDN-001.

## Invariant

A validated Fabushi account session is the product identity authority. Automatic Remote Computer target registration, heartbeat, network enrollment, or coordinator availability is subordinate capability state.

Failure to register or refresh the Remote Computer target must never reinterpret an otherwise valid Fabushi account session as logged out.

## Required behavior

When a valid session is present:

- account status remains logged-in even if Remote Computer target registration fails;
- the ordinary first-party Host access credential is still projected from the valid account session;
- any stale Remote Computer device credential is removed;
- Remote Computer registered identity and heartbeat ownership are cleared;
- a later status/token read may retry target registration;
- no target registration error is converted into a fake successful target capability.

Expired/invalid account sessions still fail closed and remain logged out.

## Verification

GitHub Actions only. The focused account contract test must prove the optional target settlement helper absorbs only target availability failure, invokes fail-closed target cleanup, and does not suppress successful registration. Existing signed packaged acceptance remains unchanged and must pass on the resulting exact source SHA.
