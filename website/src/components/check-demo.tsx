"use client";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import {
  Bell,
  Check,
  ChevronRight,
  Pause,
  Play,
  RotateCcw,
  X,
} from "lucide-react";
import { useEffect, useState } from "react";

const observations = [
  {
    code: 200,
    time: "+00:00",
    state: "UP",
    count: "All clear",
    note: "The response matches. No notification needed.",
    event: "No notification",
    kind: "quiet",
  },
  {
    code: 503,
    time: "+00:30",
    state: "UP",
    count: "Failure 1 of 3",
    note: "One failed check. Wait for a consistent signal.",
    event: "No notification",
    kind: "quiet",
  },
  {
    code: 503,
    time: "+01:00",
    state: "UP",
    count: "Failure 2 of 3",
    note: "A second failure. The threshold still holds.",
    event: "No notification",
    kind: "quiet",
  },
  {
    code: 503,
    time: "+01:30",
    state: "DOWN",
    count: "Failure 3 of 3",
    note: "Threshold reached. Record the incident and queue its alert.",
    event: "DOWN · orders-api",
    kind: "down",
  },
  {
    code: 200,
    time: "+02:00",
    state: "DOWN",
    count: "Recovery 1 of 2",
    note: "A healthy response. Wait for recovery confirmation.",
    event: "Waiting for recovery",
    kind: "quiet",
  },
  {
    code: 200,
    time: "+02:30",
    state: "UP",
    count: "Recovery 2 of 2",
    note: "Recovery confirmed. Close the incident and queue its update.",
    event: "RECOVERED · orders-api",
    kind: "recovered",
  },
];
export function CheckDemo() {
  const reduced = useReducedMotion();
  const [index, setIndex] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [manual, setManual] = useState(false);
  useEffect(() => {
    if (!reduced) setPlaying(true);
    else setPlaying(false);
  }, [reduced]);
  useEffect(() => {
    if (!playing) return;
    const timer = setInterval(
      () =>
        setIndex((current) => {
          if (current >= observations.length - 1) {
            setPlaying(false);
            return current;
          }
          return current + 1;
        }),
      2400,
    );
    return () => clearInterval(timer);
  }, [playing]);
  const observation = observations[index];
  function toggle() {
    setManual(true);
    if (index === observations.length - 1 && !playing) {
      setIndex(0);
      setPlaying(true);
    } else setPlaying(!playing);
  }
  return (
    <div className="demo" aria-label="Illustrative incident sequence">
      <div className="demo-heading">
        <div className="demo-target">
          <span className="target-symbol">
            <span />
          </span>
          <span>
            orders-api<span className="demo-url"> / health</span>
          </span>
          <span className="sample-label">Illustrative sequence</span>
        </div>
        <button
          className="demo-play"
          type="button"
          onClick={toggle}
          aria-label={
            playing
              ? "Pause demonstration"
              : index === observations.length - 1
                ? "Replay demonstration"
                : "Play demonstration"
          }
        >
          {playing ? (
            <Pause size={13} />
          ) : index === observations.length - 1 ? (
            <RotateCcw size={13} />
          ) : (
            <Play size={13} />
          )}
          <span>
            {playing
              ? "Pause"
              : index === observations.length - 1
                ? "Replay"
                : "Play"}
          </span>
        </button>
      </div>
      <div
        className="observation-strip"
        role="group"
        aria-label="Choose an observation"
      >
        {observations.map((item, i) => (
          <button
            key={item.time}
            type="button"
            aria-label={`Observation ${i + 1}: HTTP ${item.code}`}
            aria-pressed={i === index}
            onClick={() => {
              setIndex(i);
              setPlaying(false);
              setManual(true);
            }}
            className={`observation ${i === index ? "current" : ""} ${i <= index ? "observed" : "future"}`}
          >
            <span className="observation-time">{item.time}</span>
            <span
              className={`observation-code ${item.code === 200 ? "pass" : "fail"}`}
            >
              {item.code === 200 ? <Check size={15} /> : <X size={15} />}
              {item.code}
            </span>
            <span className="observation-rule">
              {i === index && (
                <motion.span
                  layoutId="observation-marker"
                  transition={{
                    duration: reduced ? 0 : 0.35,
                    ease: [0.16, 1, 0.3, 1],
                  }}
                />
              )}
            </span>
          </button>
        ))}
      </div>
      <div className="demo-result">
        <div className="demo-state">
          <span
            className={`state-dot ${observation.state === "DOWN" ? "state-down" : ""}`}
          />
          <strong>{observation.state}</strong>
          <ChevronRight size={14} />
          <span>{observation.count}</span>
        </div>
        <div className={`demo-notification ${observation.kind}`}>
          <Bell size={14} />
          <AnimatePresence initial={false} mode="wait">
            <motion.span
              key={observation.event}
              initial={{ opacity: 0, y: reduced ? 0 : 5 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: reduced ? 0 : 0.15 }}
            >
              {observation.event}
            </motion.span>
          </AnimatePresence>
        </div>
      </div>
      <p className="demo-explanation" aria-live={manual ? "polite" : "off"}>
        {observation.note}
      </p>
    </div>
  );
}
