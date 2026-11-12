# News-driven content

## Implemented foundation

`src/game/content.rs` is the typed recipe catalogue. Recipes compose an asset pack, pickup burst, rival reinforcements, timed pressure, visibility, and temporary player modifiers. The browser receives catalogue descriptions from Rust at startup, so the briefing and simulation use the same definitions.

| Recipe                 | Constructed content                                                                 | Run lifetime |
| ---------------------- | ----------------------------------------------------------------------------------- | ------------ |
| `funding`              | Five cash pickups, five investors                                                   | One burst    |
| `blackout`             | Darkness and unlit racks                                                            | 13 seconds   |
| `liquidation`          | Fifteen GPU pickups                                                                 | One burst    |
| `blue_wave`            | Inspectors, GPU vouchers, Democratic Party banners, increased inspector spawn share | 45 seconds   |
| `research`             | Eight research GPUs, three XP each                                                  | One burst    |
| `compute_expansion`    | Twelve GPUs, two XP each; 10% demand pressure increase                              | 30 seconds   |
| `regulatory_crackdown` | Five lawyers, audit signs, 15% extra spawn pressure                                 | 35 seconds   |
| `ai_stock_crash`       | Damage −20%, fire rate −15%, movement −10%; eight salvage GPUs                      | 30 seconds   |
| `ai_stock_rally`       | Damage +25%, fire rate +20%, movement +10%                                          | 30 seconds   |

Inspectors combine the existing ranged-lawyer behaviour with a new eight-direction uniform. Their GPU vouchers retain the existing XP collection behaviour. Banners are decorative. Each new behaviour needs an authored component; recipes reuse compatible components.

```text
Browser --gpu_start(character), edition JSON--> Edition::parse()
Edition::parse() --validated event kinds--> World::new()
World::trigger_events() --EventKind::recipe()--> typed catalogue
World::trigger_events() --spawn_template(template)--> World::spawn_actor(kind, asset_pack)
Renderer::draw() --actor(row, facing, frame) / pack_sprite(column)--> actors-{0..4}.png / packs.png
```

Parsing rejects unsupported event kinds and arbitrary asset/code fields. Assets are packaged before deployment. Runtime selection has no image-generation dependency.

The Blue Wave fixture remains in `/editions/blue-wave.json` for development tests. Its event starts at 00:12 and ends at 00:57. The normal game starts with neutral conditions and no scheduled news effects.

Blue Wave multiplies spawn pressure by 1.2, capped at 1.35 overall. It introduces four inspectors and replaces 28% of regular spawn selections with inspectors while active. Simultaneous recipes use the strongest pressure multiplier rather than multiplying together. Enemy rewards retain their spawn-time pressure divisor, including after the event expires.

Expiry stops the pressure modifier, additional inspector selection, and banners. Existing enemies and dropped vouchers remain until defeated or collected. Pause freezes run timers. Replay reconstructs the world and its event state.

Stock events show red/green chart signs, a player aura, and exact HUD multipliers. The newest stock event replaces earlier stock modifiers; upgrades remain permanent. Damage scales projectiles, specials, and orbit weapons. Fire rate scales the automatic weapon clock. Movement stacks with cold slowdown and dash speed. Expiry restores normal upgraded stats.

## Two classification axes

News topic describes the story. Content role describes what appears in the arena. One story can select several roles through a single recipe.

| News topic                     | Example recipe direction                    | Content roles                               |
| ------------------------------ | ------------------------------------------- | ------------------------------------------- |
| Politics and policy            | Election wave, export restrictions, hearing | Rival variants, scenery, timed pressure     |
| Finance and earnings           | Funding round, earnings shock               | Cash/GPU abundance, investor reinforcements |
| Models and research            | Open weights, efficiency breakthrough       | Companion/weapon variants, upgrade offers   |
| Chips and infrastructure       | New hardware, outage, capacity expansion    | Pickup variants, hazards, scenery           |
| Competition and company events | Acquisition, liquidation, price war         | Elite rivals, loot caches, encounters       |

The content-role vocabulary is **rival**, **pickup**, **environment**, **encounter**, and **world_modifier**. Jev selects topic and supported recipe. Deterministic code resolves budgets and compatible assets. Affected roster members, significance, and evidence status remain future classification fields. Election themes are authored satire, not a model's prediction that a party causes market difficulty.

## Live feed and Jev

