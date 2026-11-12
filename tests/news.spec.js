import { test, expect } from "./fixtures.js";
import { compileSignals } from "../src/director/news.mjs";

function fixture(version = 1, regime = "bear", classified = true) {
  const now = Date.now();
  const items = ["research", "blue_wave", "none"].map((recipe, index) => ({
    id: `fixture-${version}-${index}`,
    source: "TEST FIXTURE",
    title: `[TEST FIXTURE ${version}/${index}] ${["A new open model releases research tools for developers", "Democrats pass new oversight rules for advanced AI infrastructure", "An uncertain opinion about the future of technology"][index]}`,
    url: `https://example.test/model/${index}`,
    summary: "Synthetic browser-test news.",
    publishedAt: new Date(now).toISOString(),
    expiresAt: new Date(now + 86400000).toISOString(),
    classification: classified
      ? {
          status: "classified",
          topic: index === 1 ? "policy" : "models",
          recipe,
          confidence: index === 2 ? 0.42 : 0.94,
          topicConfidence: 0.94,
          model: "jev-fixture",
        }
      : {
          status: "unavailable",
          topic: null,
          recipe: null,
          reason: "Jev key is not configured.",
        },
  }));
  const market = {
    status: classified ? "classified" : "unavailable",
    regime,
    confidence: classified ? 0.86 : 0,
    evidence: classified ? items.slice(0, 2).map((item) => item.id) : [],
    reason: "Shared market/news assessment.",
  };
  return {
    mode: "fixture",
    checkedAt: new Date(now).toISOString(),
    refreshing: false,
    jevConfigured: classified,
    sources: [{ name: "Fixture", status: "ok" }],
    items,
    market,
    live: compileSignals(items, market, version, now),
    cache: { backend: "sqlite" },
  };
}

async function start(page) {
  await expect(page.locator("#start-button")).toBeEnabled();
  await page.evaluate(() =>
    window.addEventListener("gpu:state", ({ detail }) => {
      if (detail.phase === "upgrade") window.gpu.choose(0);
    }),
  );
  await page.locator("#start-button").click();
  await page.waitForFunction(
    () =>
      window.gpu.getState().phase === "playing" &&
      window.gpu.getState().time < 1,
  );
}

test("news starts only with play, updates current sentiment, queues through pause, and deduplicates effects", async ({
  page,
}) => {
  let data = fixture(),
    requests = 0;
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.route("**/api/news", (route) => {
    requests++;
    return route.fulfill({ json: data });
  });
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await expect(page.locator("#news-status")).toContainText("when you play");
  await expect(
    page.locator("#tab-market, [data-regime], #news-next, #news-previous"),
  ).toHaveCount(0);
  await page.evaluate(() =>
    document.dispatchEvent(new Event("visibilitychange")),
  );
  expect(requests).toBe(0);
  await start(page);
  await page.waitForFunction(
    () => window.gpu.getState().marketRegime === "bear",
  );
  expect(
    await page.evaluate(() => window.gpu.getState().xpMultiplier),
  ).toBeCloseTo(0.9);
  await expect(page.locator("#news-category")).toContainText(
    "JEV / MODELS / 94%",
  );
  await expect(page.locator("#news-application")).toContainText("IN");
  const original = await page.evaluate(() => window.gpu.getState().live.events);
  expect(original).toHaveLength(2);
  await page.locator("#news-refresh").click();
  await expect(page.locator("#news-refresh")).toBeEnabled();
  expect(await page.evaluate(() => window.gpu.getState().live.events)).toEqual(
    original,
  );
  await page.evaluate(() => window.gpu.pause());
  await page.waitForFunction(() => window.gpu.getState().phase === "paused");
  data = fixture(2, "bull");
  data.items[0].classification.recipe = "ai_stock_rally";
  data.items[1].classification.recipe = "regulatory_crackdown";
  data.live = compileSignals(data.items, data.market, 2, Date.now());
  await page.locator("#news-refresh").click();
  await expect(page.locator("#market-decision")).toContainText("QUEUED");
  expect(await page.evaluate(() => window.gpu.getState().marketRegime)).toBe(
    "bear",
  );
  expect(await page.evaluate(() => window.gpu.getState().live.events)).toEqual(
    original,
  );
  await page.locator("#resume-button").click();
  await page.waitForFunction(
    () => window.gpu.getState().marketRegime === "bull",
  );
  expect(
    await page.evaluate(() => window.gpu.getState().xpMultiplier),
  ).toBeCloseTo(1.15);
  expect(
    await page.evaluate(() => window.gpu.getState().live.events.length),
  ).toBe(4);
  await expect(page.locator("#news-toast-label")).toHaveText(
    "NEWS EFFECT / ACTIVE NOW",
    { timeout: 15000 },
  );
  await expect(page.locator("#news-toast-title")).toContainText("FIXTURE 1/0");
  await expect(page.locator("#news-title")).toContainText("FIXTURE 1/0");
  await page.screenshot({ path: "test-results/live-current-run.png" });
  await page.evaluate(() => window.gpu.menu());
  await page.waitForFunction(() => window.gpu.getState().phase === "menu");
  await expect(page.locator("#news-card")).toBeHidden();
  const count = requests;
  await page.evaluate(() =>
    document.dispatchEvent(new Event("visibilitychange")),
  );
  expect(requests).toBe(count);
  await start(page);
  await page.waitForFunction(() => window.gpu.getState().live.sequence === 2);
  expect(
    await page.evaluate(() => window.gpu.getState().live.events.length),
  ).toBe(2);
  expect(errors).toEqual([]);
});

