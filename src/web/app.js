import { Audio } from "./audio.js";
import { Briefing } from "./briefing.js";
import { NewsFeed } from "./news.js";

const $ = (selector) => document.querySelector(selector);
const canvas = $("#glcanvas");
const canvasResize = new ResizeObserver(() => {
  if (ready) window.resize(canvas, window.wasm_exports.resize);
});
canvasResize.observe(canvas);
const audio = new Audio();
const encoder = new TextEncoder();
const decoder = new TextDecoder();
let edition;
let selectedCharacter = 0;
let ready = false;
let state = { phase: "loading" };
let previousPhase;
let choiceKey = "";
let encodedEdition = new Uint8Array();
let encodedLive = new Uint8Array();
const briefing = new Briefing({
  getState: () => state,
  getEdition: () => edition,
  pause,
  focusGame,
});
const news = new NewsFeed({
  briefing,
  getState: () => state,
  getEdition: () => edition,
  deliver: (update) => {
    encodedLive = encoder.encode(JSON.stringify(update));
    window.wasm_exports.gpu_live_update();
  },
});

function fail(message) {
  $("#error-message").textContent = message;
  $("#error-overlay").hidden = false;
  $("#connection-label").textContent = "LOAD FAILED";
}

function focusGame() {
  canvas.focus({ preventScroll: true });
}

function start(character = selectedCharacter) {
  if (!ready) return;
  news.stop();
  briefing.tab("news");
  selectedCharacter = character;
  if (edition.id === "live-news-v1")
    edition.seed = crypto.getRandomValues(new Uint32Array(1))[0];
  encodedEdition = encoder.encode(JSON.stringify(edition));
  window.wasm_exports.gpu_start(character);
  focusGame();
}

function choose(index) {
  if (state.phase !== "upgrade") return;
  window.wasm_exports.gpu_upgrade(index);
  focusGame();
}

function pause() {
  if (!ready) return;
  window.wasm_exports.gpu_pause();
  focusGame();
}

function menu() {
  window.wasm_exports.gpu_menu();
  focusGame();
}

function setEdition(next) {
  if (!["loading", "menu", "won", "lost"].includes(state.phase)) {
    throw new Error(
      "A running game pins its edition. Return to the lobby first.",
    );
  }
  edition = structuredClone(next);
  encodedEdition = encoder.encode(JSON.stringify(edition));
  briefing.renderEdition();
}

function updateUi(next) {
  state = next;
  window.dispatchEvent(new CustomEvent("gpu:state", { detail: next }));
  for (const phase of ["menu", "upgrade", "paused", "result"]) {
    const id = phase === "paused" ? "pause" : phase;
    $(`#${id}-overlay`).hidden =
      phase === "result"
        ? !["won", "lost"].includes(next.phase)
        : next.phase !== phase;
  }
  $("#pause-button").disabled = !["playing", "paused"].includes(next.phase);
  briefing.update(next);
  news.update(next);
  const labels = {
    menu: "AWAITING FOUNDER",
    playing: "RUN IN PROGRESS",
    paused: "BOARD MEETING",
    upgrade: "NEW RESEARCH AVAILABLE",
    won: "BUBBLE BURST",
    lost: "RUNWAY DEPLETED",
  };
  $("#game-status").textContent = labels[next.phase] ?? next.phase;
  $("#connection-label").textContent = "COMPUTE ONLINE";
  $("#game-engine").textContent =
    next.phase === "menu"
      ? "RUST / WASM / DETAILED PIXEL ART"
      : `${next.enemyCount} RIVALS · ${next.fps} FPS · ${next.edition}`;

  if (next.phase === "upgrade") {
    const key = `${next.level}:${next.choices.map((c) => c.kind).join(",")}`;
    if (choiceKey !== key || previousPhase !== "upgrade") {
      choiceKey = key;
      $("#upgrade-level").textContent =
        `LEVEL ${String(next.level).padStart(2, "0")} · PICK ONE UPGRADE`;
      $("#upgrade-cards").replaceChildren(
        ...next.choices.map((card, index) => {
          const button = document.createElement("button");
          button.className = "upgrade-card";
          button.dataset.upgrade = index;
          button.dataset.kind = card.kind;
          const number = document.createElement("span");
          number.className = "upgrade-number";
          number.textContent = `0${index + 1}`;
          const name = document.createElement("strong");
          name.textContent = card.name;
          const description = document.createElement("p");
          description.textContent = card.description;
          const tag = document.createElement("small");
          tag.textContent = `${card.tag} / RANK ${card.level}`;
          button.append(number, name, description, tag);
          button.addEventListener("click", () => choose(index));
          return button;
        }),
      );
    }
  }
  if (["won", "lost"].includes(next.phase) && previousPhase !== next.phase) {
    const won = next.phase === "won";
    $("#result-eyebrow").textContent = won
      ? "THE BUBBLE HAS BURST"
      : "THE QUARTERLY REPORT";
    $("#result-title").textContent = won
      ? "TOO BIG TO FAIL."
      : next.hp > 0
        ? "YOUR TIME IS UP."
        : "YOUR RUNWAY RAN OUT.";
    $("#result-description").textContent = won
      ? "You burst the AI Bubble. AGI is still six months away."
      : next.hp > 0
        ? "The final showdown ends only when you or the AI Bubble falls."
        : "Incredible vision. Unprecedented scale. Absolutely no runway.";
    $("#result-time").textContent = formatTime(next.time);
    $("#result-gpus").textContent = next.gpus;
    $("#result-kills").textContent = next.kills;
  }
  previousPhase = next.phase;
}

