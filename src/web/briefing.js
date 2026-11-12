const $ = (selector) => document.querySelector(selector);
const timeLabel = (seconds) =>
  `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;

export class Briefing {
  recipes = new Map();
  open = false;
  resumeOnClose = false;
  compact = matchMedia("(max-width: 900px), (max-height: 520px)");

  constructor({ getState, getEdition, pause, focusGame }) {
    Object.assign(this, { getState, getEdition, pause, focusGame });
    $("#briefing-toggle").addEventListener("click", () => this.show());
    $("#briefing-close").addEventListener("click", () => this.close());
    this.compact.addEventListener("change", () => {
      if (!this.compact.matches) this.close();
    });
    document.querySelectorAll("[data-briefing-tab]").forEach((button) => {
      button.addEventListener("click", () =>
        this.tab(button.dataset.briefingTab),
      );
      button.addEventListener("keydown", (event) => {
        if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key))
          return;
        event.preventDefault();
        const tabs = [...document.querySelectorAll("[data-briefing-tab]")];
        const index =
          event.key === "Home"
            ? 0
            : event.key === "End"
              ? tabs.length - 1
              : (tabs.indexOf(button) +
                  (event.key === "ArrowRight" ? 1 : -1) +
                  tabs.length) %
                tabs.length;
        this.tab(tabs[index].dataset.briefingTab);
        tabs[index].focus();
      });
    });
    document.addEventListener(
      "keydown",
      (event) => {
        if (!this.open) return;
        if (event.key === "Escape") {
          event.preventDefault();
          event.stopImmediatePropagation();
          this.close();
        } else if (event.key === "Tab") {
          const controls = [
            ...$("#world-briefing").querySelectorAll(
              "button:not(:disabled), select:not(:disabled), a[href]",
            ),
          ].filter(
            (element) => element.checkVisibility() && element.tabIndex >= 0,
          );
          const next =
            controls[
              (controls.indexOf(document.activeElement) +
                (event.shiftKey ? -1 : 1) +
                controls.length) %
                controls.length
            ];
          event.preventDefault();
          next?.focus();
        }
      },
      true,
    );
  }

  load(recipes) {
    this.recipes = new Map(recipes.map((recipe) => [recipe.kind, recipe]));
    this.renderEdition();
  }

  tab(name) {
    document.querySelectorAll("[data-briefing-tab]").forEach((button) => {
      const selected = button.dataset.briefingTab === name;
      button.setAttribute("aria-selected", String(selected));
      button.tabIndex = selected ? 0 : -1;
      $(`#briefing-${button.dataset.briefingTab}`).hidden = !selected;
    });
  }

  show() {
    this.open = true;
    this.resumeOnClose = this.getState().phase === "playing";
    if (this.resumeOnClose) this.pause();
    $("#world-briefing").classList.add("briefing-open");
    $("#world-briefing").setAttribute("role", "dialog");
    $("#world-briefing").setAttribute("aria-modal", "true");
    $("#briefing-toggle").setAttribute("aria-expanded", "true");
    $('.briefing-tabs [aria-selected="true"]').focus();
  }

  close() {
    if (!this.open) return;
    this.open = false;
    $("#world-briefing").classList.remove("briefing-open");
    $("#world-briefing").removeAttribute("role");
    $("#world-briefing").removeAttribute("aria-modal");
    $("#briefing-toggle").setAttribute("aria-expanded", "false");
    this.update(this.getState());
    if (this.getState().phase === "menu" && this.compact.matches)
      $("#briefing-toggle").focus();
    else this.focusGame();
  }

  renderEdition() {
    const edition = this.getEdition();
    if (!edition) return;
    $("#edition-title").textContent = edition.title;
    $("#edition-provenance").textContent = edition.provenance;
    $("#edition-kind").textContent = "JEV / SHARED";
    $("#boss-time").textContent =
      `AI BUBBLE / ${timeLabel(edition.duration_seconds)}`;
    $("#frame-label").textContent = edition.title;
    this.update(this.getState());
  }

  update(state) {
    if (!this.open && this.resumeOnClose && state.phase === "paused") {
      this.resumeOnClose = false;
      if (!document.hidden) this.pause();
    }
  }
}