test("headlines rotate automatically and hold for reading while gameplay continues", async ({
  page,
}) => {
  await page.clock.install();
  const data = fixture();
  data.live.stories = [];
  await page.route("**/api/news", (route) => route.fulfill({ json: data }));
  await page.goto("/");
  await start(page);
  await expect(page.locator("#news-title")).toContainText("FIXTURE 1/0");
  await page.mouse.move(5, 5);
  await page.clock.runFor(12_500);
  await expect(page.locator("#news-title")).toContainText("FIXTURE 1/1");
  await page.clock.runFor(12_500);
  await expect(page.locator("#news-title")).toContainText("FIXTURE 1/2");
  await expect(page.locator("#news-reason")).toContainText("Below 60%");
  await page.locator("#news-hold").click();
  await page.clock.runFor(13_000);
  await expect(page.locator("#news-title")).toContainText("FIXTURE 1/2");
  await expect(page.locator("#news-cadence")).toHaveText("HELD");
});

test("late responses cannot enter the lobby or a different run", async ({
  page,
}) => {
  let release;
  const gate = new Promise((resolve) => (release = resolve));
  let requests = 0;
  await page.route("**/api/news", async (route) => {
    requests++;
    if (requests === 1) await gate;
    await route.fulfill({ json: fixture(requests) }).catch(() => {});
  });
  await page.goto("/");
  await start(page);
  await expect.poll(() => requests).toBe(1);
  await page.evaluate(() => window.gpu.menu());
  await page.waitForFunction(() => window.gpu.getState().phase === "menu");
  release();
  await start(page);
  await page.waitForFunction(() => window.gpu.getState().live.sequence === 2);
  const events = await page.evaluate(() => window.gpu.getState().live.events);
  expect(events.every((event) => event.id.startsWith("fixture-2"))).toBe(true);
});

