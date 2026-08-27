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