Connected feeds: [OpenAI](https://openai.com/news/rss.xml), [NVIDIA](https://blogs.nvidia.com/feed/), [DeepMind](https://deepmind.google/blog/rss.xml), [Hugging Face](https://huggingface.co/blog/feed.xml), [TechCrunch AI](https://techcrunch.com/category/artificial-intelligence/feed/), and [Yahoo Finance AI stocks](https://finance.yahoo.com/rss/headline?s=NVDA,AMD,MSFT,GOOG). The director parses RSS/Atom, removes markup, normalizes links, and retains publication timestamps. Classification uses supplied excerpts; full articles are not independently verified.

The **Live timeline** connects only when gameplay starts. Dispatches rotate every 12 seconds without scrolling or pagination. Cards show the publisher, Jev topic/confidence, recipe, scheduling decision, and exact effect. The rail uses Rust-confirmed countdowns. Incoming stories affect the current run; imminent and newly activated effects take priority. A compact ribbon repeats the dispatch on small screens. Leaving a run stops timers and aborts its request; delayed responses cannot enter another run.

Hover or focus a card to hold it, or use the headline pause control. Combat pause freezes countdowns and rotation. Incoming updates wait for simulation to resume. Reduced-motion mode removes transitions. There are no manual sentiment controls.

```text
NewsFeed::refresh() --GET /api/news--> NewsDirector::snapshot()
NewsDirector::snapshot() --DecisionCache.get()/remember()--> SQLite snapshot + decision cache
NewsDirector::refresh() --GET fixed feed URLs--> RSS/Atom publishers
classifyArticle() --POST /v1/systemone(state, questions)--> Jev topic + recipe
classifyMarket() --POST /v1/systemone(state.articles, questions.regime)--> Jev sentiment
compileSignals() --revision, sequence, regime, stories--> NewsFeed::refresh()
NewsFeed::refresh() --gpu_live_update() + gpu_live_read()--> LiveUpdate::parse()
LiveState::queue() --increasing sequence; stale updates refused--> pending update
World::update() --LiveState::apply_pending()--> market modifiers + event times
World::trigger_events() --EventKind::recipe()--> authored effects
```

Set `TYPESAFE_API_KEY` in the Git-ignored root `.env`. Development and preview load it server-side. Static hosting requires a backend for `/api/news`; combat works without it.

Jev receives two `choice` questions through `POST https://api.typesafe.ai/v1/systemone`, using `jev-latest`. Responses must match allowed topics/recipes and probability bounds. Both decisions need at least 0.60 confidence. Unknown, uncertain, unrelated, and failed classifications schedule no effect.

Blue Wave requires an explicit Democratic Party election win or party-led AI policy action. A deterministic party-evidence guard rejects unsupported Blue Wave decisions. Generic hearings, export controls, and investigations use Regulatory Crackdown. Stock crashes/rallies require reported share-price movements; predictions, funding rounds, and general sentiment alone do not qualify. Decision version `news-v3` invalidates earlier broad policy classifications.

The aggregate `regime` choice uses relevant classified excerpts with topic confidence at least 0.60. Bull applies 0.9× threat and 1.15× XP; neutral 1×/1×; bear 1.2×/0.9×. Market confidence below 0.60 yields neutral. This is news-derived sentiment, not a live stock-price service. The timeline shows confidence, evidence count, queued/applied status, and effective multipliers.

Feeds refresh every two minutes. The latest 24 articles are deduplicated by URL and normalized title. Three classification workers share persistent SQLite decisions in `.cache/news.sqlite`. WAL, leases, and in-process coalescing prevent duplicate cold work across server processes sharing that file. Cache keys include model and decision version. Successful article/market decisions last 72 hours; unavailable/error decisions last two minutes. Stale snapshots remain available during refresh. Preserve the database on a persistent local volume. Separate hosts require a shared cache service.

Each snapshot selects at most one story per recipe. Rust accepts only increasing sequences, inserts up to three new stories per snapshot and eight per run, and never repeats an accepted story ID. Effects receive eight seconds' warning, at least 20-second spacing, and a 60-second same-recipe cooldown. Nothing new is scheduled at or after the final boss. Already accepted source mappings remain stable when the feed refreshes.

`pnpm test:news` covers parsing, Jev contracts, cross-process coalescing, restart reuse, lease recovery, failures, and expiry. Rust tests cover ordered updates, pause, sentiment, effects, deduplication, and budgets. Browser tests cover lifecycle, late responses, current-run delivery, rotation, layouts, and accessibility using labelled fixture headlines.

## Two expiry clocks

**News lifetime** is 72 hours after the publisher's timestamp. New signals exclude expired stories; accepted run events retain their source mapping. Event-level expiry across rewritten or syndicated stories requires semantic clustering.

**Run lifetime** determines how long a selected recipe affects a match. Blue Wave lasts 45 simulation seconds; stock modifiers last 30 or until replaced by newer stock news. Sentiment persists until the next accepted update, still within the global pressure cap.

Live ingestion and classification run in the development/preview director. Multiplayer room ownership, a production publication worker, semantic clustering, and full-article verification remain future work.
