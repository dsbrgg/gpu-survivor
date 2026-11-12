import { test, expect } from "./fixtures.js";

const viewports = [
  [1440, 900],
  [1366, 768],
  [1280, 600],
  [1024, 768],
  [800, 600],
  [390, 844],
  [844, 390],
  [320, 568],
];

async function expectViewportFit(page) {
  await expect
    .poll(() =>
      page.evaluate(() => {
        const canvas = document.querySelector("#glcanvas");
        return (
          Math.abs(
            canvas.width - Math.floor(canvas.clientWidth * devicePixelRatio),
          ) +
          Math.abs(
            canvas.height - Math.floor(canvas.clientHeight * devicePixelRatio),
          )
        );
      }),
    )
    .toBeLessThanOrEqual(2);
  const issues = await page.evaluate(() => {
    const issues = [];
    const root = document.documentElement;
    if (root.scrollHeight > innerHeight || root.scrollWidth > innerWidth)
      issues.push("document scrolls");
    for (const element of document.querySelectorAll(".site-shell *")) {
      if (!element.checkVisibility()) continue;
      const box = element.getBoundingClientRect();
      const name = element.id || element.className || element.tagName;
      if (
        box.left < -1 ||
        box.top < -1 ||
        box.right > innerWidth + 1 ||
        box.bottom > innerHeight + 1
      ) {
        issues.push(`${name} outside viewport`);
      }
      const overlay = element.closest(".overlay");
      if (overlay) {
        const bounds = overlay.getBoundingClientRect();
        if (
          box.left < bounds.left - 1 ||
          box.top < bounds.top - 1 ||
          box.right > bounds.right + 1 ||
          box.bottom > bounds.bottom + 1
        ) {
          issues.push(`${name} clipped by overlay`);
        }
      }
      if (
        element.matches(
          ".overlay, .edition-panel, .founder-card, .upgrade-card",
        ) &&
        (element.scrollHeight > element.clientHeight + 2 ||
          element.scrollWidth > element.clientWidth + 2)
      ) {
        issues.push(`${name} content overflows`);
      }
    }
    return issues;
  });
  expect(issues).toEqual([]);
}

async function checkViewportSizes(page, phase) {
  for (const [width, height] of viewports) {
    await page.setViewportSize({ width, height });
    await expectViewportFit(page);
    if (phase)
      await page.screenshot({
        path: `test-results/${phase}-${width}x${height}.png`,
      });
  }
}

test("browser boots WASM, plays, pauses, resumes, and pins its edition", async ({
  page,
}) => {
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled({ timeout: 30000 });
  await page.screenshot({ path: "test-results/lobby.png", fullPage: true });
  await page.locator("#start-button").click();
  await page.waitForFunction(() => window.gpu.getState().phase === "playing");
  const start = await page.evaluate(() => window.gpu.getState());
  expect(start.edition).toBe("live-news-v1");
  await page.keyboard.down("d");
  await page.waitForFunction(
    (x) =>
      window.gpu.getState().position.x > x + 80 ||
      window.gpu.getState().phase === "upgrade",
    start.position.x,
  );
  await page.keyboard.up("d");
  if (await page.evaluate(() => window.gpu.getState().phase === "upgrade")) {
    await page.locator('[data-upgrade="0"]').click();
    await page.waitForFunction(() => window.gpu.getState().phase === "playing");
  }
  expect(await page.evaluate(() => window.gpu.getState().gpus)).toBeGreaterThan(
    0,
  );
  await page.keyboard.press("Space");
  await page.waitForFunction(() => window.gpu.getState().dashCooldown > 1);
  await page.keyboard.press("Escape");
  await expect(page.locator("#pause-overlay")).toBeVisible();
  const paused = await page.evaluate(() => window.gpu.getState().time);
  expect(
    await page.evaluate(() => {
      try {
        window.gpu.setEdition(window.gpu.getEdition());
        return false;
      } catch {
        return true;
      }
    }),
  ).toBe(true);
  await page.screenshot({ path: "test-results/paused.png", fullPage: true });
  expect(await page.evaluate(() => window.gpu.getState().time)).toBe(paused);
  await page.locator("#resume-button").click();
  await page.waitForFunction(
    (time) => window.gpu.getState().time > time + 3,
    paused,
  );
  await page.screenshot({ path: "test-results/gameplay.png", fullPage: true });
  expect(errors).toEqual([]);
});

test("Sam collects GPUs, receives a real upgrade, and resumes combat", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled({ timeout: 30000 });
  await page.locator('[data-character="1"]').click();
  await page.locator("#start-button").click();
  await page.waitForFunction(() => window.gpu.getState().phase === "playing");
  await page.keyboard.down("d");
  await page.waitForFunction(
    () => window.gpu.getState().phase === "upgrade",
    null,
    {
      timeout: 15000,
    },
  );
  await page.keyboard.up("d");
  await expect(page.locator("#upgrade-cards button")).toHaveCount(3);
  await page.screenshot({ path: "test-results/upgrades.png", fullPage: true });
  const choice = await page.evaluate(
    () => window.gpu.getState().choices[0].kind,
  );
  await page.locator('[data-upgrade="0"]').click();
  await page.waitForFunction(() => window.gpu.getState().phase === "playing");
  expect(await page.evaluate(() => window.gpu.getState().character)).toBe(1);
  expect(choice).toBe("wider_context");
  expect(
    await page.evaluate(() =>
      window.gpu.getState().upgrades.reduce((sum, rank) => sum + rank, 0),
    ),
  ).toBe(1);
  expect(
    await page.evaluate(() => window.gpu.getState().attackRadiusMultiplier),
  ).toBeCloseTo(1.2);
  await page.keyboard.press("e");
  await page.waitForFunction(() => window.gpu.getState().abilityCooldown > 10);
  await expect(page.locator("#error-overlay")).toBeHidden();
});

