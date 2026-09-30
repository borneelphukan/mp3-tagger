/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** App version injected at build time (CI passes the Cargo.toml version). */
  readonly VITE_APP_VERSION?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
