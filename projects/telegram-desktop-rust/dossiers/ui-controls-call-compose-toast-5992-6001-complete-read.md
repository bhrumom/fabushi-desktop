# UI controls call/composer/toast 5992-6001 complete read

Status: exact-source read complete; mapped-open except a partial Human-call production adaptation; no global unknown reduction
Accepted upstream: `65e23ba7137ea4129b6bc1b2616104a1f59495ef`
Accepted tree: `6b616494f3465324e749a04dcd1c9d508657a998`

## Responsibilities

Orders 5,992-5,993 (`call_button.{cpp,h}`) own circular hit testing, ripple/outer emphasis, style-progress interpolation, icon morphing, reactive fit-gated labels, color overrides and optional corner-action composition. They map to canonical Button/IconButton plus the existing Human call surface; no source-named CallButton is introduced.

Orders 5,994-5,995 (`call_mute_button.{cpp,h}`) own Connecting/Active/Muted/ForceMuted/RaisedHand/ConferenceForceMuted/scheduled visual state, audio-level glow/blob projection, label/icon transitions, tooltip/input gating and teardown. Force-muted/raised-hand states can remain actionable, so visual mute/disabled state is not authoritative command truth.

Orders 5,996-5,997 (`chat_service_checkbox.{cpp,h}`) specialize Checkbox presentation with palette-invalidated animation frames, no ripple and an optional nontransparent rounded background. They map to canonical `SandCheckbox` plus semantic theme tokens.

Orders 5,998-5,999 (`compose_ai_button_factory.{cpp,h}`) gate compose-AI/expand affordances on availability, meaningful text, usable three-line field height and current message-length limits. Large non-image paste classification uses an eight-times-current-message-limit threshold and preserves exact resulting text for UTF-8 `message.txt`; markdown/entities round-trip through the canonical composer. These remain mapped-open under Composer/attachment/AI owners.

Orders 6,000-6,001 (`custom_emoji_toast_icon.{cpp,h}`) project a session-resolved custom emoji into a pointer-transparent Toast icon, repaint on resource updates, center it and inherit semantic Toast foreground. They remain mapped-open to canonical Toast plus emoji/media resolution.

## Partial production adaptation

The shipping `frontend/src/production/human-call-media.tsx#HumanCallControls` now reuses canonical `SandButton` for call actions. Mute independently acquires the existing source-neutral in-flight command fence, exposes pending through the canonical Button contract, refuses duplicate entry, restores audio track/UI state when authoritative `updateCallMedia` fails, releases in `finally`, and disposes the fence on teardown.

Focused contract: `frontend/src/production/human-call-controls.contract.test.ts`, wired into the Rust desktop runtime renderer contract job. This does not close broader call animation/scheduled/raised-hand/group-call, compose-AI/large-paste or custom-emoji Toast responsibilities before descendant same-head evidence and remaining behavior are complete.

## Accounting

Read-through **6,001 / 16,125**; unread **10,124**; unknown **15,846**; omitted **0**. First unread: order **6,002** `Telegram/SourceFiles/ui/controls/delete_message_context_action.cpp@9cecf76e10501486a63c9eff49ed2dfa083be45f`. Reading alone grants no baseline/release credit.
