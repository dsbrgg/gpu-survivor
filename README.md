# GPU Survivor

**Scale fast. Stay alive.** A browser-first, pixel-art survival game. Collect GPUs, upgrade your founder, and burst the AI Bubble.

## How to run

Install stable Rust, Node.js 24+ and pnpm 11.

```sh
rustup target add wasm32-unknown-unknown
pnpm install
pnpm build:game
pnpm dev
```

Open **http://127.0.0.1:4173**. A keyboard and WebGL-capable desktop browser are required. Press **Sound off** to enable synthesized sound effects.

Run `pnpm build:game` after Rust changes. Vite reloads browser code automatically. `pnpm build` produces a static site in `dist/`; deploy that directory to a static host with WebAssembly MIME support. `pnpm preview` serves the production build locally.

For live news classification, set `TYPESAFE_API_KEY` in the root `.env`. Development and preview load it privately. Shared SQLite decisions persist in `.cache/news.sqlite`; both paths are Git-ignored. Static hosting needs a separate `/api/news` backend. Without it, combat still works.

## Controls

WASD/arrows move. Space dashes. E activates your founder's special. Escape/P pauses. Attacks aim and fire automatically. Pick upgrades with 1–3 or a click. Cash restores health; GPU drops grant XP.

## What works

- Four founders fire NVIDIA/OpenAI/Claude/Grok logos. Level-up choices add attack radius and stacking Leather Jacket, Capital Injection, Safety Rails, or Cybertruck weapons. E activates each founder's special.
- Infinite data-center halls contain shaded racks, cooling cabinets, and industrial flooring. Cold slows movement 38%; failed cooling ramps heat damage until you leave.
- Random Bernie, AOC, and Alice Huang — (Alleged) GPU Espionage minibosses appear after 00:40. At 03:00, the AI Bubble joins surviving rivals. Its escalating final showdown continues until it or you dies.
- Eight-direction enemies throw contextual attacks: legislation, subpoenas, citations, Chinese flags, wrenches, coins, electrical bolts, and scrap. Viewport-fit UI, synthesized sound, pause, and restart work.
- The **Live timeline** connects when play starts. Headlines rotate automatically; Jev sentiment changes threat/XP immediately. New effects get eight seconds' warning. Server-side caching shares decisions across players and restarts. Small screens show a live ribbon.

Nine news recipes include Democratic-logo Blue Wave, Regulatory Crackdown, and 30-second AI Stock Crash/Rally stat changes. Rally boosts damage 25%, fire rate 20%, and movement 10%; crash reduces them 20%, 15%, and 10%. New stock effects replace older ones. Repeated recipes have a 60-second cooldown.

## What's faked or mocked

Live RSS headlines and Jev decisions are real; gameplay interpretations are authored satire. Sentiment is news-derived, not a live stock quote. Boss abilities and visuals are fictional. Multiplayer, full-article verification, online leaderboards, persistent unlocks, and mobile controls are not implemented.

## Development

```sh
pnpm check
pnpm exec playwright install chromium
pnpm test:browser
pnpm assets
pnpm assets:check
```

`src/game/` holds Rust; `src/web/` holds the browser shell; `src/director/` handles RSS/Jev. Asset rebuilding requires Python 3.11+ and [uv](https://docs.astral.sh/uv/), with no image-service calls. See [art pipeline](docs/art.md). Builds copy the locked Miniquad browser loader.

Architecture and multiplayer: [docs/architecture.md](docs/architecture.md). News integration: [docs/news-content.md](docs/news-content.md). Roster and researched boss concepts: [docs/roster.md](docs/roster.md).

Third-party licenses: [docs/credits.md](docs/credits.md).
