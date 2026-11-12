import { test } from "node:test";
import assert from "node:assert/strict";
import {
  parseFeed,
  classifyArticle,
  compileSignals,
  classifyMarket,
  marketEvidence,
  NewsDirector,
  REFRESH_MS,
} from "./news.mjs";
import { DecisionCache } from "./cache.mjs";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const now = Date.parse("2026-11-12T12:00:00Z");
const source = { name: "Fixture", url: "https://example.test/feed" };
const xml = `<rss><channel><item><title><![CDATA[New model &#8212; open weights]]></title><link>https://example.test/release?utm_source=feed</link><pubDate>Thu, 12 Nov 2026 10:00:00 GMT</pubDate><description><![CDATA[<p>A released model.</p>]]></description></item><item><title>Bad link</title><link>javascript:alert(1)</link><pubDate>Thu, 12 Nov 2026 10:00:00 GMT</pubDate></item></channel></rss>`;
const answer = (value, confidence = 0.9) => ({
  type: "choice",
  choice: value,
  confidence,
  probabilities: { [value]: 1 },
});
const result = (recipe = "research", confidence = 0.9) => ({
  model: "jev-test",
  answers: {
    topic: answer("models", confidence),
    recipe: answer(recipe, confidence),
  },
});

test("feed extraction decodes excerpts, normalizes links, and rejects invalid entries", () => {
  const items = parseFeed(xml, source, now);
  assert.equal(items.length, 1);
  assert.equal(items[0].title, "New model — open weights");
  assert.equal(items[0].summary, "A released model.");
  assert.equal(items[0].url, "https://example.test/release");
  assert.throws(() => parseFeed("<!DOCTYPE rss><rss/>", source, now));
  const atom =
    '<feed><entry><title>Model</title><link rel="self" href="https://example.test/self"/><link rel="alternate" href="https://example.test/model"/><updated>2026-11-12T10:00:00Z</updated></entry></feed>';
  assert.equal(
    parseFeed(atom, source, now)[0].url,
    "https://example.test/model",
  );
});

test("Jev decisions use the documented contract and reject unsupported or uncertain effects", async () => {
  const article = parseFeed(xml, source, now)[0];
  let payload = result();
  const fetcher = async (url, options) => {
    assert.equal(url, "https://api.typesafe.ai/v1/systemone");
    assert.equal(options.headers.Authorization, "Bearer fixture-only");
    const request = JSON.parse(options.body);
    assert.equal(request.model, "jev-latest");
    assert.equal(request.questions.recipe.type, "choice");
    assert.equal(request.state.title, article.title);
    return Response.json(payload);
  };
  assert.equal(
    (await classifyArticle(article, { apiKey: "fixture-only", fetcher }))
      .recipe,
    "research",
  );
  payload = result("research", 0.3);
  assert.equal(
    (await classifyArticle(article, { apiKey: "fixture-only", fetcher }))
      .recipe,
    "none",
  );
  payload = result("execute_code");
  await assert.rejects(
    classifyArticle(article, { apiKey: "fixture-only", fetcher }),
    /Invalid Jev/,
  );
  assert.equal((await classifyArticle(article, {})).status, "unavailable");
});

test("live signals retain identities and exclude expired, uncertain and unsupported stories", () => {
  const item = parseFeed(xml, source, now)[0];
  const items = [
    "research",
    "funding",
    "blue_wave",
    "blackout",
    "research",
  ].map((recipe, index) => ({
    ...item,
    id: String(index),
    classification: { status: "classified", recipe, confidence: 0.9 },
  }));
  const market = {
    status: "classified",
    regime: "bear",
    confidence: 0.8,
    evidence: [],
  };
  const first = compileSignals(items, market, now, now);
  assert.deepEqual(first, compileSignals(items, market, now, now));
  assert.equal(first.stories.length, 4);
  assert.equal(first.regime, "bear");
  assert.ok(
    first.stories.every(
      (event) => event.title.length <= 64 && /^[\x20-\x7e]*$/.test(event.title),
    ),
  );
  assert.equal(
    compileSignals(items, market, now, now + 4 * 86400000).stories.length,
    0,
  );
  items[0].classification.confidence = 0.2;
  items[1].classification.recipe = "execute_code";
  assert.equal(compileSignals(items, market, now, now).stories.length, 3);
});

