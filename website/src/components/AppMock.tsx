import Icon from "./Icon.tsx";
import Logo from "./Logo.tsx";

const SONGS = [
  { title: "Midnight Drive", artist: "Neon Coast", album: "Afterglow", year: "2024" },
  { title: "Paper Satellites", artist: "June Archive", album: "Low Orbit", year: "2023" },
  { title: "Velvet Static", artist: "Neon Coast", album: "Afterglow", year: "2024" },
  { title: "Copper Skies", artist: "Field Lines", album: "Meridian", year: "2022" },
  { title: "Half-Light Parade", artist: "June Archive", album: "Low Orbit", year: "2023" },
];

/** Decorative illustration, not an interactive editor or an app screenshot. */
export default function AppMock() {
  return (
    <div className="overflow-hidden rounded-xl border border-white/10 bg-[#101014] text-left sm:rounded-2xl" aria-hidden="true">
      <div className="flex items-center justify-between border-b border-white/6 bg-white/3 px-4 py-3 sm:px-5">
        <span className="flex items-center gap-2 text-xs font-medium text-zinc-300"><Logo size={20} /> MP3 Tagger</span>
        <span className="text-xs tracking-[0.3em] text-zinc-500">− □ ×</span>
      </div>
      <div className="flex gap-5 border-b border-white/6 px-5 py-2.5 text-[11px] text-zinc-400">
        {['File', 'AI', 'View', 'Help'].map((item) => <span key={item}>{item}</span>)}
      </div>
      <div className="grid md:grid-cols-[1fr_230px] lg:grid-cols-[1fr_270px]">
        <div className="min-w-0 px-3 py-5 sm:px-5">
          <div className="mb-5 flex items-center justify-between">
            <div><p className="text-sm font-medium text-zinc-100">Your collection</p><p className="mt-1 text-[11px] text-zinc-500">Good music starts with the details.</p></div>
            <span className="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-[11px] text-zinc-300">+ Add songs</span>
          </div>
          <table className="w-full table-fixed text-left text-xs">
            <thead className="text-[10px] tracking-wider text-zinc-500 uppercase">
              <tr><th className="w-3/5 px-2 pb-3 font-medium sm:w-1/2">Title / Artist</th><th className="hidden pb-3 font-medium sm:table-cell">Album</th><th className="w-12 pb-3 text-right font-medium">Year</th></tr>
            </thead>
            <tbody>
              {SONGS.map((song, i) => (
                <tr key={song.title} className={i === 0 ? 'bg-indigo-400/10' : 'border-b border-white/4'}>
                  <td className={`rounded-l-lg px-2 py-3 ${i === 0 ? 'border-l-2 border-indigo-400' : ''}`}>
                    <div className="flex items-center gap-3">
                      <div className={`hidden h-9 w-9 shrink-0 items-center justify-center rounded-md sm:flex ${i === 0 || i === 2 ? 'album-cover text-amber-100/80' : 'bg-zinc-800 text-zinc-500'}`}><Icon name="music" className="h-4 w-4" /></div>
                      <div className="min-w-0"><p className={`truncate font-medium ${i === 0 ? 'text-indigo-200' : 'text-zinc-300'}`}>{song.title}</p><p className="mt-1 truncate text-[10px] text-zinc-500">{song.artist}</p></div>
                    </div>
                  </td>
                  <td className="hidden truncate text-zinc-400 sm:table-cell">{song.album}</td>
                  <td className="rounded-r-lg pr-2 text-right text-[11px] text-zinc-500">{song.year}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <aside className="hidden border-l border-white/6 bg-white/2 p-5 md:block">
          <div className="album-cover relative flex aspect-square items-end overflow-hidden rounded-lg p-4">
            <div className="absolute -top-8 -right-6 h-44 w-44 rounded-full border-[24px] border-white/15" />
            <div className="relative"><p className="text-[9px] tracking-[0.25em] text-white/70 uppercase">Neon Coast</p><p className="mt-1 text-2xl font-light tracking-tight text-amber-50">Afterglow</p></div>
          </div>
          <p className="mt-4 text-sm font-medium text-zinc-100">Midnight Drive</p>
          <p className="mt-1 text-xs text-zinc-500">Neon Coast · Afterglow</p>
          <div className="mt-4 flex gap-2 text-[10px] text-zinc-400"><span className="rounded border border-white/10 px-2 py-1">MP3</span><span className="rounded border border-white/10 px-2 py-1">ID3v2</span></div>
          <div className="mt-5 flex items-center gap-2 rounded-lg border border-indigo-400/15 bg-indigo-400/10 px-3 py-2.5 text-xs text-indigo-300"><Icon name="sparkle" className="h-4 w-4" /> Metadata, filled in.</div>
        </aside>
      </div>
      <div className="flex justify-between border-t border-white/6 px-5 py-3 text-[10px] text-zinc-500">
        <span>5 tracks in your library</span><span className="flex items-center gap-1.5 text-emerald-400/80"><Icon name="check" className="h-3 w-3" /> All changes saved</span>
      </div>
    </div>
  );
}
