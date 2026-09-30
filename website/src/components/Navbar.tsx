import { useEffect, useRef, useState } from "react";
import { APP_VERSION, SOURCE_URL } from "../site.ts";
import Logo from "./Logo.tsx";

const LINKS = [
  { href: "#features", label: "Features" },
  { href: "#download", label: "Download" },
  { href: "#install", label: "Install" },
  { href: "#formats", label: "Formats" },
  { href: "#faq", label: "FAQ" },
];

export default function Navbar() {
  const [open, setOpen] = useState(false);
  const toggle = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        toggle.current?.focus();
      }
    };
    window.addEventListener("keydown", close);
    return () => window.removeEventListener("keydown", close);
  }, [open]);

  return (
    <header className="fixed inset-x-0 top-0 z-50 border-b border-white/6 bg-zinc-950/85 backdrop-blur-xl">
      <nav aria-label="Main navigation" className="mx-auto flex h-20 max-w-6xl items-center justify-between px-5 sm:px-6">
        <a href="#top" onClick={() => setOpen(false)} className="flex items-center gap-2.5">
          <Logo size={30} />
          <span className="text-base font-semibold tracking-tight text-zinc-50">
            MP3 Tagger
          </span>
          <span className="rounded-md border border-white/10 bg-white/5 px-2 py-0.5 text-[10px] font-medium text-zinc-400">
            v{APP_VERSION}
          </span>
        </a>

        <div className="hidden items-center gap-1 md:flex">
          {LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="rounded-md px-3 py-2 text-sm text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100"
            >
              {link.label}
            </a>
          ))}
          <a
            href={SOURCE_URL}
            target="_blank"
            rel="noreferrer"
            className="ml-3 rounded-lg border border-white/15 bg-white/5 px-4 py-2 text-sm font-medium text-white transition hover:bg-white/10"
          >
            GitHub ↗
          </a>
        </div>

        <button
          ref={toggle}
          type="button"
          className="rounded-md p-2 text-zinc-300 hover:bg-zinc-800 md:hidden"
          aria-label={open ? "Close menu" : "Open menu"}
          aria-expanded={open}
          aria-controls="mobile-navigation"
          onClick={() => setOpen((value) => !value)}
        >
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
            {open ? (
              <path d="M6 6l12 12M18 6L6 18" />
            ) : (
              <path d="M4 7h16M4 12h16M4 17h16" />
            )}
          </svg>
        </button>
      </nav>

      {open && (
        <div id="mobile-navigation" className="border-t border-white/6 bg-zinc-950 px-5 pt-2 pb-5 md:hidden">
          {LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              onClick={() => setOpen(false)}
              className="block rounded-md px-3 py-2.5 text-sm text-zinc-300 hover:bg-zinc-800"
            >
              {link.label}
            </a>
          ))}
          <a
            href={SOURCE_URL}
            target="_blank"
            rel="noreferrer"
            className="mt-2 block rounded-lg bg-indigo-600 px-4 py-2.5 text-center text-sm font-semibold text-white"
          >
            GitHub
          </a>
        </div>
      )}
    </header>
  );
}
