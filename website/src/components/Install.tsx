import { useState } from "react";
import { APT_MANUAL, APT_ONELINER, APT_UPGRADE } from "../site.ts";
import CodeBlock from "./CodeBlock.tsx";
import Section from "./Section.tsx";

type TabId = "ubuntu" | "windows" | "manual";

const TABS: { id: TabId; label: string }[] = [
  { id: "ubuntu", label: "Ubuntu / Debian" },
  { id: "windows", label: "Windows" },
  { id: "manual", label: "Manual apt setup" },
];

export default function Install() {
  const [tab, setTab] = useState<TabId>("ubuntu");

  return (
    <Section
      id="install"
      eyebrow="Installation"
      title="Up and running in a minute"
      intro="Three ways in — the one-liner is recommended on Ubuntu."
    >
      <div className="mb-6 inline-flex max-w-full flex-wrap gap-1 rounded-xl border border-white/10 bg-zinc-900/70 p-1.5">
        {TABS.map((entry) => (
          <button
            key={entry.id}
            type="button"
            onClick={() => setTab(entry.id)}
            aria-pressed={tab === entry.id}
            aria-controls="installation-instructions"
            className={
              tab === entry.id
                ? "rounded-lg bg-zinc-700/60 px-4 py-2.5 text-sm font-medium text-white shadow-sm"
                : "rounded-lg px-4 py-2.5 text-sm font-medium text-zinc-400 transition hover:bg-white/5 hover:text-white"
            }
          >
            {entry.label}
          </button>
        ))}
      </div>

      <div id="installation-instructions" className="surface-card p-4 sm:p-7">
      {tab === "ubuntu" && (
        <div className="space-y-4">
          <CodeBlock code={APT_ONELINER} />
          <p className="text-sm text-zinc-400">
            The script adds the signed repository, installs the app, and supports{" "}
            <code className="text-xs">--print</code>, <code className="text-xs">--yes</code> and{" "}
            <code className="text-xs">--remove</code> flags. Later upgrades are just:
          </p>
          <CodeBlock code={APT_UPGRADE} />
        </div>
      )}

      {tab === "windows" && (
        <ol className="max-w-2xl list-decimal space-y-3 pl-6 text-sm leading-relaxed text-zinc-300 marker:text-indigo-400">
          <li>
            Download the <strong className="text-white">Setup .exe</strong> from the{" "}
            <a href="#download" className="text-indigo-400 hover:text-indigo-300">
              download section
            </a>{" "}
            above.
          </li>
          <li>
            Run it and pick the optional components you want: Start Menu entry,
            desktop shortcut, and adding MP3 Tagger to your <code>PATH</code>.
          </li>
          <li>
            Launch from the Start Menu. Your settings live in{" "}
            <code className="rounded bg-zinc-800 px-1.5 py-0.5 text-xs">
              %APPDATA%\mp3-tagger\settings.json
            </code>
            .
          </li>
          <li>
            No installer? Unzip the <strong className="text-white">portable build</strong>{" "}
            anywhere and run <code>mp3-tagger.exe</code> directly.
          </li>
        </ol>
      )}

      {tab === "manual" && (
        <div className="space-y-4">
          <p className="max-w-2xl text-sm text-zinc-400">
            The same steps the one-liner performs, written out for auditing or
            locked-down machines. To remove the repository later, delete the two
            files it creates.
          </p>
          <CodeBlock code={APT_MANUAL} />
        </div>
      )}
      </div>
    </Section>
  );
}
