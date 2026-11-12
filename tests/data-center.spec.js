import { test, expect } from "./fixtures.js";

test("all four founder cards start the correct playable character", async ({
  page,
}) => {
  test.setTimeout(60000);
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await expect(page.locator("[data-character]")).toHaveCount(4);
  for (const [id, company] of [
    "NVIDIA",
    "OpenAI",
    "Anthropic",
    "xAI",
  ].entries()) {
    await page.locator(`[data-character="${id}"]`).click();
    await expect(page.locator(`[data-character="${id}"]`)).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    await page.locator("#start-button").click();
    await page.waitForFunction(
      (id) =>
        window.gpu.getState().phase === "playing" &&
        window.gpu.getState().character === id,
      id,
    );
    await page.waitForFunction(() => window.gpu.getState().projectileCount > 0);
    const state = await page.evaluate(() => window.gpu.getState());
    expect(state.company).toBe(company);
    expect(state.loadedChunks).toBe(25);
    await page.screenshot({ path: `test-results/founder-${id}-logos.png` });
    await page.keyboard.press("e");
    await page.waitForFunction(
      () => window.gpu.getState().abilityCooldown > 10,
    );
    await page.evaluate(() => window.gpu.menu());
    await page.waitForFunction(() => window.gpu.getState().phase === "menu");
  }
  expect(errors).toEqual([]);
});

test("thermal zones activate across streamed chunks and pause freezes exposure", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator("#start-button")).toBeEnabled();
  await page.evaluate(() => {
    window.addEventListener("gpu:state", ({ detail }) => {
      if (detail.phase === "upgrade") window.gpu.choose(0);
    });
    window.gpu.start(2);
  });
  await page.keyboard.down("d");
  await page.waitForFunction(() => window.gpu.getState().position.x > 480);
  await page.keyboard.up("d");
  await page.waitForFunction(() => window.gpu.getState().heatExposure > 2);
  const hot = await page.evaluate(() => window.gpu.getState());
  expect(hot.climate).toBe("hot");
  expect(hot.hp).toBeLessThan(100);
  expect(hot.heatDamagePerSecond).toBeGreaterThan(3);
  expect(hot.chunk).toEqual([1, 0]);
  expect(hot.loadedChunks).toBe(25);
  await page.screenshot({ path: "test-results/data-center-hot.png" });
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => window.gpu.getState().phase === "paused");
  const paused = await page.evaluate(() => window.gpu.getState());
  await expect(page.locator("#pause-overlay")).toBeVisible();
  expect(await page.evaluate(() => window.gpu.getState().heatExposure)).toBe(
    paused.heatExposure,
  );
  await page.locator("#resume-button").click();
  await page.keyboard.down("a");
  await page.waitForFunction(() => window.gpu.getState().position.x < 370);
  await page.keyboard.up("a");
  await page.waitForFunction(() => window.gpu.getState().heatExposure === 0);
  expect(
    await page.evaluate(() => window.gpu.getState().heatDamagePerSecond),
  ).toBe(0);
  await page.evaluate(() => window.gpu.menu());
  await page.waitForFunction(() => window.gpu.getState().phase === "menu");
  await page.evaluate(() => window.gpu.start(3));
  await page.keyboard.down("a");
  await page.waitForFunction(() => window.gpu.getState().climate === "cold");
  await page.keyboard.up("a");
  const cold = await page.evaluate(() => window.gpu.getState());
  expect(cold.chunk).toEqual([-1, 0]);
  expect(cold.movementMultiplier).toBeCloseTo(0.62);
  expect(cold.heatDamagePerSecond).toBe(0);
  await page.screenshot({ path: "test-results/data-center-cold.png" });
});
