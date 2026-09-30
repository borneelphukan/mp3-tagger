import Download from "./components/Download.tsx";
import Faq from "./components/Faq.tsx";
import Features from "./components/Features.tsx";
import Footer from "./components/Footer.tsx";
import Formats from "./components/Formats.tsx";
import Hero from "./components/Hero.tsx";
import Install from "./components/Install.tsx";
import Navbar from "./components/Navbar.tsx";
import Usage from "./components/Usage.tsx";

export default function App() {
  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-50">
      <a href="#main-content" className="sr-only fixed top-3 left-3 z-[60] rounded-lg bg-indigo-500 px-4 py-3 text-sm text-white focus:not-sr-only">Skip to content</a>
      <Navbar />
      <main id="main-content" tabIndex={-1}>
        <Hero />
        <Features />
        <Download />
        <Install />
        <Formats />
        <Usage />
        <Faq />
      </main>
      <Footer />
    </div>
  );
}
