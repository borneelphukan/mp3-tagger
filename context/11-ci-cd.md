# 11 — CI/CD and Automation

Three workflows live in `.github/workflows/`. All use the pinned Rust **1.98.1** toolchain and `Swatinem/rust-cache@v2`.

## 1. `ci.yml` — formatting, lints, tests, packaging assets

Triggers:

- `push` to `main`, path-filtered on `src/**`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `assets/**`, `packaging/**`, `ci.yml`.
- `pull_request`, same paths.
- `workflow_dispatch`.

Concurrency `ci-<ref>` with `cancel-in-progress: true`. Permissions: `contents: read`. Env: `CARGO_TERM_COLOR=always`, `RUST_BACKTRACE=1`, `CARGO_INCREMENTAL=0`.

### Job `check` (ubuntu-latest, 30 min)

1. Install Linux build headers (X11/XCB/Wayland/xkbcommon).
2. `dtolnay/rust-toolchain@1.98.1` with `rustfmt, clippy`.
3. `cargo fmt --all -- --check`
4. `cargo clippy --workspace --all-targets -- -D warnings -A clippy::never_loop -A clippy::large_enum_variant`
5. `cargo test --workspace --locked`

The two clippy allows cover pre-existing findings. No release build is performed here; packaging jobs already do one.

### Job `linux-assets` (ubuntu-latest, 10 min)

- `desktop-file-validate packaging/debian/xur-porisoy.desktop`
- `appstreamcli validate --no-net packaging/debian/xur-porisoy.metainfo.xml`
- `bash -n build-deb.sh`; `sh -n` on `postinst`, `prerm`, `postrm`, `install.sh`; `python3 -m py_compile packaging/apt/build-repo.py`

### Path-filter gotchas

`context/**`, `README.md`, `.cargo/**`, and workflow files other than `ci.yml` do **not** trigger CI. A docs-only commit can therefore land without a check run. Adding `context/**` and `README.md` to the filters is a safe improvement.

## 2. `audit.yml` — RustSec advisories

Triggers:

- Schedule: Sundays at `0 6 * * 0` UTC.
- `push`/`pull_request` on changes to `Cargo.toml`, `Cargo.lock`, `.cargo/audit.toml`, `audit.yml`.
- `workflow_dispatch`.

Concurrency `audit-<ref>`, cancels only PRs. Installs `cargo-audit` via `taiki-e/install-action`, runs `cargo audit 2>&1 | tee audit.log`, writes the log to the step summary.

Default `cargo audit` fails only on **vulnerabilities**; unmaintained/unsound advisories are informational. `.cargo/audit.toml` currently has `ignore = []`; informational warnings, severity thresholds, target filtering, and deny-on-informational are all commented out. The file documents that every ignore should record crate, reason, and a revisit date.

## 3. `release.yml` — packaging, Pages, releases

### Triggers

