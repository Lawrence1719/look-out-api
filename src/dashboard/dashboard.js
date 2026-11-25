"use strict";

(() => {
  const byId = (id) => document.getElementById(id);
  let snapshot = null;
  let selected = null;
  let connected = false;
  let refreshPromise = null;
  let formOpen = false;
  let saving = false;
  let timer;
  let detailKey = "";
  let targetKey = "";
  const pending = new Set();
  const actionErrors = new Map();
  const date = (value) =>
    value
      ? new Date(value).toLocaleString(undefined, {
          month: "short",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
        })
      : "Never";
  const element = (tag, text, className) => {
    const node = document.createElement(tag);
    if (text !== undefined) node.textContent = text;
    if (className) node.className = className;
    return node;
  };
  const badge = (text, state) => element("span", text, `badge ${state}`);
  const replace = (id, ...nodes) => byId(id).replaceChildren(...nodes);
  const statusLabel = (target) =>
    target.stale ? "UNKNOWN / STALE" : target.status.toUpperCase();
  const stateClass = (target) => (target.stale ? "unknown" : target.status);

  function renderTargets() {
    const names = snapshot.targets.map((target) => target.name);
    const key = JSON.stringify(names);
    if (key !== targetKey) {
      const list = byId("target-list");
      const existing = new Map(
        [...list.querySelectorAll("button")].map((button) => [
          button.dataset.target,
          button,
        ]),
      );
      if (!existing.size) list.replaceChildren();
      names.forEach((name, index) => {
        let button = existing.get(name);
        if (!button) {
          button = element("button", undefined, "target-button");
          button.type = "button";
          button.dataset.target = name;
          button.append(element("strong", name));
          const meta = element("span", undefined, "target-meta");
          meta.append(element("span"), element("span"));
          button.append(meta);
          button.addEventListener("click", () => {
            if (saving) return;
            selected = name;
            formOpen = false;
            detailKey = "";
            render();
          });
        }
        if (list.children[index] !== button)
          list.insertBefore(button, list.children[index] || null);
      });
      for (const button of existing.values())
        if (!names.includes(button.dataset.target)) button.remove();
      targetKey = key;
    }
    [...byId("target-list").children].forEach((button, index) => {
      const target = snapshot.targets[index];
      button.setAttribute("aria-current", String(target.name === selected));
      const meta = button.querySelector(".target-meta");
      meta.children[0].textContent = target.type.toUpperCase();
      meta.children[1].textContent = statusLabel(target);
      meta.children[1].className = stateClass(target);
    });
    byId("target-count").textContent = String(names.length);
  }

  function stepsNode(steps, title) {
    const block = element("div", undefined, "steps");
    block.append(element("h4", title));
    for (const step of steps) {
      const row = element("div", undefined, "step");
      row.append(
        badge(step.ok ? "PASS" : "FAIL", step.ok ? "pass" : "fail"),
        element("span", step.name, "step-name"),
        element(
          "span",
          `${step.status_code ? `HTTP ${step.status_code} · ` : ""}${step.latency_ms} ms`,
        ),
      );
      block.append(row);
      if (step.error) block.append(element("p", step.error, "secondary"));
    }
    return block;
  }

  function renderManual(target) {
    const result = target.manual_result;
    if (!result) {
      replace(
        "manual-result",
        element(
          "p",
          "No completed manual test yet. Run a check to inspect its result here.",
          "secondary",
        ),
      );
    } else {
      const heading = element("div", undefined, "result-heading");
      heading.append(
        badge(result.ok ? "PASS" : "FAIL", result.ok ? "pass" : "fail"),
        element(
          "strong",
          `${result.status_code ? `HTTP ${result.status_code} · ` : ""}${result.latency_ms} ms`,
        ),
      );
      const nodes = [
        heading,
        element(
          "p",
          `Manual test completed ${date(result.checked_at)}`,
          "result-time",
        ),
        element(
          "p",
          result.error || "Configured checks passed.",
          "result-detail",
        ),
      ];
      if (result.steps.length)
        nodes.push(
          stepsNode(
            result.steps,
            `Flow steps · ${result.steps.length} of ${target.flow_steps} executed`,
          ),
        );
      for (const finding of result.findings)
        nodes.push(
          element(
            "p",
            `${finding.severity.toUpperCase()} · ${finding.rule}: ${finding.detail}`,
            "finding",
          ),
        );
      replace("manual-result", ...nodes);
    }
    const rules = [];
    const rule = (label, value) => {
      const row = element("div");
      row.append(element("dt", label), element("dd", value));
      rules.push(row);
    };
    rule("Timeout", `${target.timeout_secs}s`);
    if (target.type === "http") {
      rule(
        "Expected HTTP status",
        target.expected_status ? String(target.expected_status) : "Any 2xx",
      );
      rule(
        "Response text assertion",
        target.body_assertion
          ? result?.ok
            ? "Verified in last manual test"
            : "Configured · not yet confirmed"
          : "Not configured",
      );
    }
    if (target.type === "flow")
      rule(
        "Configured flow",
        `${target.flow_steps} steps · stops on first failure`,
      );
    if (target.type === "heartbeat")
      rule("Test behavior", "Reads last heartbeat; does not send a ping");
    if (target.type === "security")
      rule("Test behavior", "Runs configured passive probes");
    replace("check-rules", ...rules);
  }

  function renderHistory(target) {
    const history = target.history;
    byId("sample-count").textContent = `${history.length} recorded`;
    const marks = [...history].reverse().map((row) => {
      const mark = element("li", undefined, row.ok ? "pass" : "fail");
      const description = `${date(row.checked_at)} · ${row.ok ? "PASS" : "FAIL"} · ${row.latency_ms} ms`;
      mark.title = description;
      mark.append(element("span", description));
      return mark;
    });
    replace("observation-strip", ...marks);
    byId("threshold-note").textContent = history.length
      ? `${target.failure_threshold} consecutive failures to open an incident; ${target.recovery_threshold} successes to recover. Threshold counters (capped): failures ${target.consecutive_fail}/${target.failure_threshold}; successes ${target.consecutive_ok}/${target.recovery_threshold}. History shows samples, not continuous uptime.`
      : "No scheduled observations yet. The first check runs shortly after the daemon starts.";
    replace(
      "scheduled-steps",
      ...(target.steps.length
        ? [stepsNode(target.steps, "Latest scheduled flow steps")]
        : []),
    );
    const rows = history.map((row) => {
      const tr = element("tr");
      const time = element("td");
      time.append(element("time", date(row.checked_at)));
      time.firstChild.dateTime = row.checked_at;
      const result = element("td");
      result.append(badge(row.ok ? "PASS" : "FAIL", row.ok ? "pass" : "fail"));
      tr.append(
        time,
        result,
        element("td", row.status_code === null ? "—" : String(row.status_code)),
        element("td", `${row.latency_ms} ms`),
        element(
          "td",
          [row.failed_step, row.error].filter(Boolean).join(" · ") ||
            "Configured checks passed",
        ),
      );
      return tr;
    });
    if (!rows.length) {
      const row = element("tr");
      const cell = element(
        "td",
        "Waiting for the first scheduled check.",
        "secondary",
      );
      cell.colSpan = 5;
      row.append(cell);
      rows.push(row);
    }
    replace("history-rows", ...rows);
    const incidents = target.incidents.map((incident) => {
      const row = element("div", undefined, "incident-row");
      row.append(
        badge(
          incident.resolved_at ? "RECOVERED" : "OPEN",
          incident.resolved_at ? "pass" : "fail",
        ),
        element(
          "span",
          `Incident ${incident.id} · ${date(incident.started_at)}`,
        ),
        element(
          "span",
          incident.resolved_at
            ? `Resolved ${date(incident.resolved_at)}`
            : "Awaiting the recovery threshold",
        ),
      );
      return row;
    });
    replace(
      "incidents",
      ...(incidents.length
        ? incidents
        : [
            element("p", "No incidents recorded for this target.", "secondary"),
          ]),
    );
  }

  function render() {
    byId("connection-dot").className =
      `connection-dot ${connected ? "connected" : "disconnected"}`;
    byId("connection-label").textContent = connected
      ? "Connected · refreshes every 2s"
      : "Disconnected · showing last received data";
    const canAdd = connected && Boolean(snapshot?.add_api_enabled);
    byId("add-api-toggle").disabled = !canAdd || saving;
    byId("add-api-toggle").setAttribute("aria-expanded", String(formOpen));
    byId("add-api-unavailable").hidden =
      !snapshot || Boolean(snapshot.add_api_enabled);
    byId("add-api-unavailable").textContent =
      snapshot?.add_api_unavailable ||
      "Adding APIs is unavailable for this monitor.";
    byId("add-api-panel").hidden = !formOpen;
    byId("add-api-save").disabled = !canAdd || saving;
    byId("add-api-save").textContent = saving
      ? "Saving…"
      : "Save & start monitoring";
    byId("add-api-cancel").disabled = saving;
    if (!snapshot) {
      if (!connected) {
        byId("loading-detail").querySelector("h2").textContent =
          "Waiting for the daemon";
        byId("loading-detail").querySelector("p").textContent =
          "Keep the monitor running. Use Refresh to reconnect.";
      }
      return;
    }
    if (!snapshot.targets.some((target) => target.name === selected))
      selected = snapshot.targets[0]?.name;
    renderTargets();
    byId("loading-detail").hidden = true;
    byId("empty-state").hidden = formOpen || snapshot.targets.length > 0;
    byId("detail-content").hidden = formOpen || snapshot.targets.length === 0;
    const target = snapshot.targets.find((target) => target.name === selected);
    if (!target) return;
    const busy = pending.has(target.name) || Boolean(target.activity);
    byId("run-check").disabled = !connected || busy;
    byId("run-check").classList.toggle("running", busy);
    byId("run-check").querySelector("span").textContent = busy
      ? "Checking…"
      : "Run check";
    byId("run-progress").hidden = !pending.has(target.name);
    byId("activity").textContent = pending.has(target.name)
      ? "Manual check running"
      : target.activity
        ? `${target.activity.source === "manual" ? "Manual" : "Scheduled"} check running`
        : "Waiting for next scheduled check";
    byId("activity").classList.toggle("running", busy);
    if (!connected) byId("activity").textContent = "Live activity unavailable";
    const error = actionErrors.get(target.name);
    byId("action-error").hidden = !error;
    byId("action-error").textContent = error || "";
    // Keep buttons, selected target controls, and scroll containers mounted during polling.
    const key = JSON.stringify(target);
    if (key === detailKey) return;
    detailKey = key;
    byId("target-name").textContent = target.name;
    byId("target-kind").textContent = target.type;
    byId("schedule-summary").textContent =
      `Scheduled every ${target.interval_secs}s · last observed ${date(target.last_checked_at)}`;
    replace("incident-state", badge(statusLabel(target), stateClass(target)));
    const latest = target.history[0];
    replace(
      "last-observation",
      latest
        ? badge(latest.ok ? "PASS" : "FAIL", latest.ok ? "pass" : "fail")
        : element("span", "Waiting"),
    );
    byId("last-latency").textContent = latest ? `${latest.latency_ms} ms` : "—";
    byId("pending-alerts").textContent = String(target.pending_alerts);
    renderManual(target);
    renderHistory(target);
  }

  function refresh(force = false) {
    if (refreshPromise) return refreshPromise;
    if (document.hidden && force !== true) return Promise.resolve();
    refreshPromise = loadSnapshot().finally(() => {
      refreshPromise = null;
    });
    return refreshPromise;
  }

  async function loadSnapshot() {
    clearTimeout(timer);
    byId("refresh").disabled = true;
    try {
      const response = await fetch("/api/dashboard", {
        cache: "no-store",
        signal: AbortSignal.timeout(5000),
      });
      if (!response.ok) throw new Error("snapshot unavailable");
      snapshot = await response.json();
      connected = true;
      byId("connection-error").hidden = true;
    } catch {
      connected = false;
      byId("connection-error").textContent =
        `Cannot reach the local monitor. Check that the daemon is running, then refresh.${snapshot ? ` Last snapshot: ${date(snapshot.server_time)}.` : ""}`;
      byId("connection-error").hidden = false;
    } finally {
      byId("refresh").disabled = false;
      render();
      if (!document.hidden) timer = setTimeout(refresh, 2000);
    }
  }

  const form = byId("add-api-form");
  const errorNode = byId("add-api-error");
  function clearFormErrors() {
    errorNode.hidden = true;
    for (const field of form.querySelectorAll("input, select")) {
      field.setCustomValidity("");
      field.removeAttribute("aria-invalid");
      const description = field
        .getAttribute("aria-describedby")
        ?.replace(/ ?add-api-error/g, "");
      if (description) field.setAttribute("aria-describedby", description);
      else field.removeAttribute("aria-describedby");
    }
  }
  form.addEventListener("input", clearFormErrors);
  byId("api-method").addEventListener("change", () => {
    const head = byId("api-method").value === "HEAD";
    byId("api-text").disabled = head;
    clearFormErrors();
  });
  byId("add-api-toggle").addEventListener("click", () => {
    formOpen = true;
    byId("add-api-notice").hidden = true;
    render();
    byId("api-name").focus();
  });
  byId("add-api-cancel").addEventListener("click", () => {
    formOpen = false;
    form.reset();
    byId("api-text").disabled = false;
    clearFormErrors();
    render();
    byId("add-api-toggle").focus();
  });
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    if (saving || !connected || !snapshot?.add_api_enabled) return;
    clearFormErrors();
    const data = new FormData(form);
    const timeout = byId("api-timeout");
    if (Number(data.get("timeout_secs")) >= Number(data.get("interval_secs")))
      timeout.setCustomValidity(
        "Timeout must be shorter than the check interval.",
      );
    if (!form.reportValidity()) return;
    const input = {
      name: String(data.get("name")).trim(),
      url: String(data.get("url")).trim(),
      method: data.get("method"),
      expected_status: data.get("expected_status")
        ? Number(data.get("expected_status"))
        : null,
      expected_body_contains: data.get("expected_body_contains") || null,
      interval_secs: Number(data.get("interval_secs")),
      timeout_secs: Number(data.get("timeout_secs")),
    };
    saving = true;
    render();
    for (const field of form.querySelectorAll("input, select"))
      field.disabled = true;
    try {
      const response = await fetch("/api/targets", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Look-Out-Action": "add-api",
        },
        body: JSON.stringify(input),
        signal: AbortSignal.timeout(20000),
      });
      const result = await response.json();
      if (!response.ok) {
        errorNode.textContent =
          result.error ||
          "The API could not be saved. Refresh before trying again.";
        errorNode.hidden = false;
        const field = form.elements.namedItem(result.field);
        if (
          field instanceof HTMLInputElement ||
          field instanceof HTMLSelectElement
        ) {
          field.setAttribute("aria-invalid", "true");
          field.setAttribute(
            "aria-describedby",
            `${field.getAttribute("aria-describedby") || ""} add-api-error`.trim(),
          );
          field.dataset.focusError = "true";
        }
        return;
      }
      formOpen = false;
      form.reset();
      byId("add-api-notice").textContent = `${result.name}: ${result.message}`;
      byId("add-api-notice").hidden = false;
      if (refreshPromise) await refreshPromise;
      await refresh(true);
      if (result.monitoring) selected = result.name;
      detailKey = "";
      render();
      const target = [...byId("target-list").children].find(
        (button) => button.dataset.target === selected,
      );
      (target || byId("add-api-toggle")).focus();
    } catch {
      errorNode.textContent =
        "The save result is uncertain. Refresh and check the target list before trying again. Your draft is still here.";
      errorNode.hidden = false;
    } finally {
      saving = false;
      for (const field of form.querySelectorAll("input, select"))
        field.disabled = false;
      byId("api-text").disabled = byId("api-method").value === "HEAD";
      render();
      const invalid = form.querySelector('[data-focus-error="true"]');
      if (invalid) {
        invalid.removeAttribute("data-focus-error");
        invalid.focus();
      }
    }
  });

  byId("run-check").addEventListener("click", async () => {
    const target = snapshot?.targets.find((item) => item.name === selected);
    if (!target || !connected || pending.has(target.name) || target.activity)
      return;
    const name = target.name;
    pending.add(name);
    actionErrors.delete(name);
    render();
    try {
      const response = await fetch(
        `/api/targets/${encodeURIComponent(name)}/check`,
        {
          method: "POST",
          headers: { "X-Look-Out-Action": "run-check" },
          signal: AbortSignal.timeout((target.timeout_secs + 5) * 1000),
        },
      );
      if (!response.ok) {
        if (response.status === 409)
          throw new Error(
            "A check is already running. Wait a moment, then try again.",
          );
        if (response.status === 404)
          throw new Error(
            "This target is no longer available. Refresh the dashboard.",
          );
        throw new Error(
          "The test could not complete. Refresh the dashboard, then try again.",
        );
      }
      const result = await response.json();
      const current = snapshot?.targets.find((item) => item.name === name);
      if (current) current.manual_result = result;
    } catch (error) {
      actionErrors.set(
        name,
        error.name === "TimeoutError"
          ? "The dashboard stopped waiting for this test. Its result is uncertain; refresh before running another check."
          : error.name === "TypeError"
            ? "Connection lost. The test result is uncertain; reconnect before retrying."
            : error.message,
      );
    } finally {
      pending.delete(name);
      detailKey = "";
      render();
      await refresh();
    }
  });
  byId("refresh").addEventListener("click", () => refresh());
  document.addEventListener("visibilitychange", () => {
    clearTimeout(timer);
    if (!document.hidden) refresh();
  });
  refresh();
})();
