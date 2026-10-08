import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const root = new URL("../../../", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");

test("CONTRACT-TDRP-STORY-PRODUCT-SHELL-001 mounts one canonical Story capability surface", () => {
  const renderer = read("frontend/src/production/ProductionRenderer.tsx");
  assert.match(renderer, /import \{ StoryCapabilitySurface \} from "\.\/StoryCapabilitySurface";/);
  assert.equal(renderer.match(/<StoryCapabilitySurface\b/g)?.length ?? 0, 1);
  assert.match(
    renderer,
    /<StoryCapabilitySurface client=\{client\} enabled=\{account\?\.kind === "logged-in" && transport === "connected"\} onOpenOwner=\{openSidebarProfile\} \/>/,
  );
});

test("CONTRACT-TDRP-STORY-ACTION-LIFECYCLE-001 shares viewer actions and fences stale async work", () => {
  const surface = read("frontend/src/production/StoryCapabilitySurface.tsx");
  assert.match(surface, /type StoryAction = "previous" \| "next" \| "toggle-pause"/);
  assert.match(surface, /onClick=\{\(\) => act\("previous"\)\}/);
  assert.match(surface, /onClick=\{\(\) => act\("next"\)\}/);
  assert.match(surface, /onClick=\{\(\) => act\("toggle-pause"\)\}/);
  assert.match(surface, /event\.key === "ArrowLeft"[\s\S]*?act\("previous"\)/);
  assert.match(surface, /event\.key === "ArrowRight"[\s\S]*?act\("next"\)/);
  assert.match(surface, /event\.key === " " \|\| event\.key === "k"[\s\S]*?act\("toggle-pause"\)/);
  assert.match(surface, /requestGenerationRef/);
  assert.match(surface, /mediaGenerationRef/);
});

test("CONTRACT-TDRP-STORY-CANONICAL-ROUTE-001 keeps Story RPC on Coordinator to Host to canonical service", () => {
  const client = read("frontend/src/production/coordinator-client.ts");
  const shared = read("source/shared/rpc/coordinator.ts");
  const gatewayApi = read("source/host/src/host_gateway_api.rs");
  const gateway = read("source/host/src/extensions/session/gateway.rs");
  for (const method of ["listStories", "viewStory", "reactStory", "deleteStory"]) {
    assert.match(client, new RegExp(method));
    assert.match(shared, new RegExp(method));
    assert.match(gatewayApi, new RegExp("\\(\\\"" + method + "\\", Stories\\\)"));
    assert.match(gateway, new RegExp("\\"" + method + "\\" =>"));
  }
});
