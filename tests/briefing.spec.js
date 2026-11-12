import { test, expect } from "./fixtures.js";

async function expectPanelFits(page) {
  const overflow = await page.locator(".edition-panel").evaluate((panel) => {
    const bounds = panel.getBoundingClientRect();
    return [...panel.querySelectorAll("*")]
      .filter((element) => element.checkVisibility())
      .filter((element) => {
        const rect = element.getBoundingClientRect();
        return (
          rect.bottom > bounds.bottom + 1 ||
          rect.right > bounds.right + 1 ||
          rect.top < bounds.top - 1 ||
          rect.left < bounds.left - 1
        );
      })
      .map((element) => element.id || element.className || element.tagName);
  });
  expect(overflow).toEqual([]);
  expect(
    await page.evaluate(() => document.documentElement.scrollHeight),
  ).toBeLessThanOrEqual(await page.evaluate(() => innerHeight));
}

test("briefing contents stay within the panel at intermediate desktop heights", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  for (const [width, height] of [
    [1536, 864],
    [1366, 851],
    [1024, 600],
  ]) {
    await page.setViewportSize({ width, height });
    for (const tab of ["news", "sources"]) {
      await page.locator(`#tab-${tab}`).click();
      await expectPanelFits(page);
    }
  }
});

test("compact screens show an automatic dispatch and a viewport-fit briefing", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  for (const [width, height] of [
    [800, 600],
    [390, 844],
    [844, 390],
    [320, 568],
  ]) {
    await page.setViewportSize({ width, height });
    await expect(page.locator("#compact-headline")).toBeVisible();
    await page.locator("#briefing-toggle").click();
    await page.locator("#tab-news").click();
    await expect(page.locator("#news-timeline li")).toHaveCount(0);
    await expect(page.locator("#news-status")).toContainText("when you play");
    await expectPanelFits(page);
    await page.locator("#tab-sources").click();
    await expect(page.locator("#edition-provenance")).toContainText(
      "Shared RSS",
    );
    await expect(page.locator(".source-list a")).toHaveCount(6);
    await expectPanelFits(page);
    await page.keyboard.press("Escape");
    await expect(page.locator("#world-briefing")).toBeHidden();
  }
});

test("Blue Wave edition activates a bounded themed event and briefing pauses combat", async ({
  page,
}) => {
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await page.evaluate(async () =>
    window.gpu.setEdition(
      await (await fetch("/editions/blue-wave.json")).json(),
    ),
  );
  await page.locator("#start-button").click();
  await page.waitForFunction(
    () => window.gpu.getState().blueWaveRemaining > 0,
    null,
    { timeout: 20000 },
  );
  const active = await page.evaluate(() => window.gpu.getState());
  expect(active.edition).toBe("blue-wave-preview-v1");
  expect(active.effectiveThreat).toBeCloseTo(1.2);
  expect(active.inspectorCount).toBeGreaterThan(0);
  await expect(page.locator("#news-application")).toContainText("ACTIVE");
  await expect(page.locator("#news-icon")).toHaveAttribute(
    "src",
    "/assets/democrats.png",
  );
  await page.screenshot({ path: "test-results/blue-wave-active.png" });
  await page.setViewportSize({ width: 844, height: 390 });
  await page.locator("#briefing-toggle").click();
  await page.waitForFunction(() => window.gpu.getState().phase === "paused");
  const remaining = await page.evaluate(
    () => window.gpu.getState().blueWaveRemaining,
  );
  await expectPanelFits(page);
  await page.screenshot({ path: "test-results/blue-wave-briefing.png" });
  expect(
    await page.evaluate(() => window.gpu.getState().blueWaveRemaining),
  ).toBe(remaining);
  await page.locator("#briefing-close").click();
  await page.waitForFunction(
    (time) =>
      window.gpu.getState().phase === "playing" &&
      window.gpu.getState().blueWaveRemaining < time,
    remaining,
  );
  await page.evaluate(() => window.gpu.menu());
  await page.waitForFunction(() => window.gpu.getState().phase === "menu");
  await page.evaluate(() => window.gpu.start());
  await page.waitForFunction(
    () =>
      window.gpu.getState().phase === "playing" &&
      window.gpu.getState().time < 1,
  );
  expect(await page.evaluate(() => window.gpu.getState().inspectorCount)).toBe(
    0,
  );
  expect(errors).toEqual([]);
});