test("live timeline fits desktop, phone and landscape without scrolling", async ({
  page,
}) => {
  const data = fixture();
  for (const item of data.items) {
    item.source = "TEST FIXTURE / AI STOCKS";
    item.classification.recipe = "blue_wave";
    item.classification.confidence = 0.94;
  }
  data.live = compileSignals(data.items, data.market, 1, Date.now());
  await page.route("**/api/news", (route) => route.fulfill({ json: data }));
  await page.goto("/");
  await start(page);
  await expect(page.locator("#news-title")).toContainText("FIXTURE 1/0");
  for (const [width, height] of [
    [1440, 900],
    [1024, 600],
    [390, 844],
    [320, 568],
    [844, 390],
  ]) {
    await page.setViewportSize({ width, height });
    const compact = width <= 900 || height <= 520;
    if (compact) {
      await expect(page.locator("#compact-headline")).toContainText(
        "TEST FIXTURE",
      );
      await page.locator("#briefing-toggle").click();
    }
    const overflow = await page.locator("#world-briefing").evaluate((panel) => {
      const bounds = panel.getBoundingClientRect();
      return [...panel.querySelectorAll("*")]
        .filter((element) => element.checkVisibility())
        .filter((element) => {
          const box = element.getBoundingClientRect();
          return (
            box.bottom > bounds.bottom + 1 ||
            box.right > bounds.right + 1 ||
            box.left < bounds.left - 1 ||
            box.top < bounds.top - 1
          );
        })
        .map((element) => element.id || element.tagName);
    });
    expect(overflow).toEqual([]);
    expect(
      await page.evaluate(
        () =>
          document.documentElement.scrollHeight <= innerHeight &&
          document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await page.screenshot({ path: `test-results/news-${width}x${height}.png` });
    if (compact) await page.locator("#briefing-close").click();
  }
});

test("missing Jev credentials stay informational and reduced motion disables transitions", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.route("**/api/news", (route) =>
    route.fulfill({ json: fixture(1, "neutral", false) }),
  );
  await page.goto("/");
  await start(page);
  await expect(page.locator("#news-status")).toContainText("Jev key missing");
  await expect(page.locator("#news-category")).toContainText("UNAVAILABLE");
  await expect(page.locator("#news-reason")).toContainText("No classification");
  expect(
    await page
      .locator("#news-card")
      .evaluate((card) => card.getAnimations().length),
  ).toBe(0);
  expect(await page.evaluate(() => window.gpu.getState().marketRegime)).toBe(
    "neutral",
  );
});

test("stock news activates distinct timed player buffs and debuffs with matching icons", async ({
  page,
}) => {
  test.setTimeout(45000);
  let data;
  await page.route("**/api/news", (route) => route.fulfill({ json: data }));
  await page.goto("/");
  for (const [kind, label, damage, sequence] of [
    ["ai_stock_rally", "AI STOCK RALLY", 1.25, 1],
    ["ai_stock_crash", "AI STOCK CRASH", 0.8, 2],
  ]) {
    data = fixture(sequence, sequence === 1 ? "bull" : "bear");
    data.items = [data.items[0]];
    data.items[0].title = `[TEST FIXTURE] ${label}: AI chip shares move after earnings`;
    data.items[0].classification.recipe = kind;
    data.items[0].classification.topic = "finance";
    data.live = compileSignals(data.items, data.market, sequence, Date.now());
    await start(page);
    await expect(page.locator("#news-recipe")).toContainText(label);
    await expect(page.locator("#news-icon")).toHaveAttribute(
      "src",
      `/assets/${kind}.png`,
    );
    expect(
      await page.evaluate(() => window.gpu.getState().playerModifiers.damage),
    ).toBe(1);
    await page.waitForFunction(
      (kind) => window.gpu.getState().playerEvent === kind,
      kind,
      { timeout: 15000 },
    );
    expect(
      await page.evaluate(() => window.gpu.getState().playerModifiers.damage),
    ).toBeCloseTo(damage);
    await expect(page.locator("#news-application")).toContainText("ACTIVE");
    await expect(page.locator("#news-effect")).toContainText(
      kind === "ai_stock_rally" ? "damage +25%" : "damage -20%",
    );
    await page.screenshot({ path: `test-results/${kind}.png` });
    await page.evaluate(() => window.gpu.menu());
    await page.waitForFunction(() => window.gpu.getState().phase === "menu");
  }
});

test("new stock events replace held cards and remain readable on compact screens", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await page.evaluate(() => {
    const edition = window.gpu.getEdition();
    edition.events = [
      { kind: "ai_stock_rally", title: "TEST RALLY", at_seconds: 0.5 },
      { kind: "ai_stock_crash", title: "TEST CRASH", at_seconds: 7 },
    ];
    window.gpu.setEdition(edition);
  });
  await start(page);
  await page.waitForFunction(
    () => window.gpu.getState().playerEvent === "ai_stock_rally",
  );
  await expect(page.locator("#news-recipe")).toContainText("AI STOCK RALLY");
  await page.locator("#news-hold").click();
  await page.waitForFunction(
    () => window.gpu.getState().playerEvent === "ai_stock_crash",
  );
  await expect(page.locator("#news-application")).toHaveText(
    "REPLACED / NEW STOCK NEWS",
  );
  expect(await page.locator("#news-card").getAttribute("data-state")).toBe(
    "replaced",
  );
  await expect(page.locator('[data-event="ai_stock_rally"] small')).toHaveText(
    "REPLACED",
  );
  await expect(page.locator("#timeline-now")).toContainText("AI STOCK CRASH");
  await expect(page.locator("#timeline-now")).not.toContainText("RALLY");
  await page.locator("#news-hold").click();
  await expect(page.locator("#news-recipe")).toContainText("AI STOCK CRASH");
  for (const [width, height] of [
    [320, 568],
    [844, 390],
  ]) {
    await page.setViewportSize({ width, height });
    await page.locator("#briefing-toggle").click();
    await expect(page.locator("#news-icon")).toBeVisible();
    const fits = await page.locator("#news-card").evaluate((card) => {
      const bounds = document
        .querySelector("#world-briefing")
        .getBoundingClientRect();
      const box = card.getBoundingClientRect();
      return (
        box.bottom <= bounds.bottom &&
        box.right <= bounds.right &&
        document.documentElement.scrollHeight <= innerHeight
      );
    });
    expect(fits).toBe(true);
    await page.screenshot({
      path: `test-results/stock-${width}x${height}.png`,
    });
    await page.locator("#briefing-close").click();
  }
});

