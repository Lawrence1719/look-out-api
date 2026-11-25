"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { ArrowUpRight, BookOpen, Code2, Menu, Radio, X } from "lucide-react";

const links = [
  { label: "Overview", id: "overview" },
  { label: "The checks", id: "checks" },
  { label: "Reliable by design", id: "reliability" },
  { label: "Get started", id: "get-started" },
  { label: "What’s next", id: "roadmap" },
];
export function Sidebar() {
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState("overview");
  const pathname = usePathname();
  const isDocs = pathname.includes("/docs");
  const repository =
    process.env.NEXT_PUBLIC_REPOSITORY_URL ||
    "https://github.com/Lawrence1719/look-out-api";
  useEffect(() => {
    if (isDocs) return;
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries)
          if (entry.isIntersecting) setActive(entry.target.id);
      },
      { rootMargin: "-10% 0px -65% 0px" },
    );
    for (const link of links) {
      const element = document.getElementById(link.id);
      if (element) observer.observe(element);
    }
    return () => observer.disconnect();
  }, [isDocs]);
  useEffect(() => {
    if (!open) return;
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  }, [open]);
  return (
    <aside className={`sidebar ${open ? "is-open" : ""}`}>
      <div className="brand-row">
        <Link
          className="brand"
          href="/"
          aria-label="Look Out API home"
          onClick={() => setOpen(false)}
        >
          <span className="brand-mark">
            <Radio size={25} strokeWidth={1.6} />
          </span>
          <span>
            look out<span className="brand-api"> / api</span>
          </span>
        </Link>
        <button
          className="menu-toggle icon-button"
          type="button"
          aria-label={open ? "Close navigation" : "Open navigation"}
          aria-expanded={open}
          aria-controls="site-navigation"
          onClick={() => setOpen(!open)}
        >
          {open ? <X size={22} /> : <Menu size={22} />}
        </button>
      </div>
      <div id="site-navigation" className="sidebar-content">
        <nav aria-label="Main navigation" className="primary-nav">
          {links.map((link) => (
            <Link
              key={link.id}
              href={`/#${link.id}`}
              onClick={() => {
                setActive(link.id);
                setOpen(false);
              }}
              className={!isDocs && active === link.id ? "active" : ""}
              aria-current={
                !isDocs && active === link.id ? "location" : undefined
              }
            >
              <span className="nav-dot" />
              {link.label}
            </Link>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <Link
            href="/docs/"
            className={`resource-link ${isDocs ? "selected" : ""}`}
            onClick={() => setOpen(false)}
          >
            <BookOpen size={16} /> Documentation
            <ArrowUpRight size={14} />
          </Link>
          <a
            className="resource-link"
            href={repository}
            onClick={() => setOpen(false)}
            target="_blank"
            rel="noopener noreferrer"
          >
            <Code2 size={16} /> Source code
            <ArrowUpRight size={14} />
          </a>
          <div className="sidebar-note">
            <span className="small-signal" />
            <span>
              Independent by design.
              <br />
              Built in Rust.
            </span>
          </div>
        </div>
      </div>
    </aside>
  );
}
