const $ = (selector) => document.querySelector(selector);
const text = (selector, value) => {
  const element = $(selector);
  if (element.textContent !== value) element.textContent = value;
};
const timeLabel = (seconds) => {
  const value = Math.max(0, Math.floor(seconds));
  return `${String(Math.floor(value / 60)).padStart(2, "0")}:${String(value % 60).padStart(2, "0")}`;
};
const READ_MS = 12_000;
const tones = {
  funding: "mixed",
  blackout: "pressure",
  liquidation: "support",
  blue_wave: "pressure",
  regulatory_crackdown: "pressure",
  ai_stock_crash: "pressure",
  ai_stock_rally: "support",
  research: "support",
  compute_expansion: "mixed",
};

export class NewsTimeline {
  focusId = null;
  residence = 0;
  held = false;
  hovering = false;
  pendingId = null;
  previousTick = performance.now();
  lastTime = -1;
  lastEdition = null;
  lastPhase = null;
  railKey = "";
  cardKey = "";
  toastTimer = null;

  constructor({ briefing, getState, getEdition, getData, getSource }) {
    Object.assign(this, {
      briefing,
      getState,
      getEdition,
      getData,
      getSource,
    });
    $("#news-hold").addEventListener("click", () => {
      this.held = !this.held;
      $("#news-hold").setAttribute("aria-pressed", String(this.held));
      $("#news-hold").setAttribute(
        "aria-label",
        this.held ? "Resume automatic headlines" : "Pause automatic headlines",
      );
      text("#news-hold", this.held ? "▶" : "Ⅱ");
      this.render();
    });
    $("#news-card").addEventListener("pointerenter", () => {
      this.hovering = true;
    });
    $("#news-card").addEventListener("pointerleave", () => {
      this.hovering = false;
    });
    $("#news-toast-close").addEventListener("click", () => {
      $("#news-toast").hidden = true;
    });
  }

  start() {
    this.stop();
    this.focusId = null;
    this.pendingId = null;
    this.lastTime = -1;
    this.held = false;
    this.residence = 0;
    this.previousTick = performance.now();
    $("#news-hold").setAttribute("aria-pressed", "false");
    $("#news-hold").setAttribute("aria-label", "Pause automatic headlines");
    text("#news-hold", "Ⅱ");
    this.interval = setInterval(() => {
      const now = performance.now();
      if (!this.frozen())
        this.residence += Math.min(now - this.previousTick, 1000);
      this.previousTick = now;
      if (!document.hidden) this.render();
    }, 250);
  }

  stop() {
    clearInterval(this.interval);
    clearTimeout(this.toastTimer);
    this.interval = null;
    $("#news-toast").hidden = true;
  }

  frozen() {
    return (
      this.held ||
      this.hovering ||
      document.hidden ||
      $("#news-card").contains(document.activeElement) ||
      this.getState().phase !== "playing"
    );
  }

  entries() {
    const edition = this.getEdition();
    if (!edition) return [];
    const events = [...edition.events, ...(this.getState().live?.events ?? [])];
    return events
      .map((event, index) => {
        const item = this.getSource(event.id);
        return {
          id: `event:${edition.id}:${index}`,
          event,
          item,
          title: item?.title ?? event.title,
          recipe: this.briefing.recipes.get(event.kind),
        };
      })
      .sort((a, b) => a.event.at_seconds - b.event.at_seconds);
  }

  elapsed() {
    const state = this.getState();
    return state.phase !== "menu" && state.edition === this.getEdition()?.id
      ? state.time
      : 0;
  }

  status(entry, elapsed) {
    if (!entry.event) return "information";
    if (elapsed < entry.event.at_seconds) return "upcoming";
    const duration = entry.recipe?.duration_seconds ?? 0;
    if (
      entry.recipe?.player_modifiers &&
      elapsed < entry.event.at_seconds + duration &&
      this.getState().playerEvent !== entry.event.kind
    )
      return "replaced";
    if (duration && elapsed < entry.event.at_seconds + duration)
      return "active";
    return "complete";
  }

  arrival(item) {
    if (item) this.pendingId = item.id;
    this.render();
  }

