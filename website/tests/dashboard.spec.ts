import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test.beforeEach(async ({ request }) => {
  await request.post("http://127.0.0.1:4182/control/up");
  const snapshot = await (
    await request.get("http://127.0.0.1:4181/api/dashboard")
  ).json();
  for (const target of snapshot.targets) {
    if (target.paused)
      await request.post(
        `http://127.0.0.1:4181/api/targets/${encodeURIComponent(target.name)}/resume`,
        {
          headers: {
            Origin: "http://127.0.0.1:4181",
            "X-Look-Out-Action": "resume-target",
          },
        },
      );
  }
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
  // Isolate explicit manual requests from background scheduler contention.
  const snapshot = await (
    await request.get("http://127.0.0.1:4181/api/dashboard")
  ).json();
  for (const target of snapshot.targets) {
    const response = await request.post(
      `http://127.0.0.1:4181/api/targets/${encodeURIComponent(target.name)}/pause`,
      {
        headers: {
          Origin: "http://127.0.0.1:4181",
          "X-Look-Out-Action": "pause-target",
        },
      },
    );
    expect(response.status()).toBe(200);
  }
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

test("add API validates a draft and starts real scheduled monitoring without restart", async ({
  page,
}) => {
  await selectDemo(page);
  const resources = page.getByRole("navigation", { name: "Project resources" });
  await expect(
    resources.getByRole("link", { name: "Website", exact: true }),
  ).toHaveAttribute("href", "https://lawrence1719.github.io/look-out-api/");
  await expect(
    resources.getByRole("link", { name: "Documentation", exact: true }),
  ).toHaveAttribute(
    "href",
    "https://lawrence1719.github.io/look-out-api/docs/",
  );
  await expect(
    resources.getByRole("link", { name: "Documentation", exact: true }),
  ).toHaveAttribute("target", "_blank");
  await page.getByRole("button", { name: "Add API", exact: true }).click();
  const name = `new-health-${Date.now()}`;
  await page.getByLabel("Name", { exact: true }).fill(name);
  await page
    .getByLabel("API URL", { exact: true })
    .fill("http://127.0.0.1:4182/health");
  await page.getByLabel("Response contains", { exact: false }).fill("demo-ok");
  await page.getByLabel("Check interval (seconds)").fill("5");
  const timeout = page.getByLabel("Timeout (seconds)");
  await timeout.fill("5");
  const save = page.getByRole("button", { name: "Save & start monitoring" });
  await save.click();
  await expect(timeout).toBeFocused();
  await timeout.fill("1");
  await page.waitForTimeout(2200);
  await expect(page.getByLabel("Name", { exact: true })).toHaveValue(name);
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  await save.click();
  await expect(page.locator("#add-api-notice")).toContainText(
    "Monitoring starts now",
  );
  await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
  await expect(page.locator("#history-rows")).toContainText("200", {
    timeout: 8000,
  });
  await expect(page.locator("#last-observation")).toContainText("PASS");
  await page.reload();
  await page.getByRole("button", { name: new RegExp(name) }).click();
  await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Add API", exact: true }).click();
  await page.getByLabel("Name", { exact: true }).fill(name);
  await page
    .getByLabel("API URL", { exact: true })
    .fill("http://127.0.0.1:4182/health");
  await save.click();
  await expect(page.locator("#add-api-error")).toContainText("already exists");
  await expect(page.getByLabel("Name", { exact: true })).toBeFocused();
  await expect(page.getByLabel("API URL", { exact: true })).toHaveValue(
    "http://127.0.0.1:4182/health",
  );
  await page.setViewportSize({ width: 375, height: 812 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Add API", exact: true }),
  ).toBeFocused();
});

test("custom dropdown supports keyboard, pointer, reset, mobile, and real HEAD monitoring", async ({
  page,
}) => {
  await selectDemo(page);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.getByRole("button", { name: "Add API", exact: true }).click();
  const method = page.getByRole("combobox", { name: "Method", exact: true });
  const options = page.getByRole("listbox", { name: "Method", exact: true });
  await expect(method).toHaveText("GET");
  await method.focus();
  await method.press("ArrowDown");
  await expect(options).toBeVisible();
  await method.press("End");
  await method.press("Enter");
  await expect(method).toHaveText("HEAD");
  await expect(
    page.getByLabel("Response contains", { exact: false }),
  ).toBeDisabled();
  await method.press("ArrowUp");
  await method.press("Home");
  await method.press("Escape");
  await expect(method).toHaveText("HEAD");
  await expect(method).toBeFocused();
  await method.click();
  await options.getByRole("option", { name: "GET", exact: true }).click();
  await expect(method).toHaveText("GET");
  await expect(
    page.getByLabel("Response contains", { exact: false }),
  ).toBeEnabled();
  await method.press("h");
  await method.press("Tab");
  await expect(method).toHaveText("HEAD");
  await expect(
    page.getByLabel("Expected status", { exact: false }),
  ).toBeFocused();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await page.getByRole("button", { name: "Add API", exact: true }).click();
  await expect(method).toHaveText("GET");
  await expect(
    page.getByLabel("Response contains", { exact: false }),
  ).toBeEnabled();
  await method.click();
  await page.getByLabel("Name", { exact: true }).click();
  await expect(options).toBeHidden();
  await expect(page.getByLabel("Name", { exact: true })).toBeFocused();
  await method.click();
  await page.waitForTimeout(2200);
  await expect(options).toBeVisible();
  await expect(method).toBeFocused();
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  await method.press("Escape");
  await page.setViewportSize({ width: 375, height: 812 });
  await method.focus();
  await method.evaluate((node) =>
    scrollTo({
      top: node.getBoundingClientRect().top + scrollY - 250,
      behavior: "instant",
    }),
  );
  const scrollBeforeOpen = await page.evaluate(() => scrollY);
  await method.press("ArrowDown");
  expect(await page.evaluate(() => scrollY)).toBe(scrollBeforeOpen);
  const bounds = await options.boundingBox();
  expect(bounds!.x).toBeGreaterThanOrEqual(0);
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(375);
  expect(bounds!.y).toBeGreaterThanOrEqual(0);
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(812);
  await options.getByRole("option", { name: "HEAD", exact: true }).click();
  const name = `head-health-${Date.now()}`;
  await page.getByLabel("Name", { exact: true }).fill(name);
  await page
    .getByLabel("API URL", { exact: true })
    .fill("http://127.0.0.1:4182/health");
  await page.getByLabel("Check interval (seconds)").fill("5");
  await page.getByLabel("Timeout (seconds)").fill("1");
  const sent = page.waitForRequest(
    (request) =>
      request.method() === "POST" && request.url().endsWith("/api/targets"),
  );
  await page.getByRole("button", { name: "Save & start monitoring" }).click();
  expect((await sent).postDataJSON()).toMatchObject({
    method: "HEAD",
    expected_body_contains: null,
  });
  await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
  await expect(page.locator("#history-rows")).toContainText("200", {
    timeout: 8000,
  });
});

test("shared dropdown handles required values, disabled options and multiple controls", async ({
  page,
}) => {
  await selectDemo(page);
  await page.getByRole("button", { name: "Add API", exact: true }).click();
  await page.evaluate(() => {
    const form = document.createElement("form");
    form.id = "dropdown-fixture";
    const fieldset = document.createElement("fieldset");
    const label = document.createElement("label");
    label.htmlFor = "fixture-choice";
    label.textContent = "Fixture choice";
    const select = document.createElement("select");
    select.id = "fixture-choice";
    select.name = "choice";
    select.required = true;
    for (const [text, value, disabled] of [
      ["Choose a value", "", false],
      ["Unavailable", "unavailable", true],
      ["Alpha", "alpha", false],
      ["Beta", "beta", false],
    ] as const) {
      const option = new Option(text, value);
      option.disabled = disabled;
      select.append(option);
    }
    fieldset.append(label, select);
    form.append(fieldset);
    document.querySelector("main")!.append(form);
    (
      window as unknown as { LookOutDropdowns: { enhance: () => void } }
    ).LookOutDropdowns.enhance();
  });
  const choice = page.getByRole("combobox", {
    name: "Fixture choice",
    exact: true,
  });
  expect(
    await page
      .locator("#dropdown-fixture")
      .evaluate((form) => (form as HTMLFormElement).reportValidity()),
  ).toBe(false);
  await expect(choice).toBeFocused();
  await expect(choice).toHaveAttribute("aria-invalid", "true");
  await expect(page.locator("#fixture-choice-validation")).toBeVisible();
  await choice.press("ArrowDown");
  await choice.press("ArrowDown");
  await choice.press("Enter");
  await expect(choice).toHaveText("Alpha");
  await expect(page.locator("#fixture-choice-validation")).toBeHidden();
  expect(
    await page
      .locator("#dropdown-fixture")
      .evaluate((form) => new FormData(form as HTMLFormElement).get("choice")),
  ).toBe("alpha");
  await choice.click();
  await page.getByRole("combobox", { name: "Method", exact: true }).click();
  await expect(
    page.getByRole("listbox", { name: "Fixture choice", exact: true }),
  ).toBeHidden();
  await expect(
    page.getByRole("listbox", { name: "Method", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("combobox", { name: "Method", exact: true })
    .press("Escape");
  await page.locator("#dropdown-fixture fieldset").evaluate((fieldset) => {
    (fieldset as HTMLFieldSetElement).disabled = true;
  });
  await expect(choice).toBeDisabled();
  await page.locator("#dropdown-fixture fieldset").evaluate((fieldset) => {
    (fieldset as HTMLFieldSetElement).disabled = false;
  });
  await expect(choice).toBeEnabled();
  await page
    .locator("#dropdown-fixture")
    .evaluate((form) => (form as HTMLFormElement).reset());
  await expect(choice).toHaveText("Choose a value");
  await page.route("**/dropdown.js", (route) => route.abort());
  await page.reload();
  await page.getByRole("button", { name: "Add API", exact: true }).click();
  await expect(page.locator("#api-method")).toBeVisible();
});

test("target monitoring pauses without new observations and resumes on its schedule", async ({
  page,
  request,
}) => {
  await selectDemo(page);
  await expect(page.locator("#history-rows")).toContainText("200", {
    timeout: 8000,
  });
  const pause = page.getByRole("button", {
    name: "Pause monitoring",
    exact: true,
  });
  await pause.click();
  const resume = page.getByRole("button", {
    name: "Resume monitoring",
    exact: true,
  });
  await expect(resume).toBeEnabled();
  await expect(resume.locator("#monitor-resume-icon")).toBeVisible();
  await expect(resume.locator("#monitor-pause-icon")).toBeHidden();
  await expect(page.locator("#monitoring-note")).toContainText(
    "Monitoring paused",
  );
  await expect(page.locator("#target-list")).toContainText("PAUSED");
  await expect(resume).toBeFocused();
  const history = (
    await (await request.get("/api/dashboard")).json()
  ).targets.find(
    (target: { name: string }) => target.name === "demo-api",
  ).history;
  await page.waitForTimeout(5600);
  const paused = (
    await (await request.get("/api/dashboard")).json()
  ).targets.find((target: { name: string }) => target.name === "demo-api");
  expect(paused.paused).toBe(true);
  expect(paused.history).toEqual(history);
  await page.getByRole("button", { name: "Run check", exact: true }).click();
  await expect(page.locator("#manual-result")).toContainText("PASS");
  await expect(resume).toBeVisible();
  await page.reload();
  await page.getByRole("button", { name: /demo-api/ }).click();
  await expect(resume).toBeVisible();
  await page.setViewportSize({ width: 375, height: 812 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  await page.route("**/api/targets/demo-api/resume", (route) =>
    route.fulfill({ status: 500, json: { error: "simulated local failure" } }),
  );
  await resume.click();
  await expect(page.locator("#monitoring-error")).toContainText(
    "could not be changed",
  );
  await expect(resume).toBeEnabled();
  await page.unroute("**/api/targets/demo-api/resume");
  await resume.click();
  await expect(pause).toBeEnabled();
  await expect
    .poll(
      async () => {
        const target = (
          await (await request.get("/api/dashboard")).json()
        ).targets.find((item: { name: string }) => item.name === "demo-api");
        return target.history[0].checked_at;
      },
      { timeout: 8000 },
    )
    .not.toBe(history[0].checked_at);

  // Model a durable pause before the first scheduled observation.
  const emptySnapshot = await (await request.get("/api/dashboard")).json();
  const emptyTarget = emptySnapshot.targets.find(
    (target: { name: string }) => target.name === "demo-api",
  );
  emptyTarget.history = [];
  emptyTarget.paused = true;
  await page.route("**/api/dashboard", (route) =>
    route.fulfill({ json: emptySnapshot }),
  );
  await page.getByRole("button", { name: "Refresh monitoring data" }).click();
  await expect(page.locator("#threshold-note")).toContainText(
    "Scheduled checks begin after monitoring resumes",
  );
  await expect(page.locator("#history-rows")).toContainText(
    "Resume to record scheduled observations",
  );
  emptyTarget.paused = false;
  await page.getByRole("button", { name: "Refresh monitoring data" }).click();
  await expect(page.locator("#threshold-note")).toContainText(
    "The first check runs shortly after the daemon starts",
  );
  await expect(page.locator("#history-rows")).toContainText(
    "Waiting for the first scheduled check",
  );
});
