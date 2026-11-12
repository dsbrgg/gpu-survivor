# Architecture

## Runtime

The browser is the primary deployment target. Rust compiles to WebAssembly with Macroquad's WebGL loader. Vite builds the HTML/CSS/JavaScript shell. The game is static; live news additionally requires the server-side `/api/news` endpoint included in development and preview.

`world.rs` owns the fixed 60 Hz simulation. `Input` is a tick's movement and action intent. Simulation updates do not invoke rendering, HTTP, browser APIs, or wall-clock time. A seeded local RNG controls spawns, drops, and upgrade choices. Floating-point simulation is not a promise of cross-platform lockstep determinism.

`upgrade.rs` owns upgrade cards and rank limits; `founder.rs` selects each character's signature upgrade. Signature weapons use a separate automatic cooldown alongside starter fire. Friendly projectiles share a 96-shot budget. Stable enemy IDs prevent repeated piercing hits after enemy removal. Swept collisions query grid cells using actual hit radii; cash explosions and Safety Rails reuse the same shot pipeline. Radius upgrades scale simulation geometry and rendered sizes together.

`render.rs` draws a 640×360 logical view into a 1280×720 render target using nearest-neighbour presentation. This preserves 128px actor detail without changing world coordinates. The render target uses zero samples to avoid a WebGL2 resolve pass on the WebGL1 renderer. Builds copy the loader from Cargo's exact Miniquad dependency. A depth-sorted sprite pass handles actors and scenery. Spatial buckets limit enemy separation and projectile collision checks. Entity budgets bound particles, enemies, and loot. `main.rs` advances fixed ticks, handles local input, and publishes snapshots to the browser at 5 Hz or phase changes.

The browser shell occupies `100dvh`. The arena fills the remaining grid space, and Rust letterboxes its fixed-resolution view. Container queries adapt lobby and upgrade cards to the arena dimensions. A `ResizeObserver` keeps the WebGL drawing buffer synchronized with CSS layout and fullscreen changes.

`actors-0.png` through `actors-4.png` hold 17 actors with four frames for each of eight directions. IDs follow Jensen, Sam, lawyer, investor, bot, crawler, Bernie, inspector, Dario, Elon, engineer, politician, GLM, Kimi, DeepSeek, AOC, and Alice Huang. Each 2048×1024 page holds four actors in 128px cells. `bubble.png` is the floating final boss. `founder.rs` maps starters to actor IDs and weapons. `logos.png`, `packs.png`, and `sprites.png` hold projectile marks, themed content, and props.

Players face movement input; enemies face actual displacement, including retreat. Walk clocks advance only while moving. Idle actors retain their facing, including for dashes.

`terrain.rs` streams a 5×5 neighborhood of 384-unit chunks. Coordinate hashing uses the edition seed independently of combat randomness. Each chunk contains 24 racks/cooling/power units and a seeded thermal zone. Cross aisles connect at chunk seams. The starting hall is safe; west introduces cold and east introduces failed cooling. Other climates and equipment variants come from the seed.

Cold applies a 0.62 movement multiplier, including dashes. Heat allows 0.75 seconds of grace, then deals `min(22, 2 + 1.4 × (exposure - 0.75))` HP/second. Leaving stops damage immediately and sheds three exposure seconds per second. Re-entry retains residual exposure. Combat shields do not stop heat; pause and upgrade screens freeze it.

The camera follows without arena bounds. Distant ordinary rivals and loot are evicted at 900/1100 units; eviction grants no rewards. Bosses relocate near the player beyond 800 units, preserving health. Chunk eviction bounds terrain memory; revisiting reconstructs terrain, not discarded loot.

The briefing contains Live Timeline and Sources. `news.js` starts fetching only after Rust enters play, aborts on leaving a run, and retains accepted source identities. `timeline.js` rotates dispatches every 12 seconds and renders Rust-confirmed event times. A compact ribbon shows the same dispatch. Opening the full compact briefing traps focus and pauses combat until closed.

Impending effects take focus five seconds before activation and remain featured for eight seconds afterward. Hover, keyboard focus, headline hold, paused combat, and hidden documents suspend rotation. Reduced-motion preferences disable transitions. New stories affect this run; accepted source mappings remain stable across feed refreshes.

```text
Browser --GET /editions/live.json--> static host
Browser --gpu_config_read(pointer, length)--> Rust edition buffer
Browser --gpu_start(character)--> simulation
Browser --gpu_pause() / gpu_upgrade(index) / gpu_menu()--> command inbox
Simulation --gpu_event(JSON bytes)--> browser HUD / overlays / audio
Simulation --Renderer::draw(world)--> WebGL canvas
```

Rust reads the edition through host imports when starting a run. Invalid versions, unsupported event kinds, out-of-range modifiers, and invalid timing are rejected before replacing the world. The synchronous bridge copies JSON bytes; no Rust memory pointer is retained by JavaScript. Web UI text uses DOM text nodes rather than generated HTML.

`window.gpu` exposes `start`, `pause`, `choose`, `menu`, `getState`, `getEdition`, and `setEdition`. `gpu:state` DOM events expose read-only-by-copy telemetry. `setEdition` refuses updates during active, paused, or upgrade phases.

## Edition contract

The runnable example is `src/web/public/editions/demo.json`. Version 1 includes ID, seed, duration, boss arrival, regime, threat and XP multipliers, provenance, and a bounded list of timed events.

