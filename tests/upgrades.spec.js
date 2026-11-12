import { test, expect } from "./fixtures.js";

const signatures = [
  [0, "leather_jacket", "LEATHER JACKET", "leather_jacket"],
  [1, "capital_injection", "CAPITAL INJECTION", "capital"],
  [2, "safety_rails", "SAFETY RAILS", "safety_rail"],
  [3, "cybertruck", "CYBERTRUCK", "cybertruck"],
];

for (const [character, kind, label, projectile] of signatures) {
  test(`${label} unlocks beside the starter weapon and resets with the run`, async ({
    page,
  }) => {
    const errors = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto("/");
    await expect(page.locator("#start-button")).toBeEnabled();
    await page.evaluate((character) => {
      const edition = window.gpu.getEdition();
      window.gpu.setEdition({
        ...edition,
        id: "weapon-test",
        seed: 7,
        events: [],
      });
      window.gpu.start(character);
    }, character);
    await page.waitForFunction(() => window.gpu.getState().phase === "playing");
    await page.keyboard.down("d");
    await page.waitForFunction(() => window.gpu.getState().phase === "upgrade");
    await page.keyboard.up("d");
    await expect(page.locator('[data-kind="wider_context"]')).toBeVisible();
    await expect(page.locator(`[data-kind="${kind}"]`)).toContainText(label);
    const choices = await page.evaluate(() =>
      window.gpu.getState().choices.map((choice) => choice.kind),
    );
    expect(
      choices.filter((choice) =>
        signatures.some(([, signature]) => choice === signature),
      ),
    ).toEqual([kind]);
    for (const [width, height] of [
      [1440, 900],
      [390, 844],
      [844, 390],
    ]) {
      await page.setViewportSize({ width, height });
      const clipped = await page
        .locator(".upgrade-card")
        .evaluateAll((cards) =>
          cards.some(
            (card) =>
              card.scrollHeight > card.clientHeight + 2 ||
              card.scrollWidth > card.clientWidth + 2,
          ),
        );
      expect(clipped).toBe(false);
    }
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.screenshot({ path: `test-results/${kind}-upgrade.png` });
    await page.locator(`[data-kind="${kind}"]`).click();
    await page.waitForFunction(
      () => window.gpu.getState().signature.rank === 1,
    );
    await page.evaluate(() =>
      window.addEventListener("gpu:state", ({ detail }) => {
        if (detail.phase === "upgrade") window.gpu.choose(0);
      }),
    );
    await page.waitForFunction(
      (projectile) => {
        const shots = window.gpu.getState().friendlyProjectiles;
        return shots.includes("founder") && shots.includes(projectile);
      },
      projectile,
      { timeout: 20000 },
    );
    await page.screenshot({ path: `test-results/${kind}-active.png` });
    await page.evaluate(() => window.gpu.menu());
    await page.waitForFunction(() => window.gpu.getState().phase === "menu");
    await page.evaluate(() => window.gpu.start());
    await page.waitForFunction(() => window.gpu.getState().phase === "playing");
    expect(
      await page.evaluate(() => window.gpu.getState().signature.rank),
    ).toBe(0);
    expect(
      await page.evaluate(() => window.gpu.getState().attackRadiusMultiplier),
    ).toBe(1);
    expect(errors).toEqual([]);
  });
}
