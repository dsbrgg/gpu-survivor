import { test as base, expect } from "@playwright/test";

export { expect };
export const test = base.extend({
  offlineNews: [
    async ({ page }, use) => {
      await page.route("**/api/news", (route) =>
        route.fulfill({
          json: {
            mode: "fixture",
            checkedAt: new Date().toISOString(),
            refreshing: false,
            jevConfigured: false,
            sources: [],
            items: [],
            market: {
              status: "unavailable",
              regime: "neutral",
              confidence: 0,
              evidence: [],
              reason: "Fixture offline",
            },
            live: {
              revision: "offline",
              sequence: 1,
              regime: "neutral",
              confidence: 0,
              stories: [],
            },
          },
        }),
      );
      await use();
    },
    { auto: true },
  ],
});
