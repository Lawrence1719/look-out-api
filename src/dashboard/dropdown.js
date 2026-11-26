"use strict";

// Shared single-select enhancement. The original control owns the submitted value.
(() => {
  const controls = new Map();
  let openControl = null;
  let sequence = 0;
  const icon = (path) => {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    for (const [key, value] of Object.entries({
      width: "16",
      height: "16",
      viewBox: "0 0 24 24",
      fill: "none",
      stroke: "currentColor",
      "stroke-width": "2",
      "stroke-linecap": "round",
      "stroke-linejoin": "round",
      "aria-hidden": "true",
      focusable: "false",
    }))
      svg.setAttribute(key, value);
    const line = document.createElementNS(svg.namespaceURI, "path");
    line.setAttribute("d", path);
    svg.append(line);
    return svg;
  };
  class Dropdown {
    constructor(select) {
      this.select = select;
      this.active = -1;
      this.search = "";
      this.lastKey = 0;
      select.id ||= `dropdown-${++sequence}`;
      this.trigger = document.createElement("button");
      this.trigger.type = "button";
      this.trigger.id = `${select.id}-trigger`;
      this.trigger.className = "dropdown-trigger";
      this.trigger.setAttribute("role", "combobox");
      this.trigger.setAttribute("aria-haspopup", "listbox");
      this.trigger.setAttribute("aria-expanded", "false");
      this.value = document.createElement("span");
      this.value.className = "dropdown-value";
      this.trigger.append(this.value, icon("m6 9 6 6 6-6"));
      this.popup = document.createElement("div");
      this.popup.id = `${select.id}-options`;
      this.popup.className = "dropdown-options";
      this.popup.setAttribute("role", "listbox");
      this.popup.hidden = true;
      this.trigger.setAttribute("aria-controls", this.popup.id);
      const labels = [...select.labels];
      const labelIds = labels
        .map((label, index) => {
          label.id ||= `${select.id}-label-${index}`;
          label.htmlFor = this.trigger.id;
          return label.id;
        })
        .join(" ");
      if (labelIds || select.getAttribute("aria-labelledby")) {
        const ids = labelIds || select.getAttribute("aria-labelledby");
        this.trigger.setAttribute("aria-labelledby", ids);
        this.popup.setAttribute("aria-labelledby", ids);
      } else {
        const name =
          select.getAttribute("aria-label") ||
          select.name ||
          "Choose an option";
        this.trigger.setAttribute("aria-label", name);
        this.popup.setAttribute("aria-label", name);
      }
      this.error = document.createElement("p");
      this.error.id = `${select.id}-validation`;
      this.error.className = "dropdown-error";
      this.error.setAttribute("role", "alert");
      this.error.hidden = true;
      select.after(this.trigger, this.error);
      document.body.append(this.popup);
      select.hidden = true;
      select.tabIndex = -1;
      select.dataset.dropdownTrigger = this.trigger.id;
      this.trigger.addEventListener("click", () =>
        this.isOpen ? this.close() : this.open(),
      );
      this.trigger.addEventListener("keydown", (event) => this.keydown(event));
      this.trigger.addEventListener("blur", () => this.close());
      this.popup.addEventListener("pointerdown", (event) =>
        event.preventDefault(),
      );
      select.addEventListener("change", () => this.sync());
      select.addEventListener("invalid", (event) => {
        event.preventDefault();
        this.error.textContent = select.validationMessage;
        this.error.hidden = false;
        this.sync();
        this.trigger.focus();
      });
      select.form?.addEventListener("reset", () =>
        queueMicrotask(() => {
          this.close();
          this.sync();
        }),
      );
      new MutationObserver(() => this.sync()).observe(select, {
        attributes: true,
        childList: true,
        subtree: true,
        characterData: true,
      });
      const fieldset = select.closest("fieldset");
      if (fieldset)
        new MutationObserver(() => this.sync()).observe(fieldset, {
          attributes: true,
          attributeFilter: ["disabled"],
        });
      this.sync();
    }
    get isOpen() {
      return openControl === this;
    }
    get enabled() {
      return [...this.select.options]
        .map((option, index) =>
          !option.disabled && !option.parentElement.disabled ? index : -1,
        )
        .filter((index) => index >= 0);
    }
    sync() {
      this.trigger.disabled =
        this.select.matches(":disabled") || !this.enabled.length;
      for (const attribute of [
        "aria-describedby",
        "aria-invalid",
        "aria-required",
      ]) {
        if (this.select.hasAttribute(attribute))
          this.trigger.setAttribute(
            attribute,
            this.select.getAttribute(attribute),
          );
        else this.trigger.removeAttribute(attribute);
      }
      if (this.select.required)
        this.trigger.setAttribute("aria-required", "true");
      if (this.select.validity.valid) this.error.hidden = true;
      if (!this.error.hidden) {
        this.trigger.setAttribute("aria-invalid", "true");
        this.trigger.setAttribute(
          "aria-describedby",
          `${this.select.getAttribute("aria-describedby") || ""} ${this.error.id}`.trim(),
        );
      }
      this.value.textContent =
        this.select.selectedOptions[0]?.label || "Choose an option";
      // Rebuild only when option content/state changes, never on daemon polling.
      const key = JSON.stringify(
        [...this.select.options].map((option) => [
          option.label,
          option.disabled || Boolean(option.parentElement.disabled),
          option.selected,
        ]),
      );
      if (key !== this.optionsKey) {
        this.optionsKey = key;
        this.popup.replaceChildren(
          ...[...this.select.options].map((option, index) => {
            const row = document.createElement("div");
            row.id = `${this.select.id}-option-${index}`;
            row.className = "dropdown-option";
            row.setAttribute("role", "option");
            row.setAttribute("aria-selected", String(option.selected));
            row.setAttribute(
              "aria-disabled",
              String(!this.enabled.includes(index)),
            );
            const label = document.createElement("span");
            label.textContent = option.label;
            row.append(label, icon("m20 6-11 11-5-5"));
            row.addEventListener("pointermove", () => {
              if (this.enabled.includes(index)) this.highlight(index);
            });
            row.addEventListener("click", () => {
              if (this.enabled.includes(index)) this.commit(index);
            });
            return row;
          }),
        );
      }
      if (this.trigger.disabled || this.trigger.closest("[hidden]"))
        this.close();
      else if (this.isOpen) {
        this.highlight(
          this.enabled.includes(this.active) ? this.active : this.enabled[0],
        );
        this.position();
      }
    }
    highlight(index) {
      this.active = index;
      for (const [i, row] of [...this.popup.children].entries())
        row.classList.toggle("is-active", i === index);
      const row = this.popup.children[index];
      if (row && this.isOpen) {
        this.trigger.setAttribute("aria-activedescendant", row.id);
        // Scroll only the menu; scrollIntoView can move the page on mobile.
        if (row.offsetTop < this.popup.scrollTop)
          this.popup.scrollTop = row.offsetTop;
        else if (
          row.offsetTop + row.offsetHeight >
          this.popup.scrollTop + this.popup.clientHeight
        )
          this.popup.scrollTop =
            row.offsetTop + row.offsetHeight - this.popup.clientHeight;
      }
    }
    open() {
      if (this.trigger.disabled) return;
      openControl?.close();
      openControl = this;
      this.search = "";
      this.popup.hidden = false;
      this.trigger.setAttribute("aria-expanded", "true");
      this.position();
      this.highlight(
        this.enabled.includes(this.select.selectedIndex)
          ? this.select.selectedIndex
          : this.enabled[0],
      );
    }
    close() {
      if (!this.isOpen) return;
      this.popup.hidden = true;
      this.trigger.setAttribute("aria-expanded", "false");
      this.trigger.removeAttribute("aria-activedescendant");
      openControl = null;
      this.search = "";
    }
    commit(index = this.active) {
      if (!this.enabled.includes(index)) return;
      const changed = this.select.selectedIndex !== index;
      this.select.selectedIndex = index;
      this.close();
      this.sync();
      if (changed) {
        this.select.dispatchEvent(new Event("input", { bubbles: true }));
        this.select.dispatchEvent(new Event("change", { bubbles: true }));
      }
    }
    position() {
      if (!this.isOpen) return;
      const bounds = this.trigger.getBoundingClientRect();
      if (!bounds.width || bounds.bottom < 0 || bounds.top > innerHeight) {
        this.close();
        return;
      }
      const viewport = window.visualViewport;
      const leftEdge = viewport?.offsetLeft || 0;
      const topEdge = viewport?.offsetTop || 0;
      const rightEdge = leftEdge + (viewport?.width || innerWidth);
      const bottomEdge = topEdge + (viewport?.height || innerHeight);
      const below = bottomEdge - bounds.bottom - 12;
      const above = bounds.top - topEdge - 12;
      const upward =
        below < Math.min(240, this.popup.scrollHeight) && above > below;
      const available = Math.max(0, upward ? above : below);
      this.popup.style.width = `${Math.min(bounds.width, rightEdge - leftEdge - 16)}px`;
      this.popup.style.maxHeight = `${Math.min(264, available)}px`;
      this.popup.style.left = `${Math.max(leftEdge + 8, Math.min(bounds.left, rightEdge - this.popup.offsetWidth - 8))}px`;
      this.popup.style.top = `${upward ? bounds.top - this.popup.offsetHeight - 6 : bounds.bottom + 6}px`;
    }
    keydown(event) {
      if (
        event.ctrlKey ||
        event.metaKey ||
        (event.altKey && !["ArrowDown", "ArrowUp"].includes(event.key))
      )
        return;
      if (event.key === "Escape" && this.isOpen) {
        event.preventDefault();
        this.close();
        return;
      }
      if (event.key === "Tab") {
        if (this.isOpen) this.commit();
        return;
      }
      if (["Enter", " "].includes(event.key)) {
        event.preventDefault();
        if (this.isOpen) this.commit();
        else this.open();
        return;
      }
      if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
        event.preventDefault();
        const wasOpen = this.isOpen;
        if (!wasOpen) this.open();
        const enabled = this.enabled;
        const current = enabled.indexOf(this.active);
        const index =
          event.key === "Home"
            ? enabled[0]
            : event.key === "End"
              ? enabled.at(-1)
              : !wasOpen
                ? this.active
                : enabled[
                    Math.max(
                      0,
                      Math.min(
                        enabled.length - 1,
                        current + (event.key === "ArrowDown" ? 1 : -1),
                      ),
                    )
                  ];
        this.highlight(index);
      } else if (event.key.length === 1 && !event.altKey) {
        event.preventDefault();
        if (!this.isOpen) this.open();
        const now = Date.now();
        this.search =
          now - this.lastKey > 700 ? event.key : this.search + event.key;
        this.lastKey = now;
        const query = [...this.search].every(
          (char) => char.toLowerCase() === event.key.toLowerCase(),
        )
          ? event.key
          : this.search;
        const enabled = this.enabled;
        const offset = enabled.indexOf(this.active);
        const ordered = [
          ...enabled.slice(offset + 1),
          ...enabled.slice(0, offset + 1),
        ];
        const match = ordered.find((index) =>
          this.select.options[index].label
            .toLocaleLowerCase()
            .startsWith(query.toLocaleLowerCase()),
        );
        if (match !== undefined) this.highlight(match);
      }
    }
  }
  function enhance(root = document) {
    for (const select of root.querySelectorAll("select")) {
      if (!select.multiple && select.size <= 1 && !controls.has(select))
        controls.set(select, new Dropdown(select));
    }
  }
  document.addEventListener("pointerdown", (event) => {
    if (
      openControl &&
      !openControl.trigger.contains(event.target) &&
      !openControl.popup.contains(event.target)
    )
      openControl.close();
  });
  document.addEventListener(
    "scroll",
    (event) => {
      if (openControl && !openControl.popup.contains(event.target))
        openControl.position();
    },
    true,
  );
  window.addEventListener("resize", () => openControl?.position());
  window.visualViewport?.addEventListener("resize", () =>
    openControl?.position(),
  );
  window.visualViewport?.addEventListener("scroll", () =>
    openControl?.position(),
  );
  window.addEventListener("blur", () => openControl?.close());
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) openControl?.close();
  });
  window.LookOutDropdowns = {
    enhance,
    close: () => openControl?.close(),
    focus: (select) => (controls.get(select)?.trigger || select).focus(),
  };
  enhance();
})();
