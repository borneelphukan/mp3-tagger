import Section from "./Section.tsx";
import Icon, { type IconName } from "./Icon.tsx";

const FEATURES: { icon: IconName; title: string; text: string }[] = [
  {
    icon: "grid",
    title: "List & grid library views",
    text: "Browse your songs in a compact list or a full metadata table. Double-click any title to open it.",
  },
  {
    icon: "edit",
    title: "Detail editor with artwork",
    text: "Edit every field with a live album-artwork preview — view, add and remove embedded cover art.",
  },
  {
    icon: "sparkle",
    title: "LLM-powered auto-tagging",
    text: "Point it at any OpenAI-compatible API and Fill Data completes title, artist, album and more.",
  },
  {
    icon: "layers",
    title: "Bring your collection",
    text: "Add multiple audio files in one go. Your library is ready to browse, and your preferences stay with you.",
  },
  {
    icon: "music",
    title: "34+ metadata fields",
    text: "ID3v2, Vorbis Comments and APE tags — from BPM and ISRC to lyrics, composers and producers.",
  },
  {
    icon: "monitor",
    title: "Native & cross-platform",
    text: "A fast Rust binary with a native GUI on Ubuntu and Windows. No Electron, no browser required.",
  },
];

export default function Features() {
  return (
    <Section
      id="features"
      eyebrow="Features"
      title="A little order. A lot more music."
      intro="From the album cover to the smallest credit, keep the details that make your collection yours."
    >
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {FEATURES.map((feature) => (
          <article
            key={feature.title}
            className="surface-card group p-7 transition duration-300 hover:-translate-y-1 hover:border-indigo-300/25"
          >
            <div
              className="flex h-11 w-11 items-center justify-center rounded-xl border border-indigo-400/15 bg-indigo-400/10 text-indigo-300 transition group-hover:bg-indigo-400/15"
              aria-hidden="true"
            >
              <Icon name={feature.icon} />
            </div>
            <h3 className="mt-6 text-base font-medium text-white">{feature.title}</h3>
            <p className="mt-3 text-sm leading-7 text-zinc-400">{feature.text}</p>
          </article>
        ))}
      </div>
    </Section>
  );
}
