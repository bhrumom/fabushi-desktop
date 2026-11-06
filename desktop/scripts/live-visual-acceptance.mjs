import { chromium } from "playwright";
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { existsSync } from "node:fs";

const appPath = "/Applications/Fabushi.app/Contents/MacOS/Fabushi";
const screenshotPath = "/Users/bhrum/.gemini/antigravity/brain/1e5f0704-2329-4a7e-82df-96c3c8162362/fabushi-3segment-v3-live.png";
const fusedScreenshotPath = "/Users/bhrum/.gemini/antigravity/brain/1e5f0704-2329-4a7e-82df-96c3c8162362/fabushi-openclaw-telegram-fused-live.png";
const computerScreenshotPath = "/Users/bhrum/.gemini/antigravity/brain/1e5f0704-2329-4a7e-82df-96c3c8162362/fabushi-computer-workbench-live.png";
const port = 14333;

async function main() {
  console.log("Checking executable:", appPath);
  if (!existsSync(appPath)) {
    throw new Error(`Executable not found: ${appPath}`);
  }

  console.log("Spawning Fabushi with --remote-debugging-port=" + port);
  const child = spawn(appPath, [`--remote-debugging-port=${port}`, "--remote-debugging-address=127.0.0.1"], {
    detached: true,
    stdio: "ignore",
  });
  child.unref();

  console.log("Waiting for CDP endpoint...");
  let pageTarget = null;
  for (let i = 0; i < 40; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/list`);
      if (res.ok) {
        const targets = await res.json();
        console.log(`Found ${targets.length} targets:`, targets.map(t => ({ type: t.type, title: t.title, url: t.url })));
        pageTarget = targets.find(t => t.type === "page" || t.url?.includes("index.html"));
        if (pageTarget) break;
      }
    } catch {}
    await new Promise(r => setTimeout(r, 500));
  }

  if (!pageTarget) {
    throw new Error("Could not find page target via CDP");
  }

  console.log("Connecting Playwright over CDP to port", port);
  const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
  const contexts = browser.contexts();
  const pages = contexts.flatMap(c => c.pages());
  console.log(`Found ${pages.length} pages in browser contexts`);
  const page = pages[0] || await browser.newPage();

  console.log("Page URL:", page.url());

  // Wait for React to render workspace
  console.log("Waiting for #root or workspace...");
  await page.waitForSelector("#root", { timeout: 15000 });
  await page.waitForTimeout(2000);

  // Check 3-segment shell elements
  console.log("Checking 3-segment shell elements...");
  const navRail = page.locator('[data-testid="fabushi-primary-nav-rail"]');
  const navRailVisible = await navRail.isVisible().catch(() => false);
  console.log("fabushi-primary-nav-rail visible:", navRailVisible);

  const filterBar = page.locator('[data-testid="fabushi-collection-filter-bar"]');
  const filterBarVisible = await filterBar.isVisible().catch(() => false);
  console.log("fabushi-collection-filter-bar visible:", filterBarVisible);

  const storiesBar = page.locator('[data-testid="tdrp-stories-stars-bar"]');
  const storiesBarVisible = await storiesBar.isVisible().catch(() => false);
  console.log("tdrp-stories-stars-bar visible:", storiesBarVisible);

  const composerToolbar = page.locator('[data-testid="tdrp-composer-toolbar"]');
  const composerToolbarVisible = await composerToolbar.isVisible().catch(() => false);
  console.log("tdrp-composer-toolbar visible:", composerToolbarVisible);

  const sessionToolbar = page.locator('[data-testid="fabushi-agent-session-toolbar"]');
  const sessionToolbarVisible = await sessionToolbar.isVisible().catch(() => false);
  console.log("fabushi-agent-session-toolbar visible:", sessionToolbarVisible);

  // Test Computer Domain click & LocalComputerWorkbench
  console.log("Testing Computer domain transition...");
  const computerNavButton = page.locator('[data-testid="fabushi-nav-computer"]');
  if (await computerNavButton.isVisible().catch(() => false)) {
    console.log("Clicking Computer nav item...");
    await computerNavButton.click();
    await page.waitForTimeout(1000);
    const workbench = page.locator('[data-testid="local-computer-workbench"]');
    const workbenchVisible = await workbench.isVisible().catch(() => false);
    console.log("local-computer-workbench visible:", workbenchVisible);
    if (workbenchVisible) {
      console.log("Capturing Computer Workbench screenshot to:", computerScreenshotPath);
      await page.screenshot({ path: computerScreenshotPath, fullPage: false });
    }
    const backButton = page.locator('[data-testid="local-workbench-back-to-chat"]');
    if (await backButton.isVisible().catch(() => false)) {
      console.log("Clicking Back to Chat button...");
      await backButton.click();
      await page.waitForTimeout(1000);
    }
  }

  // Take main screenshot
  console.log("Capturing live screenshot to:", screenshotPath);
  await page.screenshot({ path: screenshotPath, fullPage: false });
  console.log("Capturing fused live screenshot to:", fusedScreenshotPath);
  await page.screenshot({ path: fusedScreenshotPath, fullPage: false });
  console.log("Screenshots saved successfully!");

  // Disconnect CDP leaving app open and running
  await browser.close();
  console.log("Playwright disconnected. /Applications/Fabushi.app remains open and running for the user.");
}

main().catch(err => {
  console.error("Verification failed:", err);
  process.exit(1);
});