function formatTime(seconds) {
  return `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
}

window.gpu = Object.freeze({
  getState: () => structuredClone(state),
  getEdition: () => structuredClone(edition),
  setEdition,
  start,
  pause,
  choose,
  menu,
  isReady: () => ready,
});

document.querySelectorAll("[data-character]").forEach((button) => {
  button.addEventListener("click", () => {
    selectedCharacter = Number(button.dataset.character);
    document.querySelectorAll("[data-character]").forEach((card) => {
      const selected = Number(card.dataset.character) === selectedCharacter;
      card.classList.toggle("selected", selected);
      card.setAttribute("aria-pressed", String(selected));
    });
  });
});

$("#start-button").addEventListener("click", () => start());
$("#retry-button").addEventListener("click", () => start());
$("#pause-button").addEventListener("click", pause);
$("#resume-button").addEventListener("click", pause);
$("#reload-button").addEventListener("click", () => location.reload());
document
  .querySelectorAll(".return-menu")
  .forEach((button) => button.addEventListener("click", menu));
$("#sound-toggle").addEventListener("click", async () => {
  try {
    const enabled = await audio.toggle();
    $("#sound-toggle").textContent = enabled ? "SOUND ON" : "SOUND OFF";
    $("#sound-toggle").setAttribute("aria-pressed", String(enabled));
    focusGame();
  } catch {
    $("#sound-toggle").textContent = "AUDIO UNAVAILABLE";
  }
});
$("#fullscreen").addEventListener("click", async () => {
  try {
    if (document.fullscreenElement) await document.exitFullscreen();
    else await $("#game-viewport").requestFullscreen();
    focusGame();
  } catch {
    $("#fullscreen").title = "Fullscreen is unavailable in this browser.";
  }
});
document.addEventListener("visibilitychange", () => {
  if (document.hidden && state.phase === "playing") pause();
});
canvas.addEventListener("keydown", (event) => {
  if (
    ["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(
      event.code,
    )
  )
    event.preventDefault();
});

window.addEventListener("error", (event) =>
  fail(event.message || "The game encountered an error."),
);
window.addEventListener("unhandledrejection", (event) =>
  fail(String(event.reason?.message ?? event.reason)),
);

async function boot() {
  const editions = await Promise.all(
    ["live"].map(async (name) => {
      const response = await fetch(`/editions/${name}.json`);
      if (!response.ok)
        throw new Error(`Edition could not load (${response.status}).`);
      return response.json();
    }),
  );
  setEdition(editions[0]);
  window.miniquad_add_plugin({
    name: "gpu_bridge",
    version: 1,
    register_plugin(imports) {
      imports.env.gpu_config_len = () => encodedEdition.length;
      imports.env.gpu_live_len = () => encodedLive.length;
      imports.env.gpu_live_read = (pointer, length) => {
        new Uint8Array(window.wasm_memory.buffer, pointer, length).set(
          encodedLive.subarray(0, length),
        );
      };
      imports.env.gpu_config_read = (pointer, length) => {
        new Uint8Array(window.wasm_memory.buffer, pointer, length).set(
          encodedEdition.subarray(0, length),
        );
      };
      imports.env.gpu_event = (pointer, length) => {
        const message = JSON.parse(
          decoder.decode(
            new Uint8Array(window.wasm_memory.buffer, pointer, length),
          ),
        );
        if (message.type === "ready") {
          ready = true;
          briefing.load(message.recipes);
          requestAnimationFrame(() =>
            window.resize(canvas, window.wasm_exports.resize),
          );
          $("#start-button").disabled = false;
          $("#start-button span").textContent = "ENTER THE MARKET";
        } else if (message.type === "state") updateUi(message);
        else if (message.type === "sounds")
          message.cues.forEach((cue) => audio.play(cue));
        else if (message.type === "error") fail(message.message);
      };
    },
  });
  window.load("/game.wasm");
}

boot().catch((error) => fail(error.message));
