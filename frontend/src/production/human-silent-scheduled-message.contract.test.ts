import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");

describe("canonical Human silent and scheduled transport", () => {
  it("keeps silent/scheduledAtMs on the existing Composer -> queue -> sendHumanMessage route", () => {
    const submission = read("recovered/features/conversation/workspace/submission.ts");
    const composer = read("recovered/features/conversation/workspace/composer.tsx");
    const renderer = read("production/ProductionRenderer.tsx");
    expect(submission).toContain("silent?: boolean");
    expect(submission).toContain("scheduledAtMs?: number");
    expect(composer).toContain('aria-label="Send options"');
    expect(composer).toContain("Send silently");
    expect(composer).toContain("Schedule send");
    expect(renderer).toContain('client.call("sendHumanMessage"');
    expect(renderer).toContain("{ silent: true }");
    expect(renderer).toContain("{ scheduledAtMs: submission.scheduledAtMs }");
    expect(renderer).toContain("composerSubmissionQueue.submit(submission)");
    expect(renderer).toContain("clearDraftIfMatches");
    expect(renderer).toContain("removeStashIfMatches");
  });

  it("keeps Host durability and canonical server payload in one owner", () => {
    const gateway = read("../host/src/extensions/session/gateway.rs");
    const production = read("../host/src/extensions/session/production.rs");
    const transport = read("../host/src/extensions/session/native_messaging.rs");
    expect(gateway).toContain('optional_bool(args, "silent")');
    expect(gateway).toContain('optional_i64(args, "scheduledAtMs")');
    expect(production).toContain('"delivery": "pending"');
    expect(production).toContain('"delivery": if remote.delivery_state.as_deref() == Some("scheduled") { "scheduled" } else { "sent" }');
    expect(production).toContain("remote.silent != silent");
    expect(production).toContain("remote.scheduled_at_ms != scheduled_at_ms");
    expect(transport).toContain("scheduled_at_ms: Option<i64>");
    expect(transport).toContain("silent: bool");
  });
});
