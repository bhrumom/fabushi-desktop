import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const root = path.resolve(__dirname, "..", "..");
const read = (relative: string) => fs.readFileSync(path.join(root, relative), "utf8");

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
});
