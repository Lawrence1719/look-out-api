import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test.beforeEach(async ({ request }) => {
  await request.post("http://127.0.0.1:4182/control/up");
});

async function selectDemo(page: import("@playwright/test").Page) {
  await page.goto("/");
  await page.getByRole("button", { name: /demo-api/ }).click();
  await expect(
    page.getByRole("heading", { name: "demo-api", exact: true }),
  ).toBeVisible();
}

test("dashboard manual checks reach the real API and show assertion failures", async ({
  page,
  request,
}) => {
  await selectDemo(page);
  const run = page.getByRole("button", { name: "Run check", exact: true });
  await expect(run).toBeEnabled();
  const before = await (
    await request.get("http://127.0.0.1:4182/requests")
  ).json();
  await run.click();
  await expect(
    page.getByText("Check in progress. Waiting for the API result…"),
  ).toBeVisible();
  await expect(page.locator("#activity")).toHaveText("Manual check running");
  await expect(page.locator("#manual-result")).toContainText("PASS");
  await expect(page.locator("#manual-result")).toContainText("HTTP 200");
  expect(
    await (await request.get("http://127.0.0.1:4182/requests")).json(),
  ).toBeGreaterThan(before);
  await expect(page.locator("#check-rules")).toContainText(
    "Verified in last manual test",
  );
  await request.post("http://127.0.0.1:4182/control/mismatch");
  await expect(run).toBeEnabled();
  await run.click();
  await expect(page.locator("#manual-result")).toContainText("FAIL");
  await expect(page.locator("#manual-result")).toContainText(
    "expected response text was not found",
  );
  await expect(page.locator("#manual-result")).toContainText("HTTP 200");
  await page.getByRole("button", { name: /checkout-flow/ }).click();
  await expect(run).toBeEnabled();
  await run.click();
  await expect(page.locator("#manual-result")).toContainText("2 of 2 executed");
  await expect(page.locator("#manual-result")).toContainText("create-session");
  await expect(page.locator("#manual-result")).toContainText("read-orders");
  await page.getByRole("button", { name: /slow-api/ }).click();
  await expect(run).toBeEnabled();
  await run.click();
  await expect(page.locator("#manual-result")).toContainText("FAIL");
  await expect(page.locator("#manual-result")).toContainText("timed out");
});

test("scheduled observations visibly open and recover a real local incident", async ({
  page,
  request,
}) => {
  test.setTimeout(45_000);
  await selectDemo(page);
  await request.post("http://127.0.0.1:4182/control/down");
  await expect(page.locator("#incident-state")).toContainText("DOWN", {
    timeout: 18_000,
  });
  await expect(page.locator("#history-rows")).toContainText("503");
  await expect(page.locator("#incidents")).toContainText("OPEN");
  await request.post("http://127.0.0.1:4182/control/up");
  await expect(page.locator("#incident-state")).toContainText("UP", {
    timeout: 18_000,
  });
  await expect(page.locator("#incidents")).toContainText("RECOVERED");
});

test("dashboard handles disconnect, empty state, keyboard focus and mobile layout", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await selectDemo(page);
  const target = page.getByRole("button", { name: /demo-api/ });
  await target.focus();
  await page.waitForTimeout(2200);
  await expect(target).toBeFocused();
  await page.route("**/api/dashboard", (route) => route.abort());
  await page.getByRole("button", { name: "Refresh monitoring data" }).click();
  await expect(page.getByRole("alert").first()).toContainText(
    "Cannot reach the local monitor",
  );
  await expect(
    page.getByRole("button", { name: "Run check", exact: true }),
  ).toBeDisabled();
  await page.unroute("**/api/dashboard");
  await page.getByRole("button", { name: "Refresh monitoring data" }).click();
  await expect(page.locator("#connection-label")).toContainText("Connected");
  await page.setViewportSize({ width: 375, height: 812 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  expect(errors).toEqual([]);
  await page.route("**/api/dashboard", (route) =>
    route.fulfill({
      json: { server_time: new Date().toISOString(), targets: [] },
    }),
  );
  await page.getByRole("button", { name: "Refresh monitoring data" }).click();
  await expect(
    page.getByRole("heading", { name: "No configured targets" }),
  ).toBeVisible();
});