test("invalid edition cannot start a run", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled({ timeout: 30000 });
  await page.evaluate(() => {
    const edition = window.gpu.getEdition();
    edition.threat_multiplier = 99;
    window.gpu.setEdition(edition);
    window.gpu.start();
  });
  await expect(page.locator("#error-overlay")).toBeVisible();
  expect(await page.evaluate(() => window.gpu.getState().phase)).toBe("menu");
});

test("AI Bubble joins surviving enemies at the deadline and replay resets simulation", async ({
  page,
}) => {
  test.setTimeout(90000);
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled({ timeout: 30000 });
  await page.evaluate(() => {
    const edition = window.gpu.getEdition();
    edition.id = "short-browser-test";
    edition.duration_seconds = 30;
    edition.boss_at_seconds = 30;
    edition.events = [
      { at_seconds: 29, kind: "funding", title: "FINAL FUNDING ROUND" },
    ];
    window.gpu.setEdition(edition);
    window.addEventListener("gpu:state", ({ detail }) => {
      if (detail.phase === "upgrade") window.gpu.choose(0);
    });
    window.gpu.start();
  });
  await page.waitForFunction(() => window.gpu.getState().phase === "playing");
  await page.keyboard.down("w");
  await page.waitForFunction(() => window.gpu.getState().bossSpawned, null, {
    timeout: 40000,
  });
  await page.keyboard.up("w");
  expect(await page.evaluate(() => window.gpu.getState().boss.kind)).toBe(
    "bubble",
  );
  expect(
    await page.evaluate(
      () =>
        window.gpu.getState().rivals.filter((kind) => kind !== "bubble").length,
    ),
  ).toBeGreaterThan(0);
  expect(
    await page.evaluate(() => window.gpu.getState().time),
  ).toBeGreaterThanOrEqual(30);
  await page.waitForFunction(() => {
    const state = window.gpu.getState();
    return state.time >= 34 || ["won", "lost"].includes(state.phase);
  });
  await page.screenshot({ path: "test-results/boss.png", fullPage: true });
  await expect(page.locator("#result-overlay")).toBeVisible({ timeout: 35000 });
  const finished = await page.evaluate(() => window.gpu.getState());
  expect(["lost", "won"]).toContain(finished.phase);
  expect(finished.time).toBeGreaterThanOrEqual(30);
  await page.screenshot({ path: "test-results/result.png", fullPage: true });
  await checkViewportSizes(page, "result");
  await page.locator("#retry-button").click();
  await page.waitForFunction(() => window.gpu.getState().phase === "playing");
  const restarted = await page.evaluate(() => window.gpu.getState());
  expect(restarted.time).toBeLessThan(1);
  expect(restarted.level).toBe(1);
  expect(restarted.bossSpawned).toBe(false);
  expect(errors).toEqual([]);
});

test("lobby, combat, pause, upgrades, and errors fit the viewport after resizing", async ({
  page,
}) => {
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await checkViewportSizes(page, "lobby");
  await page.locator("#briefing-toggle").click();
  await page.locator("#tab-sources").click();
  await page.locator("#briefing-close").click();
  await page.locator("#start-button").click();
  await page.waitForFunction(() => window.gpu.getState().phase === "playing");
  await expectViewportFit(page);
  await page.keyboard.press("Escape");
  await expect(page.locator("#pause-overlay")).toBeVisible();
  await checkViewportSizes(page, "pause");
  await page.locator("#resume-button").click();
  await page.keyboard.down("d");
  await page.waitForFunction(() => window.gpu.getState().phase === "upgrade");
  await page.keyboard.up("d");
  await checkViewportSizes(page, "upgrade");
  await page.locator('[data-upgrade="0"]').click();
  await page.evaluate(() => window.gpu.menu());
  await page.waitForFunction(() => window.gpu.getState().phase === "menu");
  await page.evaluate(() => {
    window.gpu.setEdition({
      ...window.gpu.getEdition(),
      threat_multiplier: 99,
    });
    window.gpu.start();
  });
  await expect(page.locator("#error-overlay")).toBeVisible();
  await checkViewportSizes(page);
  expect(errors).toEqual([]);
});

test("keyboard movement selects eight facings and keeps the last facing at rest", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await page.evaluate(() => {
    window.addEventListener("gpu:state", ({ detail }) => {
      if (detail.phase === "upgrade") window.gpu.choose(0);
    });
    window.gpu.start();
  });
  const directions = [
    ["up", ["w"]],
    ["up_right", ["w", "d"]],
    ["right", ["d"]],
    ["down_right", ["s", "d"]],
    ["down", ["s"]],
    ["down_left", ["s", "a"]],
    ["left", ["a"]],
    ["up_left", ["w", "a"]],
  ];
  for (const [facing, keys] of directions) {
    for (const key of keys) await page.keyboard.down(key);
    await page.waitForFunction((facing) => {
      const state = window.gpu.getState();
      return state.moving && state.facing === facing;
    }, facing);
    for (const key of keys) await page.keyboard.up(key);
    await page.waitForFunction(() => !window.gpu.getState().moving);
    expect(await page.evaluate(() => window.gpu.getState().facing)).toBe(
      facing,
    );
    await page.screenshot({ path: `test-results/facing-${facing}.png` });
  }
  await page.keyboard.press("Space");
  await page.waitForFunction(() => window.gpu.getState().dashCooldown > 1);
  expect(await page.evaluate(() => window.gpu.getState().facing)).toBe(
    "up_left",
  );
});