The nine supported recipes are `funding`, `blackout`, `liquidation`, `blue_wave`, `research`, `compute_expansion`, `regulatory_crackdown`, `ai_stock_crash`, and `ai_stock_rally`. `content.rs` defines assets, pickup bursts, reinforcements, durations, pressure, and player modifiers. Threat is constrained to 0.75–1.35 and XP to 0.85–1.25. Timed pressure uses the strongest active recipe and remains capped. Enemies retain spawn-time reward pressure. Edition data cannot introduce executable code.

`World::player_event()` selects the newest active stock event. Its damage, automatic fire-rate, and movement multipliers apply without mutating permanent upgrades. Event activation precedes movement/combat each tick. HUD telemetry exposes `playerEvent` and `playerModifiers`; the timeline marks superseded stock effects as replaced.

The modular pack implementation, news/content taxonomy, and proposed publication lifetimes are described in [news-content.md](news-content.md).

Public-market returns, company news momentum, and individual news events are separate future inputs. Private companies receive labelled news-momentum scores, not fabricated prices. Daily bars suffice for daily editions. The latest trading session should retain its timestamp across market closures.

The target XP budget is divided by spawn pressure when assigning per-enemy rewards. This avoids accidentally multiplying rewards twice when harsher conditions produce more enemies. These are balancing approximations, not an economic model.

## News director

`src/director/news.mjs` fetches RSS/Atom feeds, validates Jev decisions, and compiles bounded live signals. `plugin.mjs` serves `GET /api/news` in Vite development and preview. It loads `TYPESAFE_API_KEY` from the Git-ignored root `.env` into the server process. Credentials never enter browser bundles.

```text
NewsFeed::refresh() --GET /api/news--> NewsDirector::snapshot()
NewsDirector::snapshot() --DecisionCache.get()/remember()--> .cache/news.sqlite
NewsDirector::refresh() --GET fixed RSS URLs--> publisher feeds
classifyArticle() --POST /v1/systemone--> Jev typed topic + recipe choices
classifyMarket() --POST /v1/systemone--> Jev aggregate regime choice
NewsFeed::refresh() --gpu_live_update(), gpu_live_read()--> LiveUpdate::parse()
LiveState::queue() --increasing sequence only--> pending simulation update
World::update() --LiveState::apply_pending()--> regime + bounded event schedule
World::trigger_events() --EventKind::recipe()--> gameplay effect
NewsTimeline::update(state) --confirmed schedule + simulation time--> timeline + effect popup
```

Snapshots refresh every two minutes. SQLite WAL, per-key leases, and in-process coalescing share cold work across processes using the same database file. Article and aggregate decisions are keyed by content, model, prompt version, and credential availability. Successful decisions last 72 hours; failures retry after two minutes. Stale snapshots remain available during refresh. Separate hosts need a shared cache service; independent database files do not coordinate.

The latest 24 articles are deduplicated by title and URL. Jev assesses relevant, unexpired excerpts for bull/neutral/bear sentiment. Confidence below 0.60 yields neutral. Threat/XP multipliers are 0.9/1.15, 1/1, and 1.2/0.9 respectively. Blue Wave requires explicit Democratic Party evidence; other policy stories use Regulatory Crackdown. Stock events require reported price movements. The `news-v3` decision version invalidates older cached classifications.

Signals select one story per recipe per snapshot. Rust accepts increasing sequences and queues paused updates. New events receive eight seconds' warning, 20-second spacing, a 60-second same-recipe cooldown, at most three additions per snapshot and eight per run. Repeated story IDs never retrigger. Events stop being scheduled at the finale.

## Online multiplayer direction

Server-authoritative co-op rooms are the target. The current game is single-player with a shared news backend; gameplay has no network transport.

Extract simulation into a headless Rust library when adding the server. Replace the single player fields with player-ID-keyed state. Rendering remains browser-side. The server owns enemy AI, RNG, drops, damage, upgrades, and the edition for the room.

```text
Browser --POST /rooms--> room service
Browser --WebSocket Input(sequence, tick, movement, actions)--> Rust room server
Room server --fixed tick World::update(inputs)--> authoritative world
Room server --Snapshot(tick, acknowledged_input, entity_states)--> browsers
Browser --interpolate(remote_entities) / reconcile(local_player)--> renderer
```

Use reliable WebSockets initially, 60 Hz server simulation, and 10–20 Hz network snapshots. Predict local movement; reconcile against authoritative positions. Interpolate other players/enemies. The server refuses client-supplied positions, damage, inventory, and scores. Add entity IDs, server-side rate limits, and reconnect snapshots with room ownership.

Edition randomness is server-owned; clients do not need identical RNG consumption. Keep purely cosmetic particles and audio client-side. Pause and upgrade menus are currently single-player semantics: co-op upgrade selection must be per-player and must not pause the room. Disconnect policy, shared versus individual loot, and revive mechanics need a gameplay decision before implementation.

## Current design

Runway is health. GPUs are XP. Cash heals. Auto-fire targets the nearest rival; E is a founder-specific area ability. Space grants a brief invulnerable dash. Time increases spawn pressure. Each live start chooses a random seed. Seeded miniboss order and first arrival (44–58 seconds) are reproducible. Subsequent encounters wait 18–30 seconds after a miniboss dies; only one is active. No new miniboss starts in the last 15 seconds.

At the edition duration (normally 03:00), the AI Bubble joins the surviving enemies and clears hostile projectiles. Existing rivals and minibosses retain their state and keep fighting; ambient spawns stop. `World::active_boss()` prioritizes the Bubble for the native HUD and browser telemetry. It alternates rotating coin rings and aimed fans, accelerating as health falls. Combat continues without a timeout; only Bubble death wins, and simultaneous player death loses. See [roster.md](roster.md) and [art.md](art.md).
