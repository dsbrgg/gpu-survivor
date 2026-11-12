import { DatabaseSync } from "node:sqlite";
import { mkdirSync } from "node:fs";
import { dirname } from "node:path";
import { randomUUID } from "node:crypto";
import { setTimeout as delay } from "node:timers/promises";

export class DecisionCache {
  inflight = new Map();

  constructor(path = ":memory:", clock = Date.now) {
    this.clock = clock;
    if (path !== ":memory:") mkdirSync(dirname(path), { recursive: true });
    this.database = new DatabaseSync(path);
    this.database.exec(`
      PRAGMA busy_timeout = 5000;
      PRAGMA journal_mode = WAL;
      CREATE TABLE IF NOT EXISTS entries (
        key TEXT PRIMARY KEY, value TEXT NOT NULL,
        expires_at INTEGER NOT NULL, retain_until INTEGER NOT NULL
      );
      CREATE TABLE IF NOT EXISTS leases (
        key TEXT PRIMARY KEY, owner TEXT NOT NULL, expires_at INTEGER NOT NULL
      );
    `);
  }

  get(key, { stale = false } = {}) {
    const row = this.database
      .prepare("SELECT * FROM entries WHERE key = ?")
      .get(key);
    if (
      !row ||
      row.retain_until <= this.clock() ||
      (!stale && row.expires_at <= this.clock())
    )
      return null;
    try {
      return JSON.parse(row.value);
    } catch {
      this.database.prepare("DELETE FROM entries WHERE key = ?").run(key);
      return null;
    }
  }

  remember(key, compute, { ttlMs, retainMs = ttlMs, leaseMs = 180_000 } = {}) {
    const cached = this.get(key);
    if (cached !== null) return Promise.resolve(cached);
    if (this.inflight.has(key)) return this.inflight.get(key);
    const promise = this.compute(key, compute, {
      ttlMs,
      retainMs,
      leaseMs,
    }).finally(() => this.inflight.delete(key));
    this.inflight.set(key, promise);
    return promise;
  }

  async compute(key, compute, { ttlMs, retainMs, leaseMs }) {
    const owner = randomUUID();
    const deadline = Date.now() + leaseMs + 5000;
    while (Date.now() < deadline) {
      const cached = this.get(key);
      if (cached !== null) return cached;
      const now = this.clock();
      const claim = this.database
        .prepare(
          `
        INSERT INTO leases VALUES (?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET owner = excluded.owner, expires_at = excluded.expires_at
        WHERE leases.expires_at <= ?
      `,
        )
        .run(key, owner, now + leaseMs, now);
      if (claim.changes) {
        try {
          const value = await compute();
          const ttl = typeof ttlMs === "function" ? ttlMs(value) : ttlMs;
          const retention =
            typeof retainMs === "function" ? retainMs(value) : retainMs;
          // An expired worker cannot overwrite a newer lease holder's result.
          this.database
            .prepare(
              `
            INSERT INTO entries SELECT ?, ?, ?, ? WHERE EXISTS (SELECT 1 FROM leases WHERE key = ? AND owner = ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, expires_at = excluded.expires_at, retain_until = excluded.retain_until
          `,
            )
            .run(
              key,
              JSON.stringify(value),
              this.clock() + ttl,
              this.clock() + Math.max(ttl, retention),
              key,
              owner,
            );
          this.database
            .prepare("DELETE FROM entries WHERE retain_until <= ?")
            .run(this.clock());
          return value;
        } finally {
          this.database
            .prepare("DELETE FROM leases WHERE key = ? AND owner = ?")
            .run(key, owner);
        }
      }
      await delay(25);
    }
    throw new Error("Shared news cache refresh timed out");
  }

  async close() {
    await Promise.allSettled(this.inflight.values());
    this.database.close();
  }
}
