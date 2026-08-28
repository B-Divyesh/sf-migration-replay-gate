import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const configPath = new URL("../public/staticwebapp.config.json", import.meta.url);
const legalPages = [
  new URL("../privacy/index.html", import.meta.url),
  new URL("../terms/index.html", import.meta.url)
];

test("static deployment config gives immutable assets and service worker update semantics", async () => {
  const config = JSON.parse(await readFile(configPath, "utf8"));
  const routes = new Map(
    config.routes.map((route) => [route.route, route.headers["Cache-Control"]])
  );

  assert.equal(routes.get("/assets/*"), "public, max-age=31536000, immutable");
  for (const image of [
    "/replay-landscape-480.webp",
    "/replay-landscape-720.webp",
    "/replay-landscape.webp"
  ]) {
    assert.equal(routes.get(image), "public, max-age=31536000, immutable");
  }
  assert.equal(routes.get("/sw.js"), "no-cache, max-age=0, must-revalidate");
});

test("legal pages use a same-origin stylesheet compatible with the strict CSP", async () => {
  const config = JSON.parse(await readFile(configPath, "utf8"));
  assert.match(config.globalHeaders["Content-Security-Policy"], /style-src 'self'/);

  for (const legalPage of legalPages) {
    const html = await readFile(legalPage, "utf8");
    assert.doesNotMatch(html, /<style[\s>]/i);
    assert.match(html, /<link\s+rel="stylesheet"\s+href="\/src\/legal\.css"\s*\/>/i);
  }
});
