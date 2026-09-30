import { APP_VERSION, SETUP_EXE, releaseAssetUrl } from "../site.ts";
import AppMock from "./AppMock.tsx";
import Icon from "./Icon.tsx";

export default function Hero() {
  return (
    <section className="hero-glow relative overflow-hidden px-5 pt-36 pb-12 sm:px-6 sm:pt-44" id="top">
      <div className="enter-animation relative mx-auto max-w-4xl text-center">
        <a href="#download" className="inline-flex items-center gap-3 rounded-full border border-indigo-300/15 bg-indigo-400/5 px-4 py-2 text-xs text-zinc-300 transition hover:border-indigo-300/40">
          <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
          Meet MP3 Tagger <span className="text-indigo-300">v{APP_VERSION}</span>
          <Icon name="arrow" className="h-3.5 w-3.5" />
        </a>
        <h1 className="mx-auto mt-7 max-w-3xl text-5xl leading-[1.08] font-semibold tracking-[-0.055em] text-white sm:text-6xl lg:text-7xl">
          Your music. <br />
          <span className="bg-linear-to-r from-indigo-200 via-violet-300 to-indigo-400 bg-clip-text text-transparent">Beautifully organized.</span>
        </h1>
        <p className="mx-auto mt-6 max-w-xl text-base leading-8 text-zinc-400 sm:text-lg">
          Give every track the details it deserves. Edit tags, add artwork,
          and fill in the gaps with AI — in one native desktop app.
        </p>
        <div className="mt-8 flex flex-col justify-center gap-3 sm:flex-row">
          <a href={releaseAssetUrl(SETUP_EXE)} className="button-primary">
            <Icon name="download" /> Download for Windows
          </a>
          <a href="#install" className="button-secondary">
            <Icon name="terminal" /> Install on Ubuntu
          </a>
        </div>
        <p className="mt-5 text-xs tracking-wide text-zinc-400">Free &amp; open source <span className="px-2 text-zinc-600">/</span> Built with Rust <span className="px-2 text-zinc-600">/</span> Yours to keep</p>
      </div>

      <figure className="relative mx-auto mt-14 max-w-5xl sm:mt-20">
        <div className="preview-glow rounded-2xl border border-white/10 bg-white/3 p-1.5 sm:rounded-3xl sm:p-3">
          <AppMock />
        </div>
        <figcaption className="mt-5 text-center text-xs text-zinc-500">A closer look at your collection · Illustrative interface preview</figcaption>
      </figure>

      <dl className="mx-auto mt-12 grid max-w-3xl grid-cols-3 divide-x divide-white/10 py-6 text-center sm:mt-16">
        {[["7", "audio formats"], ["34+", "metadata fields"], ["2", "desktop platforms"]].map(([value, label]) => (
          <div key={label} className="flex flex-col-reverse gap-2 px-2">
            <dt className="text-xs text-zinc-400 sm:text-sm">{label}</dt>
            <dd className="text-2xl font-medium tracking-tight text-zinc-100 sm:text-3xl">{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