  decision(entry) {
    const { item, event } = entry;
    const decision = item?.classification;
    const recipe = entry.recipe ?? this.briefing.recipes.get(decision?.recipe);
    if (!item)
      return {
        category: "AUTHORED SAMPLE / NO JEV",
        reason: "Sample edition · scripted effect",
        recipe,
        effect:
          recipe?.description ??
          "Unsupported recipe. This edition cannot start.",
      };
    if (decision.status !== "classified")
      return {
        category: `JEV / ${decision.status.toUpperCase()}`,
        reason: "No classification · no news effect",
        effect: decision.reason ?? "Awaiting Jev classification.",
      };
    const category = `JEV / ${decision.topic.replaceAll("_", " ").toUpperCase()} / ${Math.round(decision.confidence * 100)}%`;
    if (!event && Date.parse(item.expiresAt) <= Date.now())
      return {
        category,
        reason: "72-hour news window expired",
        effect: "Archived headline. No new effect scheduled.",
      };
    if (!recipe)
      return {
        category,
        reason:
          decision.confidence < 0.6
            ? "Below 60% confidence · no effect"
            : (decision.reason ?? "No supported recipe selected"),
        effect: "Informational headline. No gameplay effect.",
      };
    const pending =
      (this.getState().live?.sequence ?? 0) <
      (this.getData()?.live.sequence ?? 0);
    return {
      category,
      recipe,
      effect: recipe.description,
      reason: event
        ? "60% threshold passed · scheduled this run"
        : pending
          ? "Queued · waiting for simulation to resume"
          : this.entries().some((entry) => entry.event.kind === decision.recipe)
            ? "Recipe already represented or cooling down · no extra trigger"
            : "Run budget or final showdown · no extra trigger",
    };
  }

  renderRail(entries, elapsed) {
    const relevant = entries.findIndex(
      (entry) =>
        !["complete", "replaced"].includes(this.status(entry, elapsed)),
    );
    const start = Math.max(
      0,
      (relevant < 0 ? entries.length - 1 : relevant) - 1,
    );
    const visible = entries.slice(start, start + 3);
    const key = visible.map((entry) => entry.id).join("|");
    if (this.railKey !== key) {
      this.railKey = key;
      $("#news-timeline").replaceChildren(
        ...visible.map((entry) => {
          const row = document.createElement("li");
          row.dataset.event = entry.event.kind;
          const time = document.createElement("time");
          time.textContent = timeLabel(entry.event.at_seconds);
          const label = document.createElement("span");
          label.textContent = entry.recipe?.label ?? entry.event.kind;
          const status = document.createElement("small");
          row.append(time, label, status);
          return row;
        }),
      );
    }
    const rows = $("#news-timeline").children;
    visible.forEach((entry, index) => {
      const status = this.status(entry, elapsed);
      rows[index].dataset.state = status;
      rows[index].dataset.current = String(entry.id === this.focusId);
      const label =
        status === "upcoming"
          ? `IN ${Math.ceil(entry.event.at_seconds - elapsed)}s`
          : status === "active"
            ? `${Math.ceil(entry.event.at_seconds + entry.recipe.duration_seconds - elapsed)}s LEFT`
            : status === "replaced"
              ? "REPLACED"
              : "DONE";
      if (rows[index].lastElementChild.textContent !== label)
        rows[index].lastElementChild.textContent = label;
    });
    const active = entries.filter(
      (entry) => this.status(entry, elapsed) === "active",
    );
    const upcoming = entries.find(
      (entry) => this.status(entry, elapsed) === "upcoming",
    );
    const status = active.length
      ? active
          .map(
            (entry) =>
              `${entry.recipe.label} · ${Math.ceil(entry.event.at_seconds + entry.recipe.duration_seconds - elapsed)}s`,
          )
          .join(" / ")
      : upcoming
        ? `${upcoming.recipe?.label ?? "NEXT EFFECT"} IN ${Math.ceil(upcoming.event.at_seconds - elapsed)}s`
        : (this.getState().boss?.label ??
          `AI BUBBLE AT ${timeLabel(this.getEdition().duration_seconds)}`);
    text("#timeline-now", status);
    text("#briefing-pulse", status);
    text("#timeline-clock", timeLabel(elapsed));
    const edition = this.getEdition();
    const state = this.getState();
    $("#timeline-regime").dataset.market = state.marketRegime ?? "neutral";
    text(
      "#timeline-regime",
      `${(state.marketRegime ?? "neutral").toUpperCase()} / JEV`,
    );
    const market = this.getData()?.market;
    const pending =
      (state.live?.sequence ?? 0) < (this.getData()?.live.sequence ?? 0);
    text(
      "#market-decision",
      market
        ? `${pending ? "QUEUED" : "APPLIED"} · ${Math.round(market.confidence * 100)}% · ${market.evidence.length} excerpts · ${market.reason}`
        : "Jev is assessing recent market/news evidence.",
    );
    const threat =
      state.edition === edition.id && state.phase !== "menu"
        ? state.effectiveThreat
        : edition.threat_multiplier;
    text(
      "#timeline-modifiers",
      `${(threat ?? 1).toFixed(2)}× THREAT · ${(state.xpMultiplier ?? 1).toFixed(2)}× XP`,
    );
  }

