# Composer attachment source closure — orders 517–519

Authority: `telegramdesktop/tdesktop@63a1831728147e941ad2dc8939879cf5afe419c7`.
Fabushi evidence head: `bbe2eca6967465e683e6158d4e2490725c0ae940`.

This closes only the three exact `input_attach` scale assets. It does not close the adjacent auto-delete/TTL controls at orders 520–522.

## Exact source consumers and responsibility

Telegram accepted-tree search resolves `input_attach` in `Telegram/SourceFiles/chat_helpers/chat_helpers.style` as the history attachment button and in `Telegram/SourceFiles/media/view/media_view.style` as the story/media composer attachment button. The three exact blobs are:

- 517 `Telegram/Resources/icons/chat/input_attach.png@959b6ff3d25877507eca1b854c16f39c55a8112e`
- 518 `Telegram/Resources/icons/chat/input_attach@2x.png@d3af81c6f58cdcd5a2a2cc670bbf4486d62730a0`
- 519 `Telegram/Resources/icons/chat/input_attach@3x.png@96ff13d0dc00f77bebe6f873415703bd2e126b88`

Their responsibility is the attachment affordance and staged-resource lifecycle, not message TTL.

## Canonical Fabushi owner

`frontend/src/recovered/features/conversation/workspace/composer.tsx#ConversationComposer` is the single shipping composer owner. At `bbe2eca6967465e683e6158d4e2490725c0ae940` it has:

- an accessible `Attach file` canonical icon button;
- a hidden multi-file input;
- drag/drop and paste-file staging;
- the canonical attachment limit;
- `Attachments` list/listitem projection with size/kind metadata;
- per-item removal;
- draft-owned attachment state passed through `onStageFiles`.

ProductionRenderer continues staging/committing through the existing canonical draft/resource/Host path; no Telegram attachment store is introduced.

## Exact-head acceptance

Desktop Chat Parity CI run `38312081487`, Focused Electron chat E2E job `115020027184`, succeeded at `bbe2eca6967465e683e6158d4e2490725c0ae940`.

The shipping test `Human reply, attachment, reaction, and search stay on the shipping conversation contracts` in `desktop/e2e/mahayana-agent-workbench.spec.ts`:

1. calls `setInputFiles` on `input.sand-prompt-file-input`;
2. waits for the Composer-owned `Attachments` listitem;
3. sends through the real shipping conversation path;
4. verifies the settled transcript attachment;
5. verifies the uploaded resource bytes equal the input bytes;
6. navigates away/back and verifies the attachment is restored from canonical conversation history.

The same job produced Playwright artifact `11774878972`, digest `sha256:a1b708e3bf3a09c2a3fa554024d7fb8d07e8800654f8cdc9eda22e098484c2bf`.

Therefore orders 517–519 are `unknown_closed=true`, `omitted=false`. Orders 520–522 remain product-open for TTL/auto-delete semantics.