test("ordinary policy news cannot become a Democratic Blue Wave", async () => {
  const article = {
    ...parseFeed(xml, source, now)[0],
    title: "Jensen Huang agrees to testify before Congress",
    summary: "A hearing about AI chip export controls.",
  };
  const fetcher = async () =>
    Response.json({
      model: "fixture",
      answers: {
        topic: answer("policy"),
        recipe: answer("blue_wave"),
      },
    });
  const unrelated = await classifyArticle(article, {
    apiKey: "fixture-only",
    fetcher,
  });
  assert.equal(unrelated.recipe, "none");
  assert.match(unrelated.reason, /Democratic Party/);
  article.title = "Democrats win control of Congress after election";
  assert.equal(
    (await classifyArticle(article, { apiKey: "fixture-only", fetcher }))
      .recipe,
    "blue_wave",
  );
});

test("a policy-heavy feed preserves room for other supported event kinds", () => {
  const article = parseFeed(xml, source, now)[0];
  const items = [
    "blue_wave",
    "blue_wave",
    "blue_wave",
    "funding",
    "research",
  ].map((recipe, i) => ({
    ...article,
    id: String(i),
    classification: { status: "classified", confidence: 0.9, recipe },
  }));
  const signals = compileSignals(
    items,
    { status: "classified", regime: "neutral", confidence: 0.9, evidence: [] },
    1,
    now,
  );
  assert.deepEqual(
    signals.stories.map((story) => story.kind),
    ["blue_wave", "funding", "research"],
  );
});

test("Jev stock and audit choices survive validation and signal compilation", async () => {
  const article = parseFeed(xml, source, now)[0];
  for (const recipe of [
    "regulatory_crackdown",
    "ai_stock_crash",
    "ai_stock_rally",
  ]) {
    const classification = await classifyArticle(article, {
      apiKey: "fixture-only",
      fetcher: async () =>
        Response.json({
          model: "fixture",
          answers: {
            topic: answer(
              recipe === "regulatory_crackdown" ? "policy" : "finance",
            ),
            recipe: answer(recipe),
          },
        }),
    });
    assert.equal(classification.recipe, recipe);
    assert.equal(
      compileSignals(
        [{ ...article, classification }],
        {
          status: "classified",
          regime: "neutral",
          confidence: 0.9,
          evidence: [],
        },
        1,
        now,
      ).stories[0].kind,
      recipe,
    );
  }
});

test("concurrent readers share refreshes, cache Jev results, and survive partial source outages", async () => {
  let current = now;
  let feeds = 0;
  let classifications = 0;
  const fetcher = async (url, options) => {
    if (url === source.url) {
      feeds++;
      assert.equal(options.headers.Authorization, undefined);
      return new Response(xml);
    }
    if (url.endsWith("broken"))
      return new Response("Unavailable", { status: 503 });
    classifications++;
    return Response.json(
      JSON.parse(options.body).questions.regime
        ? { model: "jev-test", answers: { regime: answer("bull") } }
        : result(),
    );
  };
  const director = new NewsDirector({
    apiKey: "fixture-only",
    fetcher,
    clock: () => current,
    feeds: [source, { name: "Broken", url: "https://example.test/broken" }],
  });
  const [one, two] = await Promise.all([
    director.snapshot(),
    director.snapshot(),
  ]);
  assert.equal(feeds, 1);
  assert.equal(classifications, 2);
  assert.deepEqual(one.items, two.items);
  assert.equal(one.sources[1].status, "unavailable");
  assert.equal(one.live.regime, "bull");
  assert.ok(!JSON.stringify(one).includes("fixture-only"));
  current += REFRESH_MS + 1;
  await director.snapshot();
  await director.inflight;
  assert.equal(feeds, 2);
  assert.equal(classifications, 2);
  await director.close();
});

