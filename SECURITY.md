# Security policy

## Reporting a vulnerability

Do **not** publish suspected vulnerabilities, secrets, tokens, credentials, private user data, or exploit details in a public issue.

Use GitHub's private vulnerability reporting/security advisory flow for this repository when available. If that flow is unavailable, contact the repository maintainers through an established private channel rather than opening a public issue.

Include only information needed to reproduce and assess the issue: affected version/commit, affected component, prerequisites, reproduction steps, observed impact, redacted evidence, and a mitigation suggestion if known.

## Secrets and credentials

Never commit API keys, OAuth tokens, signing certificates/private keys, passwords/OTP, session cookies, production secrets, or unredacted private user data.

If a secret is committed or exposed, treat it as compromised and rotate/revoke it. Removing it from a later commit is not sufficient.

## Security-sensitive changes

Security/privacy/authentication/authorization/capability changes require an applicable Spec, explicit threat/failure considerations, review of privilege boundaries, and verification appropriate to the risk.

Privileged desktop behaviors such as local execution, computer control, connectors/MCP, OAuth, filesystem access, and sensitive network actions must remain behind explicit capability/security boundaries.

## Supported versions

This file does not invent a long-term support matrix. The governing release/project Spec and current published release information determine what is actively supported for a specific fix.
