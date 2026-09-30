import { useEffect, useRef, useState } from "react";
import Icon from "./Icon.tsx";

interface CodeBlockProps {
  code: string;
  language?: string;
}

/** A terminal-style code block with a copy button. */
export default function CodeBlock({ code, language = "bash" }: CodeBlockProps) {
  const [status, setStatus] = useState<"idle" | "copied" | "error">("idle");
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code);
    } catch {
      setStatus("error");
      return;
    }
    setStatus("copied");
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setStatus("idle"), 2000);
  };

  return (
    <div className="min-w-0 overflow-hidden rounded-xl border border-white/10 bg-[#0c0c10] shadow-lg shadow-black/10">
      <div className="flex items-center justify-between border-b border-white/6 bg-white/2 px-4 py-3">
        <span className="flex items-center gap-2 text-xs text-zinc-500">
          <Icon name="terminal" className="h-3.5 w-3.5" /> {language}
        </span>
        <button
          type="button"
          onClick={() => void copy()}
          aria-label="Copy command"
          className="flex items-center gap-2 rounded-md px-2 py-1 text-xs font-medium text-zinc-400 transition hover:bg-zinc-800 hover:text-zinc-100"
        >
          <Icon name={status === "copied" ? "check" : "copy"} className="h-3.5 w-3.5" />
          <span aria-live="polite">{status === "copied" ? "Copied" : status === "error" ? "Select to copy" : "Copy"}</span>
        </button>
      </div>
      <pre tabIndex={0} aria-label="Installation command" className="overflow-x-auto p-5 text-xs leading-7 text-indigo-100 sm:text-sm">
        <code>{code}</code>
      </pre>
    </div>
  );
}
