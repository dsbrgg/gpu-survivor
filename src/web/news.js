import { NewsTimeline } from "./timeline.js";

const $ = (selector) => document.querySelector(selector);
const running = (phase) => ["playing", "paused", "upgrade"].includes(phase);

export class NewsFeed {
  data = null;
  active = false;
  sources = new Map();
  sequence = 0;
  generation = 0;
  controller = null;
  timer = null;

  constructor({ briefing, getState, getEdition, deliver }) {
    Object.assign(this, { briefing, getState, getEdition, deliver });
    this.timeline = new NewsTimeline({
      briefing,
      getState,
      getEdition,
      getData: () => this.data,
      getSource: (id) => this.sources.get(id),
    });
    $("#news-refresh").addEventListener("click", () => this.refresh());
    document.addEventListener("visibilitychange", () => {
      if (!document.hidden && this.active) void this.refresh();
    });
    window.addEventListener("pagehide", () => this.stop());
  }

  stop() {
    this.active = false;
    this.generation++;
    clearTimeout(this.timer);
    this.controller?.abort();
    this.controller = null;
    this.timeline.stop();
  }

  begin() {
    this.active = true;
    this.data = null;
    this.sequence = 0;
    this.sources.clear();
    this.timeline.start();
    void this.refresh();
  }

  async refresh() {
    if (!this.active || this.controller || document.hidden) return;
    clearTimeout(this.timer);
    const generation = this.generation;
    this.controller = new AbortController();
    $("#news-refresh").disabled = true;
    $("#news-status").textContent = "Loading shared news + Jev decisions…";
    try {
      const response = await fetch("/api/news", {
        signal: AbortSignal.any([
          this.controller.signal,
          AbortSignal.timeout(60_000),
        ]),
        cache: "no-store",
      });
      if (!response.ok) throw new Error("News unavailable");
      const next = await response.json();
      if (
        !Array.isArray(next.items) ||
        !Array.isArray(next.live?.stories) ||
        !next.market
      )
        throw new Error("Invalid news response");
      if (!this.active || generation !== this.generation) return;
      const seen = new Set(this.data?.items.map((item) => item.id));
      if (!this.data || next.live.sequence >= this.data.live.sequence)
        this.data = next;
      if (next.live.sequence > this.sequence) {
        const accepted = new Set(
          this.getState().live?.events.map((event) => event.id),
        );
        const retained = new Set([
          ...accepted,
          ...next.items.map((item) => item.id),
        ]);
        for (const id of this.sources.keys())
          if (!retained.has(id)) this.sources.delete(id);
        for (const item of next.items) {
          if (!accepted.has(item.id))
            this.sources.set(item.id, structuredClone(item));
        }
        this.sequence = next.live.sequence;
        this.deliver(next.live);
      }
      const healthy = next.sources.filter(
        (source) => source.status === "ok",
      ).length;
      $("#news-status").textContent =
        `${next.refreshing ? "Refreshing" : "Shared feed"} · ${healthy}/${next.sources.length} sources · ${next.jevConfigured ? "Jev" : "Jev key missing"}`;
      $("#news-status").title =
        `Checked ${new Date(next.checkedAt).toLocaleTimeString()} · ${next.cache?.backend ?? "cache"}`;
      this.timeline.arrival(next.items.find((item) => !seen.has(item.id)));
    } catch {
      if (generation === this.generation && this.active)
        $("#news-status").textContent =
          "Feed unavailable · retrying automatically";
    } finally {
      if (generation === this.generation) {
        this.controller = null;
        $("#news-refresh").disabled = !this.active;
        if (this.active)
          this.timer = setTimeout(
            () => this.refresh(),
            this.data?.refreshing ? 3000 : 30_000,
          );
      }
    }
  }

  update(state) {
    if (state.phase === "playing" && !this.active) this.begin();
    else if (!running(state.phase) && this.active) this.stop();
    $("#news-refresh").disabled = !this.active || Boolean(this.controller);
    $("#news-hold").disabled = !this.active;
    $("#news-mode").textContent = this.active
      ? "LIVE DECISIONS · THIS RUN"
      : "START A RUN TO CONNECT";
    if (!this.active)
      $("#news-status").textContent = "Timeline starts when you play.";
    this.timeline.update(state);
  }
}
