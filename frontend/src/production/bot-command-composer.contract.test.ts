import fs from "node:fs";
import { describe, expect, it } from "vitest";

const root = new URL("../../", import.meta.url);
const read = (relative: string) => fs.readFileSync(new URL(relative, root), "utf8");

describe("canonical Bot command Composer contract", () => {
  it("projects durable Bot command metadata through the existing Agent model", () => {
    const model = read("src/production/model.ts");
    const renderer = read("src/production/ProductionRenderer.tsx");
    expect(model).toContain("projectRendererBotCommands");
    expect(model).toContain('command.startsWith("/")');
    expect(renderer).toContain("projectRendererBotCommands(activeAgent.raw.botCommands)");
    expect(renderer).toContain("activeIsHuman ? undefined");
  });

  it("uses the single ConversationComposer and suppresses command-start while composing or replying", () => {
    const composer = read("src/recovered/features/conversation/workspace/composer.tsx");
    expect(composer).toContain('aria-label="Bot commands"');
    expect(composer).toContain("botCommands.length > 0 && !hasPayload && replyTarget == null");
    expect(composer).toContain("editorControls.current?.insertText(command)");
    expect(composer).not.toContain("TelegramBot");
  });

  it("keeps custom reply markup message-bound inside the canonical transcript and Composer owners", () => {
    const workspaceModel = read("src/recovered/features/conversation/workspace/model.ts");
    const model = read("src/production/model.ts");
    const composer = read("src/recovered/features/conversation/workspace/composer.tsx");
    const renderer = read("src/production/ProductionRenderer.tsx");
    expect(workspaceModel).toContain("replyKeyboard?: ComposerReplyKeyboard");
    expect(model).toContain("projectComposerReplyKeyboard");
    expect(model).toContain("value.replyMarkup ?? value.replyKeyboard ?? message?.replyMarkup");
    expect(composer).toContain('aria-label="Bot keyboard"');
    expect(composer).toContain('aria-label="Hide bot keyboard"');
    expect(composer).toContain('aria-label="Show bot keyboard"');
    expect(composer).toContain("setHiddenReplyKeyboardMessageId(null)");
    expect(composer).toContain("replyKeyboard?.singleUse === true && replyKeyboard.persistent !== true");
    expect(composer).toContain('replyKeyboard.placeholder ?? "Reply…"');
    expect(renderer).toContain("function latestComposerReplyKeyboard");
    expect(renderer).toContain("entry?.kind === \"message\" && entry.role === \"assistant\" && entry.replyKeyboard != null");
    expect(renderer).toContain("replyKeyboard={activeIsHuman ? undefined : latestComposerReplyKeyboard(entries)}");
    expect(composer).not.toContain("TelegramKeyboard");
  });
});