test("news toasts clear the native modifier HUD across arena sizes", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await page.evaluate(() => {
    const edition = window.gpu.getEdition();
    edition.events = [
      {
        kind: "ai_stock_rally",
        title: "TEST STOCK RALLY: AI chip shares surge following earnings",
        at_seconds: 0.5,
      },
    ];
    window.gpu.setEdition(edition);
  });
  await start(page);
  await expect(page.locator("#news-toast")).toBeVisible();
  for (const [width, height, fullscreen] of [
    [1440, 900],
    [1440, 1000],
    [1024, 600],
    [390, 844],
    [320, 568],
    [844, 390],
    [1920, 1080, true],
  ]) {
    await page.setViewportSize({ width, height });
    if (fullscreen) {
      await page.locator("#fullscreen").click();
      await page.waitForFunction(
        () => document.fullscreenElement?.id === "game-viewport",
      );
    }
    await expect(page.locator("#news-toast")).toBeVisible();
    const layout = await page.locator("#news-toast").evaluate((toast) => {
      const canvas = document
        .querySelector("#glcanvas")
        .getBoundingClientRect();
      const box = toast.getBoundingClientRect();
      const scale = Math.min(canvas.width / 640, canvas.height / 360);
      const left = canvas.left + (canvas.width - 640 * scale) / 2;
      const top = canvas.top + (canvas.height - 360 * scale) / 2;
      // Renderer::hud draws thermals at y=292 and modifiers through y=339.
      const hud = {
        left: left + 8 * scale,
        right: left + 310 * scale,
        top: top + 292 * scale,
        bottom: top + 339 * scale,
      };
      return {
        gap: hud.top - box.bottom,
        clear:
          box.bottom <= hud.top - 8 ||
          box.top >= hud.bottom + 8 ||
          box.right <= hud.left - 8 ||
          box.left >= hud.right + 8,
        contained:
          box.top >= canvas.top &&
          box.bottom <= canvas.bottom &&
          box.left >= canvas.left &&
          box.right <= canvas.right,
        canvas: { width: canvas.width, height: canvas.height },
      };
    });
    expect
      .soft(layout.clear, `${width}x${height}: ${JSON.stringify(layout)}`)
      .toBe(true);
    expect
      .soft(
        layout.contained,
        `${width}x${height}: toast must stay inside the arena`,
      )
      .toBe(true);
    await page.screenshot({
      path: `test-results/toast-${width}x${height}${fullscreen ? "-fullscreen" : ""}.png`,
    });
  }
});
