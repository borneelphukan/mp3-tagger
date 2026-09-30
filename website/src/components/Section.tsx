import type { ReactNode } from "react";

interface SectionProps {
  id: string;
  eyebrow: string;
  title: string;
  intro?: string;
  children: ReactNode;
}

export default function Section({ id, eyebrow, title, intro, children }: SectionProps) {
  return (
    <section id={id} aria-labelledby={`${id}-heading`} className="scroll-mt-20 border-t border-white/6 even:bg-white/[0.012]">
      <div className="mx-auto max-w-6xl px-5 py-20 sm:px-6 lg:py-24">
        <p className="eyebrow">
          {eyebrow}
        </p>
        <h2 id={`${id}-heading`} className="mt-4 text-3xl font-medium tracking-[-0.035em] text-white sm:text-4xl">
          {title}
        </h2>
        {intro && <p className="mt-4 max-w-2xl text-base leading-7 text-zinc-400">{intro}</p>}
        <div className="mt-10 min-w-0 sm:mt-12">{children}</div>
      </div>
    </section>
  );
}
