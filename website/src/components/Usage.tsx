import Section from "./Section.tsx";

const STEPS = [
  ["Add songs", "Click “Add Songs” and pick one file — or a whole folder of them."],
  ["Open details", "Double-click a title in the library to open the detail editor."],
  ["Auto-fill", "Enter Title + Artist, then Fill Data to fetch the rest from your LLM."],
  ["Review & save", "Check the fields and artwork preview, then Save to write the tags."],
  ["Go back", "Return to the library with Back — repeat until the collection shines."],
];

const SHORTCUTS: [string, string][] = [
  ["Double-click", "Open song detail"],
  ["Enter (in detail)", "Save changes"],
  ["Escape", "Go back / close dialog"],
];

export default function Usage() {
  return (
    <Section
      id="usage"
      eyebrow="Workflow"
      title="From messy folder to clean library"
      intro="The loop is short on purpose: add, inspect, auto-fill, save."
    >
      <ol className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {STEPS.map(([title, text], index) => (
          <li
            key={title}
            className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5"
          >
            <span className="flex h-8 w-8 items-center justify-center rounded-full bg-indigo-600 text-sm font-bold text-white">
              {index + 1}
            </span>
            <h3 className="mt-3 text-sm font-semibold text-white">{title}</h3>
            <p className="mt-1.5 text-sm leading-relaxed text-zinc-400">{text}</p>
          </li>
        ))}
      </ol>
      <div className="mt-6 flex flex-wrap gap-2">
        {SHORTCUTS.map(([key, action]) => (
          <span
            key={key}
            className="inline-flex items-center gap-2 rounded-lg border border-zinc-800 bg-zinc-900/60 px-3 py-2 text-xs text-zinc-400"
          >
            <kbd className="rounded border border-zinc-700 bg-zinc-950 px-1.5 py-0.5 font-mono text-zinc-200">
              {key}
            </kbd>
            {action}
          </span>
        ))}
      </div>
    </Section>
  );
}
