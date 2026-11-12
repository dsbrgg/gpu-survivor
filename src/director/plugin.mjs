import { loadEnvFile } from "node:process";
import { fileURLToPath } from "node:url";
import { NewsDirector } from "./news.mjs";
import { DecisionCache } from "./cache.mjs";

export function newsPlugin() {
  let director;
  function install(server) {
    try {
      loadEnvFile(fileURLToPath(new URL("../../.env", import.meta.url)));
    } catch (error) {
      if (error.code !== "ENOENT")
        throw new Error("Could not load the server environment file.");
    }
    director ??= new NewsDirector({
      apiKey: process.env.TYPESAFE_API_KEY,
      cache: new DecisionCache(
        fileURLToPath(new URL("../../.cache/news.sqlite", import.meta.url)),
      ),
    });
    server.httpServer?.once("close", () => {
      void director.close();
    });
    server.middlewares.use(async (request, response, next) => {
      const path = request.url?.split("?")[0];
      if (path !== "/api/news") return next();
      response.setHeader("Content-Type", "application/json");
      response.setHeader("Cache-Control", "no-store");
      if (request.method !== "GET") {
        response.statusCode = 405;
        return response.end(JSON.stringify({ error: "Method not allowed" }));
      }
      try {
        response.end(JSON.stringify(await director.snapshot()));
      } catch {
        response.statusCode = 503;
        response.end(JSON.stringify({ error: "News service unavailable" }));
      }
    });
  }
  return {
    name: "gpu-news-director",
    configureServer: install,
    configurePreviewServer: install,
  };
}
