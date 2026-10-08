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
    /<StoryCapabilitySurface client=\{client\} enabled=\{account\?\.kind === "logged-in" && transport === "connected"\} resolveMedia=\{resolveAttachmentMedia\} onOpenOwner=\{openSidebarProfile\} \/>/,
  );
});

test("CONTRACT-TDRP-STORY-ACTION-LIFECYCLE-001 shares viewer actions and fences stale async work", () => {
  const surface = read("frontend/src/production/StoryCapabilitySurface.tsx");
  assert.match(surface, /type StoryAction = "previous" \| "next" \| "toggle-pause" \| "press-start" \| "press-end" \| "toggle-menu" \| "toggle-reaction-menu"/);
  assert.match(surface, /onClick=\{\(\) => act\("previous"\)\}/);
  assert.match(surface, /onClick=\{\(\) => act\("next"\)\}/);
  assert.match(surface, /onClick=\{\(\) => act\("toggle-pause"\)\}/);
  assert.match(surface, /event\.key === "ArrowLeft"[\s\S]*?act\("previous"\)/);
  assert.match(surface, /event\.key === "ArrowRight"[\s\S]*?act\("next"\)/);
  assert.match(surface, /event\.key === " " \|\| event\.key === "k"[\s\S]*?act\("toggle-pause"\)/);
  assert.match(surface, /onPointerDown=\{\(event\) => \{[\s\S]*?act\("press-start"\)/);
  assert.match(surface, /onPointerCancel=\{\(\) => act\("press-end"\)\}/);
  assert.match(surface, /onPointerUp=\{\(event\) => \{[\s\S]*?act\("press-end"\)[\s\S]*?act\("previous"\)[\s\S]*?act\("next"\)[\s\S]*?act\("toggle-pause"\)/);
  assert.match(surface, /onClick=\{\(\) => act\("toggle-menu"\)\}/);
  assert.match(surface, /onClick=\{\(\) => act\("toggle-reaction-menu"\)\}/);
  assert.match(surface, /const playbackPaused = paused \|\| pointerPressed \|\| menuOpen \|\| reactionMenuOpen/);
  assert.match(surface, /QUICK_REACTION_EMOJIS\.map/);
  assert.match(surface, /<ReactionCell/);
  assert.match(surface, /aria-label="Story reactions"/);
  assert.match(surface, /setReaction\(viewed\.myReaction \?\? null\)/);
  assert.match(surface, /selectedStory\.canDelete \? <SandButton[\s\S]*?>Delete<\/SandButton> : null/);
  assert.match(surface, /aria-label="Story media navigation"/);
  assert.match(surface, /aria-label="Story caption"/);
  assert.match(surface, /WebkitLineClamp: 2/);
  assert.match(surface, /aria-expanded=\{captionExpanded\}/);
  assert.match(surface, /Show full caption/);
  assert.match(surface, /Collapse caption/);
  assert.match(surface, /requestGenerationRef/);
  assert.match(surface, /reactionGenerationRef/);
  assert.match(surface, /reactionGeneration !== reactionGenerationRef\.current/);
  assert.match(surface, /event\.currentTarget !== videoRef\.current/);
  assert.match(surface, /key=\{selectedStory\.id\}/);
  assert.match(surface, /mediaGenerationRef/);
});

test("CONTRACT-TDRP-STORY-CANONICAL-ROUTE-001 keeps Story RPC on Coordinator to Host to canonical service", () => {
  const client = read("frontend/src/production/coordinator-client.ts");
  const shared = read("source/shared/rpc/coordinator.ts");
  const gatewayApi = read("source/host/src/host_gateway_api.rs");
  const gateway = read("source/host/src/extensions/session/gateway.rs");
  const storyMethods = ["getStoryStealthStatus", "activateStoryStealth", "listStories", "viewStory", "reactStory", "deleteStory"];
  const extensionRegistry = gatewayApi.match(
    /pub const FABUSHI_HOST_GATEWAY_METHODS: &\[&str\] = &\[([\s\S]*?)\];/,
  )?.[1];
  assert.ok(extensionRegistry, "Fabushi Host gateway extension registry must remain present");
  assert.deepEqual(
    [...extensionRegistry.matchAll(/"([^"]+)"/g)].map((match) => match[1]),
    storyMethods,
  );
  for (const method of storyMethods) {
    assert.match(client, new RegExp(method));
    assert.match(shared, new RegExp(method));
    assert.ok(extensionRegistry.includes(`"${method}"`), `${method} must remain in the Fabushi Host gateway extension registry`);
    assert.ok(gateway.includes(`"${method}" =>`), `${method} must remain routed by the shipping Host session gateway`);
  }
});

test("CONTRACT-TDRP-STORY-STEALTH-SHIPPING-001 keeps entitlement state server-owned", () => {
  const surface = read("frontend/src/production/StoryCapabilitySurface.tsx");
  const production = read("source/host/src/extensions/session/production.rs");
  const shared = read("source/shared/rpc/coordinator.ts");
  const engine = read("native/mahayana-messaging/src/engine.rs");
  assert.match(surface, /client\.getStoryStealthStatus\(\)/);
  assert.match(surface, /client\.activateStoryStealth\(\{ requestId \}\)/);
  assert.match(surface, /Anonymous viewing requires entitlement/);
  assert.match(surface, /formatStoryTimeLeft/);
  assert.match(surface, /window\.setInterval\([\s\S]*?250\)/);
  assert.match(surface, /Anonymous viewing active ·/);
  assert.match(surface, /Anonymous viewing available in/);
  assert.match(production, /MessagingClientCommand::StoryStealthStatus/);
  assert.match(production, /MessagingClientCommand::ActivateStoryStealth/);
  assert.match(production, /fn project_story_for_surface/);
  assert.match(production, /story\.owner_id == actor_id/);
  assert.match(production, /view\.reaction\.clone\(\)/);
  assert.match(shared, /readonly canDelete: boolean/);
  assert.match(shared, /readonly myReaction\?: string \| null/);
  assert.match(engine, /StoryStealthEntitlementRequired/);
  assert.match(engine, /StoryStealthCooldown/);
  assert.match(engine, /anonymize_recent_view/);
  assert.match(engine, /record_anonymous_view/);
});

test("CONTRACT-TDRP-STORY-RESOURCE-001 reuses canonical attachment media resolution with stale fencing", () => {
  const renderer = read("frontend/src/production/ProductionRenderer.tsx");
  const surface = read("frontend/src/production/StoryCapabilitySurface.tsx");
  assert.match(renderer, /resolveMedia=\{resolveAttachmentMedia\}/);
  assert.match(surface, /resolveWithSingleRetry\(resolveMedia, localPath\)/);
  assert.match(surface, /selectedStory\.media\.localPath/);
  assert.match(surface, /generation !== mediaGenerationRef\.current/);
  assert.match(surface, /setResolvedMediaSource\(fallback\)/);
  assert.doesNotMatch(surface, /fetch\(/);
});