test("Jev sentiment uses relevant classified evidence and confidence gates", async () => {
  const article = parseFeed(xml, source, now)[0];
  const items = [
    {
      ...article,
      classification: {
        status: "classified",
        topic: "models",
        topicConfidence: 0.8,
      },
    },
    {
      ...article,
      id: "irrelevant",
      classification: {
        status: "classified",
        topic: "other",
        topicConfidence: 0.9,
      },
    },
    {
      ...article,
      id: "uncertain",
      classification: {
        status: "classified",
        topic: "finance",
        topicConfidence: 0.2,
      },
    },
  ];
  const evidence = marketEvidence(items, now);
  assert.equal(evidence.length, 1);
  const market = await classifyMarket(evidence, {
    apiKey: "fixture-only",
    fetcher: async (_, options) => {
      const request = JSON.parse(options.body);
      assert.deepEqual(Object.keys(request.questions.regime.criteria), [
        "bull",
        "bear",
        "neutral",
      ]);
      assert.equal(request.state.articles.length, 1);
      return Response.json({
        model: "fixture",
        answers: { regime: answer("bear", 0.5) },
      });
    },
  });
  assert.equal(market.regime, "neutral");
  assert.equal(market.proposedRegime, "bear");
});

test("SQLite shares cold requests and reuses decisions after server restart and feed refresh", async () => {
  const directory = mkdtempSync(join(tmpdir(), "survivor-news-"));
  let current = now,
    feeds = 0,
    calls = 0;
  const create = (version) =>
    new NewsDirector({
      apiKey: "fixture-only",
      version,
      cache: new DecisionCache(join(directory, "news.sqlite"), () => current),
      clock: () => current,
      feeds: [source],
      fetcher: async (_, options) => {
        if (!options.body) {
          feeds++;
          return new Response(xml);
        }
        calls++;
        return Response.json(
          JSON.parse(options.body).questions.regime
            ? { model: "fixture", answers: { regime: answer("bear") } }
            : result(),
        );
      },
    });
  const one = create(),
    two = create();
  try {
    const snapshots = await Promise.all([
      one.snapshot(),
      two.snapshot(),
      one.snapshot(),
      two.snapshot(),
    ]);
    assert.equal(calls, 2);
    assert.equal(feeds, 1);
    assert.ok(
      snapshots.every(
        (snapshot) => snapshot.live.revision === snapshots[0].live.revision,
      ),
    );
  } finally {
    await one.close();
    await two.close();
  }
  const restarted = create();
  try {
    assert.equal((await restarted.snapshot()).cache.snapshot, "hit");
    assert.equal(calls, 2);
    current += REFRESH_MS + 1;
    await restarted.snapshot();
    await restarted.inflight;
    assert.equal(feeds, 2);
    assert.equal(calls, 2);
  } finally {
    await restarted.close();
  }
  const changed = create("new-prompt-version");
  try {
    await changed.snapshot();
    assert.equal(calls, 4);
  } finally {
    await changed.close();
    rmSync(directory, { recursive: true, force: true });
  }
});

test("failed Jev decisions are cached briefly, then retried", async () => {
  let current = now,
    calls = 0;
  const director = new NewsDirector({
    apiKey: "fixture-only",
    feeds: [source],
    clock: () => current,
    fetcher: async (_, options) => {
      if (!options.body) return new Response(xml);
      calls++;
      return new Response("Unavailable", { status: 503 });
    },
  });
  try {
    assert.equal((await director.snapshot()).live.regime, "neutral");
    await director.snapshot();
    assert.equal(calls, 1);
    current += REFRESH_MS + 1;
    await director.snapshot();
    await director.inflight;
    assert.equal(calls, 2);
  } finally {
    await director.close();
  }
});
