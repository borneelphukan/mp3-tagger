import { APP_VERSION, DEB_FILE, PORTABLE_ZIP, RELEASES_URL, SETUP_EXE, releaseAssetUrl } from "../site.ts";
import Icon from "./Icon.tsx";
import Section from "./Section.tsx";

export default function Download() {
  return (
    <Section id="download" eyebrow="Make it yours" title="Your next library upgrade." intro={`MP3 Tagger v${APP_VERSION}. Free to download, easy to get started. Choose your desktop and make yourself at home.`}>
      <div className="grid gap-5 md:grid-cols-2">
        <article className="surface-card relative flex flex-col p-6 sm:p-8">
          <div className="mb-7 flex items-center justify-between">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl border border-white/10 bg-white/5 text-indigo-200"><Icon name="windows" className="h-6 w-6" /></div>
            <span className="rounded-full border border-white/10 px-3 py-1 text-xs text-zinc-400">64-bit</span>
          </div>
          <h3 className="text-2xl font-medium tracking-tight text-white">Windows</h3>
          <p className="mt-2 text-sm text-zinc-500">Windows 10 &amp; 11</p>
          <p className="mt-5 text-sm leading-7 text-zinc-400">A familiar installer, a Start Menu shortcut, and your whole collection at your fingertips.</p>
          <a href={releaseAssetUrl(SETUP_EXE)} className="button-primary mt-8"><Icon name="download" /> Download installer <span className="ml-auto text-xs text-indigo-100">.exe</span></a>
          <a href={releaseAssetUrl(PORTABLE_ZIP)} className="mt-4 self-center py-1 text-sm text-zinc-400 underline-offset-4 hover:text-white hover:underline">Prefer no installation? Get the portable .zip ↗</a>
        </article>
        <article className="surface-card flex flex-col p-6 sm:p-8">
          <div className="mb-7 flex items-center justify-between">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl border border-white/10 bg-white/5 text-indigo-200"><Icon name="terminal" className="h-6 w-6" /></div>
            <span className="rounded-full border border-white/10 px-3 py-1 text-xs text-zinc-400">amd64</span>
          </div>
          <h3 className="text-2xl font-medium tracking-tight text-white">Ubuntu / Debian</h3>
          <p className="mt-2 text-sm text-zinc-500">Built on Ubuntu 24.04</p>
          <p className="mt-5 text-sm leading-7 text-zinc-400">Add the apt repository once. Install and keep up to date alongside the rest of your system.</p>
          <a href="#install" className="button-secondary mt-8"><Icon name="terminal" /> Install with apt <Icon name="arrow" className="ml-auto h-4 w-4" /></a>
          <a href={releaseAssetUrl(DEB_FILE)} className="mt-4 self-center py-1 text-sm text-zinc-400 underline-offset-4 hover:text-white hover:underline">Or download the standalone .deb ↗</a>
        </article>
      </div>
      <p className="mt-7 text-center text-xs leading-6 text-zinc-500">Open source. MIT licensed. <a href={RELEASES_URL} className="ml-1 text-zinc-400 underline underline-offset-4 hover:text-white">Browse all releases ↗</a></p>
    </Section>
  );
}
