import Section from "./Section.tsx";

const FORMATS: [string, string][] = [
  ["MP3", "ID3v2"],
  ["FLAC", "Vorbis Comments"],
  ["OGG", "Vorbis Comments"],
  ["M4A / MP4", "iTunes-style atoms"],
  ["WAV", "RIFF INFO / ID3"],
  ["AIFF", "ID3"],
  ["APE", "APE Tags"],
];

const FIELDS: [string, string][] = [
  ["Basic", "Title, Artist, Album, Album Artist, Year, Genre"],
  ["Track info", "Track Number, Total Tracks, Disc Number, Total Discs"],
  ["Credits", "Composer, Lyricist, Remixer, Producer"],
  ["Additional", "Comment, Lyrics, BPM, ISRC"],
  ["Artwork", "Embedded cover art (view, add, remove)"],
];

export default function Formats() {
  return (
    <Section
      id="formats"
      eyebrow="Coverage"
      title="Formats & fields"
      intro="One editor for your whole collection — every format keeps its native tag standard."
    >
      <div className="grid gap-4 lg:grid-cols-2">
        <div className="overflow-hidden rounded-2xl border border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-zinc-800 bg-zinc-900 text-xs tracking-wide text-zinc-500 uppercase">
                <th className="px-5 py-3 font-medium">Format</th>
                <th className="px-5 py-3 font-medium">Tag standard</th>
              </tr>
            </thead>
            <tbody>
              {FORMATS.map(([format, standard]) => (
                <tr key={format} className="border-b border-zinc-800/60 last:border-0">
                  <td className="px-5 py-3 font-semibold text-white">{format}</td>
                  <td className="px-5 py-3 text-zinc-400">{standard}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="overflow-hidden rounded-2xl border border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-zinc-800 bg-zinc-900 text-xs tracking-wide text-zinc-500 uppercase">
                <th className="px-5 py-3 font-medium">Category</th>
                <th className="px-5 py-3 font-medium">Fields</th>
              </tr>
            </thead>
            <tbody>
              {FIELDS.map(([category, fields]) => (
                <tr key={category} className="border-b border-zinc-800/60 last:border-0">
                  <td className="px-5 py-3 font-semibold whitespace-nowrap text-white">
                    {category}
                  </td>
                  <td className="px-5 py-3 text-zinc-400">{fields}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </Section>
  );
}
