import Section from "./Section.tsx";

const ITEMS: [string, string][] = [
  [
    "Is MP3 Tagger really free?",
    "Yes — it is MIT-licensed open source. No accounts, no telemetry, no upsells. If you use LLM auto-tagging you only pay whatever your own API provider charges.",
  ],
  [
    "Which LLM providers work?",
    "Any OpenAI-compatible API. Enter a base URL, API key and model name under AI → LLM Settings — that covers OpenAI, Anthropic and Anthropic-compatible endpoints, and local servers like Ollama.",
  ],
  [
    "Where are my settings stored?",
    "Locally, in a single JSON file: ~/.config/mp3-tagger/settings.json on Linux and %APPDATA%\\mp3-tagger\\settings.json on Windows. Delete it to reset everything.",
  ],
  [
    "Does it work offline?",
    "Browsing and editing tags works fully offline. Only the Fill Data auto-tagging needs network access to reach your LLM provider.",
  ],
  [
    "How do I update on Ubuntu?",
    "If you installed via the apt repository, updates arrive with the system: sudo apt update && sudo apt install --upgrade mp3-tagger. Windows users download the new Setup .exe from the releases page.",
  ],
  [
    "Can I contribute or report a bug?",
    "Absolutely — issues and pull requests are welcome on GitHub. The app itself is written in Rust; this site lives in the website/ directory.",
  ],
];

export default function Faq() {
  return (
    <Section
      id="faq"
      eyebrow="FAQ"
      title="Frequently asked questions"
    >
      <div className="max-w-3xl divide-y divide-white/10 border-y border-white/10">
        {ITEMS.map(([question, answer]) => (
          <details
            key={question}
            className="group px-1 py-6"
          >
            <summary className="cursor-pointer list-none text-sm font-medium text-zinc-200 marker:hidden hover:text-white sm:text-base [&::-webkit-details-marker]:hidden">
              <span className="flex items-center justify-between gap-4">
                {question}
                <span aria-hidden="true" className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-white/10 text-indigo-300 transition group-open:rotate-45">+</span>
              </span>
            </summary>
            <p className="mt-4 text-sm leading-7 break-words text-zinc-400">{answer}</p>
          </details>
        ))}
      </div>
    </Section>
  );
}