  render() {
    if (!this.getEdition() || !this.briefing.recipes.size) return;
    if (!this.interval) {
      $("#news-card").hidden = true;
      $("#news-timeline").replaceChildren();
      this.railKey = "";
      text("#timeline-regime", "NEUTRAL / WAITING");
      text("#timeline-modifiers", "1.00× THREAT · 1.00× XP");
      text(
        "#market-decision",
        "Jev assesses market sentiment after you start.",
      );
      text("#timeline-now", "Start a run to connect the timeline.");
      text("#timeline-clock", "00:00");
      text("#news-following", "Start a run to connect publisher feeds.");
      text("#compact-decision", "LIVE TIMELINE / STANDBY");
      text("#compact-headline", "News connects when you play.");
      text(
        "#compact-effect",
        "Jev decisions update your current run automatically.",
      );
      text("#briefing-pulse", "START A RUN TO CONNECT");
      return;
    }
    const entries = this.entries();
    const elapsed = this.elapsed();
    const included = new Set(entries.map((entry) => entry.item?.id));
    const stories = (this.getData()?.items ?? [])
      .filter((item) => !included.has(item.id))
      .map((item) => ({ id: `story:${item.id}`, item, title: item.title }));
    const deck = [...entries, ...stories];
    const frozen = this.frozen();
    const running = ["playing", "paused", "upgrade"].includes(
      this.getState().phase,
    );
    // Give impending and newly triggered effects a guaranteed reading window.
    const priority =
      running &&
      entries
        .filter(
          (entry) =>
            elapsed >= entry.event.at_seconds - 5 &&
            elapsed < entry.event.at_seconds + 8,
        )
        .at(-1);
    let focus = deck.find((entry) => entry.id === this.focusId);
    if (!frozen && priority) focus = priority;
    else if (!frozen && this.pendingId) {
      focus = deck.find((entry) => entry.item?.id === this.pendingId) ?? focus;
      this.pendingId = null;
      this.residence = 0;
    } else if (!focus || (!frozen && this.residence >= READ_MS)) {
      focus =
        deck[
          (deck.findIndex((entry) => entry.id === this.focusId) + 1) %
            deck.length
        ];
      this.residence = 0;
    }
    if (focus?.id !== this.focusId) {
      this.focusId = focus?.id;
      this.residence = 0;
    }
    this.renderRail(entries, elapsed);
    $("#news-card").hidden = !focus;
    if (!focus) {
      text("#compact-decision", "NEWS / WAITING");
      text("#compact-headline", "No scheduled news effects.");
      text(
        "#compact-effect",
        "Regular rival waves continue while Jev assesses the news.",
      );
      text("#news-following", "Watching publisher feeds for the next story.");
      return;
    }
    const decision = this.decision(focus);
    const status = this.status(focus, elapsed);
    const recent =
      status !== "replaced" &&
      focus.event &&
      elapsed >= focus.event.at_seconds &&
      elapsed < focus.event.at_seconds + 8;
    const application = focus.event
      ? status === "upcoming"
        ? `${timeLabel(focus.event.at_seconds)} / IN ${Math.ceil(focus.event.at_seconds - elapsed)}s`
        : status === "active"
          ? `ACTIVE / ${Math.ceil(focus.event.at_seconds + focus.recipe.duration_seconds - elapsed)}s LEFT`
          : status === "replaced"
            ? "REPLACED / NEW STOCK NEWS"
            : recent
              ? "JUST APPLIED / ONE BURST"
              : "COMPLETE / THIS RUN"
      : decision.recipe &&
          (this.getState().live?.sequence ?? 0) <
            (this.getData()?.live.sequence ?? 0)
        ? "QUEUED / THIS RUN"
        : "INFORMATION ONLY";
    const cardState = recent ? "active" : status;
    const key = `${focus.id}:${cardState}`;
    const card = $("#news-card");
    if (key !== this.cardKey) {
      this.cardKey = key;
      card.dataset.story = focus.item?.id ?? focus.id;
      card.dataset.state = cardState;
      card.dataset.tone = tones[decision.recipe?.kind] ?? "neutral";
      if (!matchMedia("(prefers-reduced-motion: reduce)").matches) {
        card.getAnimations().forEach((animation) => animation.cancel());
        card.animate(
          [
            { opacity: 0.45, transform: "translateY(7px)" },
            { opacity: 1, transform: "translateY(0)" },
          ],
          { duration: 280, easing: "ease-out" },
        );
      }
    }
    text("#news-application", application);
    text(
      "#news-signal",
      (tones[decision.recipe?.kind] ?? "feed").toUpperCase(),
    );
    text(
      "#news-source",
      focus.item
        ? `${focus.item.source} · ${new Date(focus.item.publishedAt).toLocaleDateString([], { month: "short", day: "numeric" })}`
        : "SAMPLE DISPATCH / FICTIONAL",
    );
    text("#news-title", focus.title);
    $("#news-title").title = focus.title;
    if (focus.item && /^https?:\/\//.test(focus.item.url))
      $("#news-title").href = focus.item.url;
    else $("#news-title").removeAttribute("href");
    text("#news-category", decision.category);
    text("#news-recipe", `→ ${decision.recipe?.label ?? "NO GAMEPLAY EFFECT"}`);
    const icon = {
      blue_wave: "democrats",
      regulatory_crackdown: "regulatory_crackdown",
      ai_stock_crash: "ai_stock_crash",
      ai_stock_rally: "ai_stock_rally",
    }[decision.recipe?.kind];
    $("#news-icon").hidden = !icon;
    if (icon) $("#news-icon").src = `/assets/${icon}.png`;
    text("#news-reason", decision.reason);
    text("#news-effect", decision.effect);
    text("#compact-decision", `${application} · ${decision.category}`);
    text("#compact-headline", focus.title);
    text("#compact-effect", decision.effect);
    const index = deck.indexOf(focus);
    text("#news-page", `${index + 1} / ${deck.length} DISPATCHES`);
    text(
      "#news-cadence",
      frozen
        ? "HELD"
        : priority
          ? "EVENT FOCUS"
          : `NEXT IN ${Math.max(0, Math.ceil((READ_MS - this.residence) / 1000))}s`,
    );
    $("#news-progress-fill").style.transform =
      `scaleX(${Math.min(1, this.residence / READ_MS)})`;
    const following =
      deck.find((entry) => entry.item?.id === this.pendingId) ??
      deck[(index + 1) % deck.length];
    text(
      "#news-following",
      deck.length > 1
        ? following.title
        : "Watching publisher feeds for the next story.",
    );
  }

  update(state) {
    const entries = this.entries();
    if (
      (state.phase === "menu" && this.lastPhase !== "menu") ||
      state.time < this.lastTime ||
      state.edition !== this.lastEdition
    ) {
      this.lastTime = -1;
      this.focusId = null;
      this.residence = 0;
      clearTimeout(this.toastTimer);
      $("#news-toast").hidden = true;
    }
    this.lastEdition = state.edition;
    this.lastPhase = state.phase;
    if (state.phase === "playing") {
      for (const entry of entries) {
        if (
          entry.event.at_seconds > this.lastTime &&
          entry.event.at_seconds <= state.time
        ) {
          clearTimeout(this.toastTimer);
          text(
            "#news-toast-label",
            entry.item
              ? "NEWS EFFECT / ACTIVE NOW"
              : "SAMPLE EFFECT / ACTIVE NOW",
          );
          text("#news-toast-title", entry.title);
          text("#news-toast-effect", this.decision(entry).effect);
          $("#news-toast").hidden = false;
          this.toastTimer = setTimeout(() => {
            $("#news-toast").hidden = true;
          }, 8000);
        }
      }
      this.lastTime = state.time;
    }
    this.render();
  }
}
