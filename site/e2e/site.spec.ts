import { expect, test } from "@playwright/test";

test("landing page has one clear heading and no console errors", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", (message) => message.type() === "error" && errors.push(message.text()));
  await page.goto("/");
  await expect(page).toHaveTitle(/Migration Replay Gate/);
  await expect(page.locator("h1")).toHaveCount(1);
  await expect(page.locator("main")).toBeVisible();
  await expect(page.getByRole("heading", { name: /Trust the replay/ })).toBeVisible();
  expect(errors).toEqual([]);
});

for (const legalPage of [
  { path: "/privacy/", title: /Privacy/ },
  { path: "/terms/", title: /Terms/ }
]) {
  test(`${legalPage.path} loads without CSP console errors under production headers`, async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (message) => message.type() === "error" && errors.push(message.text()));
    page.on("pageerror", (error) => errors.push(error.message));

    const response = await page.goto(legalPage.path);
    expect(response?.headers()["content-security-policy"]).toContain("style-src 'self'");
    await expect(page).toHaveTitle(legalPage.title);
    await expect(page.locator("style")).toHaveCount(0);
    await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);
    await expect(page.locator("main")).toBeVisible();
    expect(errors).toEqual([]);
  });
}

test("recorded partial replay exposes an actionable blocked state", async ({ page }) => {
  await page.goto("/#replay");
  await page.getByRole("tab", { name: /Partial state/ }).click();
  await page.getByRole("button", { name: /Run recorded replay/ }).click();
  await expect(page.getByText("BLOCKED / PARTIAL-STATE FAILURE")).toBeVisible();
  await expect(page.getByText(/already exists/)).toBeVisible();
});

test("replay rail supports arrow keys", async ({ page }) => {
  await page.goto("/#replay");
  const clean = page.getByRole("tab", { name: /Clean apply/ });
  await clean.focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("tab", { name: /Repeat apply/ })).toHaveAttribute("aria-selected", "true");
});

test("mobile layout keeps primary actions and replay usable", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "mobile");
  await page.goto("/");
  await expect(page.getByRole("link", { name: "Install the CLI" })).toBeVisible();
  await page.getByRole("tab", { name: /Repeat apply/ }).click();
  await page.getByRole("button", { name: /Run recorded replay/ }).click();
  await expect(page.getByText("PASS / IDEMPOTENT")).toBeVisible();
});

test("offline reload keeps the recorded replay available", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload();
  await page.context().setOffline(true);
  try {
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByRole("heading", { name: /Trust the replay/ })).toBeVisible();
    await page.getByRole("tab", { name: /Partial state/ }).click();
    await page.getByRole("button", { name: /Run recorded replay/ }).click();
    await expect(page.getByText("BLOCKED / PARTIAL-STATE FAILURE")).toBeVisible();
  } finally {
    await page.context().setOffline(false);
  }
});

test("landing page makes no third-party requests", async ({ page }) => {
  const requests: string[] = [];
  page.on("request", (request) => {
    if (request.url().startsWith("http")) requests.push(request.url());
  });

  await page.goto("/");
  expect(requests.every((url) => new URL(url).origin === "http://127.0.0.1:4173")).toBe(true);
});