- `push` to `main` or tags `v*`, with path filters (`src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `installer/**`, `packaging/**`, `assets/**`, `release.yml`). **Tag pushes ignore path filters**, so a release always runs.
- `pull_request` with the same paths.
- `workflow_dispatch`.
- Schedule: `37 5 * * 1` (Mondays 05:37 UTC) to keep the apt repository warm and pick up new runner images.

Concurrency `release-<ref>`; cancels in progress only for PRs. Permissions: `contents: write`.

### Jobs

| Job | Needs | Runs when | Purpose |
|---|---|---|---|
| `metadata` | — | Always | Read version/numeric version. |
| `debian-x86_64` | — | Always (including schedule/PR) | Build `.deb` and verify its layout. |
| `linux-packages` | `metadata` | Not schedule | Matrix x86_64 (`ubuntu-24.04`) / ARM64 (`ubuntu-24.04-arm`): tarball, AppImage, RPM, Flatpak, Arch. |
| `linux-riscv64` | `metadata` | Not schedule | Cross-compile with `cross`, tarball + `.deb` + RPM. |
| `windows` | `metadata` | Not schedule | Matrix x64/x86/ARM64: release binary, Inno Setup installer, portable zip. |
| `macos-universal` | `metadata` | Not schedule | Build both targets, `lipo`, DMG. |
| `ci` | — | Not schedule | Duplicate quality gate: fmt, clippy with the same allows, tests. |
| `publish-pages` | `ci`, `debian-x86_64` | Not PR, Debian success, `ci` success or skipped | Build and deploy the apt repository. |
| `release` | `ci`, `debian-x86_64`, `linux-packages`, `linux-riscv64`, `windows`, `macos-universal` | Not schedule | Verify 22 deliverable patterns, write `SHA256SUMS`, upload bundle or attach to release. |

The weekly schedule intentionally runs only `metadata` and `debian-x86_64` plus `publish-pages` (because the other jobs are skipped, and the condition allows `ci` to be skipped). This refreshes the apt repo without rebuilding every package.

### `metadata`

```bash
version="$(sed -n '0,/^version[[:space:]]*=/s/.../p' Cargo.toml)"
```

Takes the **first** `version = "..."` line anywhere in `Cargo.toml` — currently `0.0.1`. `numeric` is `${major}.${minor}.${patch}.0` for Windows version resources. Fails if no version is found.

### `ci` quality gate

Same commands as `ci.yml`. It runs in the release workflow so a tag cannot publish an artifact whose commit would fail CI. The two files must be kept in sync manually; the workflow comments say so. It is skipped on schedule only.

### `publish-pages`

1. Downloads **only** the `Debian package (x86_64)` artifact into `dist/`.
2. Prepares GPG material from `secrets.APT_REPO_PRIVATE_KEY`, `secrets.APT_REPO_PASSPHRASE`, and `vars.APT_REPO_KEY_ID`.
3. Runs `packaging/apt/build-repo.py --debs dist --out pages --suite stable --component main --origin xur-porisoy --label XurPorisoy`, adding GPG arguments when a key is configured.
4. Verifies `Release`, `binary-amd64/Packages`, `install.sh`, required fields, and every SHA256 entry.
5. `actions/configure-pages@v5` with `enablement: true` (first-run support).
6. Uploads `pages` and deploys with `actions/deploy-pages@v4`.
7. Prints the install command in the summary.

Consequences:

- The published apt repository contains **amd64 only**, even though a riscv64 `.deb` is built in `linux-riscv64`. To publish riscv64, download both Debian artifacts into `dist/` and add `riscv64` to the Release architecture list (the script derives architectures from the `.deb` files it sees).
- A PR never touches Pages.
- If the signing key or key id is missing, the repository is published **unsigned** and clients are told to use `Trusted: yes`; the run still succeeds with a warning.

### `release`

1. `actions/download-artifact` with `merge-multiple: true` into `release/`.
2. Asserts these 22 patterns exist (glob names, not exact versions):

   ```text
   xur-porisoy_*_amd64.deb
   xur-porisoy_*_riscv64.deb
   xur-porisoy-*-1.x86_64.rpm
   xur-porisoy-*-1.aarch64.rpm
   xur-porisoy-*-1.riscv64.rpm
   xur-porisoy-*-x86_64.pkg.tar.*
   xur-porisoy-*-aarch64.pkg.tar.*
   XurPorisoy-*-linux-x86_64.tar.gz
   XurPorisoy-*-linux-aarch64.tar.gz
   XurPorisoy-*-linux-riscv64.tar.gz
   XurPorisoy-*-x86_64.AppImage
   XurPorisoy-*-aarch64.AppImage
   XurPorisoy-*-x86_64.flatpak
   XurPorisoy-*-aarch64.flatpak
   XurPorisoy-Setup-*-x64.exe
   XurPorisoy-Setup-*-x86.exe
   XurPorisoy-Setup-*-arm64.exe
   XurPorisoy-*-windows-x64.zip
   XurPorisoy-*-windows-x86.zip
   XurPorisoy-*-windows-arm64.zip
   XurPorisoy-*-macos-universal.dmg
   ```

   Any extra file is still hashed and published; the list only asserts presence.
3. `sha256sum` every file into `SHA256SUMS` and verify it.
4. If the ref is **not** `refs/tags/v*`: upload `release-bundle` (retention 14 days).
5. If the ref **is** `refs/tags/v*`: `gh release create` when missing (title `XurPorisoy <tag>`, generated notes), then `gh release upload <tag> release/* --clobber`. `GH_REPO` is set explicitly because this job does not check out the repo.

Artifacts from packaging jobs have 30-day retention.

### Tagging a release

```bash
# bump version in Cargo.toml (and Cargo.lock via cargo check/build)
git commit -am "chore(release): bump version to X.Y.Z"
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

The tag run ignores path filters and runs every job. `publish-pages` also runs (unless the tag came from a PR, which cannot happen) and republishes the apt repo for the same version.

## 4. Secrets and variables

| Name | Kind | Used for |
|---|---|---|
| `APT_REPO_PRIVATE_KEY` | secret | ASCII-armored GPG private key imported by `build-repo.py`. |
| `APT_REPO_PASSPHRASE` | secret | Written to a 0600 file and passed as `--passphrase-file`. |
| `APT_REPO_KEY_ID` | **variable** (not secret) | Key id used for signing/export. |

All are optional: without them the apt repository is unsigned.

## 5. Pinned versions to keep in sync

| Tool | Version | Where |
|---|---|---|
| Rust | `1.98.1` | `ci.yml`, `release.yml`, every packaging job |
| actions/checkout | `v4` | all workflows |
| rust-cache | `v2` | all |
| upload/download-artifact | `v4` | release |
| configure-pages | `v5` | release |
| upload-pages-artifact | `v3` | release |
| deploy-pages | `v4` | release |
| Inno Setup | whatever Chocolatey installs | Windows job |

Bump Rust deliberately in **both** workflows; a new stable can introduce lints that fail an old tag build.

## 6. Local equivalents

There is no `Makefile` or `justfile`. Use the `cargo` commands in [10-build-run-and-config.md](10-build-run-and-config.md) and the packaging scripts in [12-packaging-and-release.md](12-packaging-and-release.md).

## 7. Workflow gotchas

- `release.yml` and `ci.yml` duplicate the clippy/test gate; changing one without the other leaves tag builds and PR builds with different standards.
- Path filters exclude `context/**`, so documentation-only commits run nothing.
- `debian-x86_64` has no path or event guard beyond the workflow trigger, so it also runs on schedule and PRs.
- The release job runs on PRs and uploads `release-bundle`, which is expensive but gives downloadable PR artifacts.
- `publish-pages` depends on `ci`; if `ci` fails, Pages is not updated even if the `.deb` built.
- The Pages deploy only includes the amd64 `.deb` (see above).
- `actions/configure-pages` with `enablement: true` can still fail under restrictive org policy; enable Pages manually if so.
