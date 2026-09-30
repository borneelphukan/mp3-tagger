import { APT_POOL_URL, ISSUES_URL, RELEASES_URL, SOURCE_URL } from "../site.ts";
import Logo from "./Logo.tsx";

export default function Footer() {
  const year = new Date().getFullYear();
  return (
    <footer className="border-t border-white/6">
      <div className="mx-auto max-w-6xl px-5 pt-14 sm:px-6">
        <div className="hero-glow flex flex-col items-start justify-between gap-6 rounded-2xl border border-indigo-300/10 bg-indigo-400/5 p-7 sm:flex-row sm:items-center sm:p-10">
          <div><p className="text-2xl font-medium tracking-tight text-white">Make room for a better library.</p><p className="mt-2 text-sm text-zinc-400">All the music you love. All the details in place.</p></div>
          <a href="#download" className="button-primary shrink-0">Get MP3 Tagger <span aria-hidden="true">↗</span></a>
        </div>
      </div>
      <div className="mx-auto grid max-w-6xl gap-10 px-5 py-14 sm:px-6 md:grid-cols-[2fr_1fr_1fr]">
        <div>
          <div className="flex items-center gap-2.5">
            <Logo size={26} />
            <span className="text-sm font-semibold text-zinc-100">MP3 Tagger</span>
          </div>
          <p className="mt-3 max-w-xs text-sm leading-relaxed text-zinc-500">
            A free, native audio tag editor for Windows and Ubuntu — with
            LLM-powered metadata fetching.
          </p>
          <p className="mt-5 text-xs text-zinc-500">MIT License · © {year} borneelphukan</p>
        </div>
        <nav aria-label="Project">
          <h3 className="text-xs font-semibold tracking-widest text-zinc-500 uppercase">
            Project
          </h3>
          <ul className="mt-3 space-y-2 text-sm">
            <li><a className="text-zinc-400 hover:text-zinc-100" href={SOURCE_URL} target="_blank" rel="noreferrer">Source code</a></li>
            <li><a className="text-zinc-400 hover:text-zinc-100" href={RELEASES_URL} target="_blank" rel="noreferrer">Releases</a></li>
            <li><a className="text-zinc-400 hover:text-zinc-100" href={ISSUES_URL} target="_blank" rel="noreferrer">Report an issue</a></li>
          </ul>
        </nav>
        <nav aria-label="Install">
          <h3 className="text-xs font-semibold tracking-widest text-zinc-500 uppercase">
            Install
          </h3>
          <ul className="mt-3 space-y-2 text-sm">
            <li><a className="text-zinc-400 hover:text-zinc-100" href="#download">Windows installer</a></li>
            <li><a className="text-zinc-400 hover:text-zinc-100" href="#install">Ubuntu apt repository</a></li>
            <li><a className="text-zinc-400 hover:text-zinc-100" href={APT_POOL_URL} target="_blank" rel="noreferrer">Browse the .deb pool</a></li>
          </ul>
        </nav>
      </div>
    </footer>
  );
}
