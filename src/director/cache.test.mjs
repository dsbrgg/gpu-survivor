import { test } from "node:test";
import assert from "node:assert/strict";
import { DecisionCache } from "./cache.mjs";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawn } from "node:child_process";

test("independent processes coalesce one cold computation", async () => {
  const directory = mkdtempSync(join(tmpdir(), "survivor-process-"));
  const path = join(directory, "cache.sqlite");
  const initialize = new DecisionCache(path);
  await initialize.close();
  const run = () =>
    new Promise((resolve, reject) => {
      const child = spawn(
        process.execPath,
        [
          "--input-type=module",
          "-e",
          `
      import { DecisionCache } from ${JSON.stringify(new URL("./cache.mjs", import.meta.url).href)};
      import { setTimeout } from 'node:timers/promises';
      const cache = new DecisionCache(${JSON.stringify(path)});
      const value = await cache.remember('shared', async () => {
        console.log('computed'); await setTimeout(100); return { decision: 'bear' };
      }, { ttlMs: 10000 });
      console.log(value.decision); await cache.close();
    `,
        ],
        { stdio: ["ignore", "pipe", "pipe"] },
      );
      let output = "",
        errors = "";
      child.stdout.on("data", (data) => (output += data));
      child.stderr.on("data", (data) => (errors += data));
      child.on("error", reject);
      child.on("exit", (code) =>
        code === 0 ? resolve(output) : reject(new Error(errors)),
      );
    });
  try {
    const outputs = await Promise.all([run(), run(), run()]);
    assert.equal(outputs.join("").match(/computed/g).length, 1);
    assert.ok(outputs.every((output) => output.includes("bear")));
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("expired leases recover and old workers cannot replace the new decision", async () => {
  const directory = mkdtempSync(join(tmpdir(), "survivor-lease-"));
  let time = 1000,
    release;
  const gate = new Promise((resolve) => (release = resolve));
  const path = join(directory, "cache.sqlite");
  const one = new DecisionCache(path, () => time),
    two = new DecisionCache(path, () => time);
  try {
    const old = one.remember(
      "key",
      async () => {
        await gate;
        return "old";
      },
      { ttlMs: 100, leaseMs: 10 },
    );
    time += 11;
    assert.equal(
      await two.remember("key", async () => "new", { ttlMs: 100 }),
      "new",
    );
    release();
    await old;
    assert.equal(one.get("key"), "new");
    time += 101;
    assert.equal(one.get("key"), null);
  } finally {
    release();
    await one.close();
    await two.close();
    rmSync(directory, { recursive: true, force: true });
  }
});
