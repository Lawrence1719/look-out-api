import { expect, test } from "@playwright/test";

test("incident demonstration honors thresholds and manual controls", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Keep an eye on your APIs." }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Play demonstration" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Observation 3: HTTP 503", exact: true })
    .click();
  await expect(page.locator(".demo-state strong")).toHaveText("UP");
  await expect(page.locator(".demo-notification")).toContainText(
    "No notification",
  );
  await page
    .getByRole("button", { name: "Observation 4: HTTP 503", exact: true })
    .click();
  await expect(page.locator(".demo-state strong")).toHaveText("DOWN");
  await expect(page.locator(".demo-notification")).toContainText(
    "DOWN · orders-api",
  );
  await page
    .getByRole("button", { name: "Observation 6: HTTP 200", exact: true })
    .click();
  await expect(page.locator(".demo-notification")).toContainText("RECOVERED");
});

test("check examples support keyboard navigation", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("tab", { name: "HTTP", exact: true }).focus();
  await page.keyboard.press("ArrowDown");
  await expect(
    page.getByRole("tab", { name: "TCP", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("tabpanel")).toContainText('type = "tcp"');
  await page.getByRole("tab", { name: "Flows", exact: true }).click();
  await expect(page.getByRole("tabpanel")).toContainText("{{token}}");
});

test("copy buttons report success and preserve the code", async ({
  page,
  context,
  browserName,
}) => {
  test.skip(
    browserName !== "chromium",
    "Clipboard permissions are Chromium-specific.",
  );
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/");
  await page
    .getByRole("tabpanel")
    .getByRole("button", { name: "Copy code" })
    .click();
  await expect(
    page.getByRole("button", { name: "Copied to clipboard" }),
  ).toBeVisible();
  const content = await page.evaluate(() => navigator.clipboard.readText());
  expect(content).toContain('name = "orders-api"');
});

test("mobile navigation, docs and both routes have no horizontal overflow", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/");
  await page.getByRole("button", { name: "Open navigation" }).click();
  await expect(
    page.getByRole("navigation", { name: "Main navigation" }),
  ).toBeVisible();
  await page
    .getByRole("complementary")
    .getByRole("link", { name: "Documentation", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Start your watch." }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.goto("/");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});

test("exported pages load without script errors and expose share metadata", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/");
  await expect(page.locator('meta[property="og:image"]')).toHaveAttribute(
    "content",
    /social\.png$/,
  );
  await page
    .getByRole("complementary")
    .getByRole("link", { name: "Documentation", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Start your watch." }),
  ).toBeVisible();
  await page
    .getByRole("navigation", { name: "On this page" })
    .getByRole("link", { name: "Dashboard", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Monitor from your browser" }),
  ).toBeVisible();
  await expect(page.locator("#dashboard")).toContainText(
    "Save & start monitoring",
  );
  await expect(
    page.getByRole("link", { name: "open the local dashboard", exact: true }),
  ).toHaveAttribute("href", "http://127.0.0.1:8080/");
  expect(errors).toEqual([]);
});

test("main page and docs pass automated accessibility checks", async ({
  page,
}) => {
  const { default: AxeBuilder } = await import("@axe-core/playwright");
  for (const route of ["/", "/docs/"]) {
    await page.goto(route);
    await page.emulateMedia({ reducedMotion: "reduce" });
    const result = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
      .analyze();
    expect(result.violations).toEqual([]);
  }
});
