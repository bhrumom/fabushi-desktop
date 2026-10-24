# Security & Privacy Test Plan

Status: active / normative  
Plan ID: FQT-SEC-001  
Updated: 2026-10-07

Scope：account/session isolation、permissions/roles、block/privacy、secrets/tokens、resources、Mini Apps/WebMCP、Plugins/MCP、external navigation、logs/telemetry、local persistence/update。

Negative：cross-account search/transcript leak；revoked session；permission downgrade while open；blocked participant discovery；private room after leave；stale MiniApp nonce/session；duplicate request ID；dispose/cancel pending；secret in logs/artifacts；unsafe URL/file。

Rules：authorization before exposure；least privilege；fail closed；cleanup on sign-out/dispose；evidence redacted。Security/privacy P0/P1 blocks release。
