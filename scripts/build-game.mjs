import { execFileSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { dirname, join } from "node:path";

execFileSync(
  "cargo",
  ["build", "--release", "--target", "wasm32-unknown-unknown"],
  {
    stdio: "inherit",
  },
);
copyFileSync(
  "target/wasm32-unknown-unknown/release/gpu-survivor.wasm",
  "src/web/public/game.wasm",
);
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--locked", "--format-version", "1"], {
    encoding: "utf8",
  }),
);
const miniquad = metadata.packages.find(
  (package_) => package_.name === "miniquad",
);
const source = dirname(miniquad.manifest_path);
copyFileSync(join(source, "js/gl.js"), "src/web/public/vendor/miniquad.js");
copyFileSync(
  join(source, "LICENSE-MIT"),
  "src/web/public/vendor/MINIQUAD-LICENSE",
);
