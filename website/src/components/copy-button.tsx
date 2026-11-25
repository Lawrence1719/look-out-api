"use client";
import { Check, Copy } from "lucide-react";
import { useEffect, useRef, useState } from "react";

export function CopyButton({
  text,
  label = "Copy code",
}: {
  text: string;
  label?: string;
}) {
  const [state, setState] = useState<"idle" | "copied" | "error">("idle");
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );
  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setState("copied");
    } catch {
      setState("error");
    }
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => setState("idle"), 2500);
  }
  return (
    <button
      type="button"
      className={`copy-button ${state === "error" ? "copy-error" : ""}`}
      onClick={copy}
      aria-label={state === "copied" ? "Copied to clipboard" : label}
    >
      <span aria-hidden="true">
        {state === "copied" ? <Check size={15} /> : <Copy size={15} />}
      </span>
      <span aria-live="polite">
        {state === "copied"
          ? "Copied"
          : state === "error"
            ? "Select to copy"
            : "Copy"}
      </span>
    </button>
  );
}
