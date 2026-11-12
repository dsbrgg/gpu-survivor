# GPU Survivor

**Scale fast. Stay alive.** A satirical pixel-art survival game about the AI race. Play as Jensen, Sam, Dario, or Elon. Collect GPUs, stack upgrades, and defeat the AI Bubble when it arrives at three minutes.

[Watch the 46-second gameplay demo](./gpu-survivor-presentation.mp4)

## Run locally

Requires stable Rust, Node.js 24+, pnpm 11, and a desktop browser with WebGL.

```sh
rustup target add wasm32-unknown-unknown
pnpm install --frozen-lockfile
pnpm build:game
pnpm dev
```

Open **http://127.0.0.1:4173**. The game runs without credentials.

For live news classification, add `TYPESAFE_API_KEY` to a root `.env` file and restart the server. The file is Git-ignored; the key stays server-side.

## Controls

**WASD/arrows** move, **Space** dashes, **E** activates your special, and **Escape/P** pauses. Attacks fire automatically. Choose upgrades with **1–3** or a click. GPUs grant XP; cash restores health.

## Live news

RSS headlines classified by Jev change enemy pressure, XP rewards, and timed effects such as stock crashes and regulatory crackdowns. The timeline shows the publisher, decision, countdown, and gameplay effect.

Headlines and Jev classifications are real. Their gameplay interpretations and boss abilities are fictional satire. Sentiment comes from news, not live stock quotes. Multiplayer and mobile controls are not implemented.

## Development

```sh
pnpm check
pnpm exec playwright install chromium
pnpm test:browser
pnpm build
```

Rebuild Rust changes with `pnpm build:game`; Vite reloads browser code automatically. Production output goes to `dist/`. Static hosting needs a separate `/api/news` backend for live news.

Code: `src/game/` (Rust), `src/web/` (browser), `src/director/` (RSS/Jev).

[Architecture](docs/architecture.md) · [News integration](docs/news-content.md) · [Art and licenses](docs/credits.md)
