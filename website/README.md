# MP3 Tagger website

Product landing page for MP3 Tagger (React + TypeScript + Tailwind CSS v4, built
with Vite). It is deployed to GitHub Pages together with the apt repository, so
the site root (`/`) is this page while `/apt` and `/install.sh` keep working.

## Develop

```bash
cd website
npm install
npm run dev
```

## Build

```bash
cd website
npm run build   # type-checks, then emits dist/
npm run preview # serve the production build locally
```

The displayed app version comes from the `VITE_APP_VERSION` environment variable
(the Debian workflow passes the `Cargo.toml` version); it falls back to `0.1.0`
for local development.

Because the site is served as a GitHub Pages *project* site, `vite.config.ts`
sets `base: "/mp3-tagger/"` — keep that in sync with the repository name.

## Styling

Tailwind v4 runs through `@tailwindcss/vite` in `vite.config.ts`.
`src/main.tsx` imports `src/index.css`, which loads Tailwind and defines the shared
buttons, cards, typography, and reduced-motion styles. Responsive utility classes
live alongside the components. The logo and favicon use the application's
`assets/icon.png`; Vite bundles it with the project-site asset paths.
