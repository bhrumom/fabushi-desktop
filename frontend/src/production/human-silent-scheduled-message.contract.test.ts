import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "../..");
const read = (relativePath: string) => readFileSync(resolve(root, relativePath), "utf8");

describe("canonical Human silent and scheduled delivery", () => {
  it("keeps delivery options on the single Composer submission route", () => {
    const submission = read("recovered/features/conversation/workspace/submission.ts");
    const composer = read("recovered/features/conversation/workspace/composer.tsx");
    const renderer = read("production/ProductionRenderer.tsx");

    expect(submission).toContain("silent?: boolean");
    expect(submission).toContain("scheduledAtMs?: number");
    expect(composer).toContain('aria-label="Send options"');
    expect(composer).toContain("Send silently");
    expect(composer).toContain("Schedule send");
    expect(renderer).toContain("composerSubmissionQueue.submit(submission)");
    expect(renderer).toContain("onSendSilently={activeIsHuman ? () => submit({ silent: true }) : undefined}");
    expect(renderer).toContain("onScheduleSend={activeIsHuman ? openScheduleComposer : undefined}");
    expect(renderer).toContain("{ scheduledAtMs: submission.scheduledAtMs }");
  });

  it("validates and forwards delivery options through the existing Host owner", () => {
    const gateway = read("../host/src/extensions/session/gateway.rs");
    const production = read("../host/src/extensions/session/production.rs");
    const transport = read("../host/src/extensions/session/native_messaging.rs");

    expect(gateway).toContain('optional_bool(args, "silent")');
    expect(gateway).toContain('optional_i64(args, "scheduledAtMs")');
    expect(gateway).toContain("positive JavaScript-safe integer");
    expect(gateway).toContain("append_human_message_with_delivery");

    expect(production).toContain('"silent": silent');
    expect(production).toContain('"scheduledAtMs": scheduled_at_ms');
    expect(production).toContain("remote.silent != silent");
    expect(production).toContain("remote.scheduled_at_ms != scheduled_at_ms");
    expect(production).toContain('"delivery": if remote.delivery_state.as_deref() == Some("scheduled") { "scheduled" } else { "sent" }');
    expect(production).toContain("scheduled Human messages require the canonical server transport");

    expect(transport).toContain("silent: bool");
    expect(transport).toContain("scheduled_at_ms: Option<i64>");
    expect(transport).toContain('.post(self.endpoint("/api/social/messages")?)');
  });

  it("retains the original immediate-send entrypoint for current callers", () => {
    const production = read("../host/src/extensions/session/production.rs");
    expect(production).toContain("pub fn append_human_message(");
    expect(production).toContain("false,\n            None,\n            None,");
  });
});
