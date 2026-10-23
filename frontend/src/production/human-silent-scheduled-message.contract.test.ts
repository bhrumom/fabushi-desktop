import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";

const root = resolve(import.meta.dirname, "../../..");
const read = (relativePath: string) => readFileSync(resolve(root, relativePath), "utf8");

test("Human delivery options stay on the single Composer submission route", () => {
  const submission = read("frontend/src/recovered/features/conversation/workspace/submission.ts");
  const composer = read("frontend/src/recovered/features/conversation/workspace/composer.tsx");
  const renderer = read("frontend/src/production/ProductionRenderer.tsx");

  assert.match(submission, /silent\?: boolean/);
  assert.match(submission, /scheduledAtMs\?: number/);
  assert.match(composer, /aria-label="Send options"/);
  assert.match(composer, /Send silently/);
  assert.match(composer, /Schedule send/);
  assert.match(renderer, /composerSubmissionQueue\.submit\(submission\)/);
  assert.match(renderer, /onSendSilently=\{activeIsHuman \? \(\) => submit\(\{ silent: true \}\) : undefined\}/);
  assert.match(renderer, /onScheduleSend=\{activeIsHuman \? openScheduleComposer : undefined\}/);
  assert.match(renderer, /scheduledAtMs: submission\.scheduledAtMs/);
});

test("Host validates and forwards Human delivery options through existing owners", () => {
  const gateway = read("source/host/src/extensions/session/gateway.rs");
  const production = read("source/host/src/extensions/session/production.rs");
  const transport = read("source/host/src/extensions/session/native_messaging.rs");

  assert.match(gateway, /optional_bool\(args, "silent"\)/);
  assert.match(gateway, /optional_i64\(args, "scheduledAtMs"\)/);
  assert.match(gateway, /positive JavaScript-safe integer/);
  assert.match(gateway, /append_human_message_with_delivery/);

  assert.match(production, /"silent": silent/);
  assert.match(production, /"scheduledAtMs": scheduled_at_ms/);
  assert.match(production, /remote\.silent != silent/);
  assert.match(production, /remote\.scheduled_at_ms != scheduled_at_ms/);
  assert.match(production, /remote\.delivery_state\.as_deref\(\) == Some\("scheduled"\)/);
  assert.match(production, /scheduled Human messages require the canonical server transport/);

  assert.match(transport, /silent: bool/);
  assert.match(transport, /scheduled_at_ms: Option<i64>/);
  assert.match(transport, /\.post\(self\.endpoint\("\/api\/social\/messages"\)\?\)/);
});

test("Immediate Human send remains the compatibility entrypoint", () => {
  const production = read("source/host/src/extensions/session/production.rs");
  assert.match(production, /pub fn append_human_message\(/);
  assert.match(production, /false,\s+None,\s+None,/);
});
