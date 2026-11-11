# 12 — Packaging and Release

Every packaging script packages an **already-built** binary. None of them invoke Cargo. CI builds first, then runs the scripts with `VERSION`, `ARCH`/`RPM_ARCH`, `BINARY`, and `OUT_DIR` environment variables.

## 1. Artifact matrix

| Family | Architectures produced in CI | Script / definition | Output name |
|---|---|---|---|
| Debian | `amd64` (native), `riscv64` (cross) | `packaging/debian/build-deb.sh` | `xur-porisoy_<version>_<debarch>.deb` |
| RPM | `x86_64`, `aarch64`, `riscv64` | `packaging/rpm/build-rpm.sh` | `xur-porisoy-<version>-1.<rpmarch>.rpm` |
| Linux tarball | `x86_64`, `aarch64`, `riscv64` | `packaging/linux/build-tarball.sh` | `XurPorisoy-<version>-linux-<arch>.tar.gz` |
| AppImage | `x86_64`, `aarch64` | `packaging/linux/build-appimage.sh` | `XurPorisoy-<version>-<arch>.AppImage` |
| Flatpak | `x86_64`, `aarch64` | `packaging/flatpak/build-flatpak.sh` | `XurPorisoy-<version>-<arch>.flatpak` |
| Arch | `x86_64`, `aarch64` | `packaging/arch/build-arch-pkg.sh` + `PKGBUILD` | `xur-porisoy-<version>-1-<arch>.pkg.tar.*` |
| Windows installer | `x64`, `x86`, `arm64` | `installer/xur-porisoy.iss` | `XurPorisoy-Setup-<version>-<arch>.exe` |
| Windows portable | `x64`, `x86`, `arm64` | built inline in the workflow | `XurPorisoy-<version>-windows-<arch>.zip` |
| macOS | universal (`x86_64` + `arm64`) | `packaging/macos/build-dmg.sh` | `XurPorisoy-<version>-macos-universal.dmg` |
| apt repository | `amd64` (published) | `packaging/apt/build-repo.py` | static tree under `pages/` |

`<version>` is the first `version =` line in `Cargo.toml`, currently `0.0.1`. Windows also receives a four-field numeric version `X.Y.Z.0`.

## 2. Debian (`.deb`)

`packaging/debian/build-deb.sh`:

1. Resolves `BINARY` (default `target/release/xur-porisoy`), `OUT_DIR` (default `dist`), and `DEB_ARCH` from `dpkg-architecture`/`dpkg`.
2. Reads `version`, `description`, `homepage`, and `authors` from `cargo metadata --no-deps`; falls back to parsing `Cargo.toml`, then to hard-coded values.
3. Derives the minimum glibc from `objdump -T` (`GLIBC_x.y`), falling back to `DEB_GLIBC_MIN` (default `2.17`).
4. Builds a staged tree:
   - `/usr/bin/xur-porisoy`
   - `/usr/share/applications/xur-porisoy.desktop`
   - `/usr/share/metainfo/xur-porisoy.metainfo.xml` (version/date rewritten with `sed`)
   - `/usr/share/doc/xur-porisoy/README.md`
   - `copyright` from `copyright.in` with maintainer/homepage/year/holder substituted
   - `LICENSE` **only if the repo has one** (it does not)
   - `/usr/share/icons/hicolor/256x256/apps/xur-porisoy.png`
   - maintainer scripts `postinst`, `prerm`, `postrm`
5. Writes `DEBIAN/control` with `Section: sound`, `Priority: optional`, `Depends`, `Installed-Size`, `Maintainer`, `Homepage`, and a three-line description.
6. Generates `DEBIAN/md5sums` and builds with `dpkg-deb --root-owner-group -Zxz`.

Dependencies are explicit because the GUI stack is `dlopen`ed:

```text
libc6 (>= <detected>), libgcc-s1, libdbus-1-3, libegl1, libgl1,
libwayland-client0, libwayland-egl1,
libx11-6, libx11-xcb1, libxcb1, libxcursor1, libxi6,
libxkbcommon0, libxkbcommon-x11-0, libxrender1
```

`libm6` is deliberately **not** listed: glibc ≥ 2.34 provides it via `libc6`. The CI verifies the package layout with `dpkg-deb --extract/--control`, `desktop-file-validate`, `appstreamcli validate`, executable/icon/doc checks, and `sh -n` on the maintainer scripts.

RISC-V builds pass `DEB_ARCH=riscv64` and a cross-compiled `BINARY`.

## 3. apt repository (GitHub Pages)

`packaging/apt/build-repo.py` builds a plain static tree:

```text
<out>/apt/dists/<suite>/Release[.gpg]
<out>/apt/dists/<suite>/InRelease
<out>/apt/dists/<suite>/main/binary-<arch>/Packages[.gz|.xz]
<out>/apt/pool/main/<letter>/<package>/<file>.deb
<out>/install.sh
<out>/index.html
<out>/xur-porisoy-archive-keyring.gpg  (signed only)
<out>/UNSIGNED                          (unsigned only)
```

- `apt-ftparchive` is intentionally not used; the script implements control parsing, pool copying, Packages writing, gzip/xz compression, Release generation, and checksums in Python.
- Architectures are derived from the `.deb` files present in `--debs`.
- Signing (`--gpg-key-id`, `--gpg-key-file`, `--gpg-passphrase-file`) imports the key, writes `InRelease` (clearsign) and `Release.gpg` (armored detach-sign), and exports the public keyring.
- Without a key id it writes `UNSIGNED`, adds `Acquire-By-Hash: no`, and the generated HTML says "(unsigned repository)".
- `REPO_BASE` is hard-coded to `https://borneelphukan.github.io/xur-porisoy` in the Python script; `install.sh` has the same default with `XUR_PORISOY_REPO_URL` as an override.
- CI publishes only the amd64 artifact (see [11-ci-cd.md](11-ci-cd.md)); a local run can include any set of `.deb` files.

`packaging/apt/install.sh`:

- Detects the signing key by fetching `<base>/xur-porisoy-archive-keyring.gpg`; if missing, uses `Trusted: yes`.
- Writes `/etc/apt/sources.list.d/xur-porisoy.sources` using the deb822 format.
- Re-execs itself with `sudo` when not root.
- Flags: `--print/-n`, `--remove`, `--yes/-y`, `--help/-h`.
- Never reads stdin, so it is safe to pipe.
- Installs `xur-porisoy` with `apt-get install`.

See [13-website.md](13-website.md) for the Pages site itself.

## 4. Linux tarball

`packaging/linux/build-tarball.sh` stages:

```text
XurPorisoy-<version>-linux-<arch>/
├── xur-porisoy
├── README.md
├── LICENSE            # only if present (it is not)
├── xur-porisoy.desktop
└── xur-porisoy.png
```

Creates a `.tar.gz` with `--owner=0 --group=0 --numeric-owner`.

## 5. AppImage

- Builds an `AppDir` with `usr/bin/xur-porisoy`, desktop file, and icon.
- Writes an `AppRun` that resolves its own directory with `readlink -f` and execs the binary.
- Downloads `appimagetool-<arch>.AppImage` (continuous release) unless `APPIMAGETOOL` is set.
- Runs it with `APPIMAGE_EXTRACT_AND_RUN=1` and `--no-appstream`.
- Supports only `x86_64` and `aarch64`; other values fail early.

## 6. RPM

- Generates a spec in a temporary `_topdir`.
- Uses `%global debug_package %{nil}` and `__strip /bin/true` because the binary is already stripped.
- Installs binary, desktop file, AppStream metadata, icon, and README.
- `%post`/`%postun` refresh the desktop database and icon cache when the tools exist.
- No explicit `Requires`; the prebuilt binary's dependencies are not auto-detected because nothing is compiled in the spec. This is a packaging gap to check if RPM users report missing libraries.
- The spec description repeats the README's "more than 34 metadata fields" claim, which is inaccurate (18 are exposed).

Build for `riscv64` uses the cross-compiled binary; `rpmbuild --target riscv64` is invoked on an x86_64 host.

## 7. Flatpak

- App id `io.github.borneelphukan.XurPorisoy`.
- Runtime `org.freedesktop.Platform` **25.08**, SDK `org.freedesktop.Sdk`.
- Finish args: `--share=ipc`, `--share=network`, `--socket=wayland`, `--socket=fallback-x11`, `--device=dri`, `--filesystem=home`.
- Builds entirely from four staged files with `buildsystem: simple`; no network fetch during build.
- The CI runs `sysctl -w kernel.apparmor_restrict_unprivileged_userns=0` first because Ubuntu 24.04 restricts unprivileged user namespaces, which breaks bubblewrap.
- `flatpak-builder --user --disable-rofiles-fuse ...` then `flatpak build-bundle`.

`--filesystem=home` gives the sandbox broad access, which is needed to open/save arbitrary music files.

## 8. Arch

- `arch/build-arch-pkg.sh` selects `archlinux:base-devel` for x86_64 and `menci/archlinuxarm:base-devel` for aarch64, then runs `makepkg` as a non-root `builder` user inside Docker.
- `PKGBUILD` packages a prebuilt binary; `PKGVER`, `BINARY`, and `ROOT` are injected by the script.
- Dependencies: `glibc`, `gcc-libs`, `dbus`, `libglvnd`, `wayland`, `libx11`, `libxcb`, `libxcursor`, `libxi`, `libxkbcommon`, `libxrender`.
- `options=('!strip' '!debug')`; `source=()` and `sha256sums=()` because nothing is downloaded.
- Only x86_64 and aarch64 are supported.

## 9. Windows

Built with Inno Setup (located from `ProgramFiles`/`ProgramFiles(x86)` or installed via Chocolatey).

Workflow defines:

```text
/DMyAppVersion=<version>
/DMyVersionInfo=<numeric>
/DMyArch=<x64|x86|arm64>
/DMySourceExe=<target path>
/DMyOutputBaseFilename=XurPorisoy-Setup-<version>-<arch>
```

The `.iss`:

- `AppId={{6E1C0A6F-...}` (fixed GUID).
- `DefaultDirName={autopf}\XurPorisoy`.
- Offers an unchecked desktop-icon task; the Start Menu group is always created.
- Installs the exe, `assets/icon.ico`, `README.md` into `docs`, and `LICENSE` with `skipifsourcedoesntexist`.
- Adds Start Menu entries for the app, documentation, and uninstaller.
- Runs the app post-install unless silent.
- `OutputDir=..\target`.

The workflow also builds a portable zip containing the exe and `README.md`. It verifies `ProductName == "XurPorisoy"` and `FileVersion` matches the Cargo version before packaging.

**README discrepancy:** the root README claims the installer offers "adding the app to `PATH`"; the `.iss` has no PATH task or registry entry.

## 10. macOS

- Expects a universal binary (built in CI with two `cargo build` runs + `lipo`).
- Creates `XurPorisoy.app/Contents/{MacOS,Resources}`.
- Builds `.icns` with `sips` (cropped to 301×301 first) and `iconutil`, with 16/32/128/256/512 and `@2x`.
- `Info.plist`: bundle id `io.github.borneelphukan.XurPorisoy`, version, `LSMinimumSystemVersion 11.0`, high-resolution capable, category `public.app-category.music`.
- `hdiutil create -format UDZO` produces the DMG. The app is **not code-signed or notarized**; Gatekeeper will warn.

## 11. Version bumping

1. Change `version` in `Cargo.toml`.
2. Run `cargo check`/`cargo build` to update `Cargo.lock` (the lock records the root package version).
3. Commit both files. Every packaging script reads the version from Cargo metadata or `Cargo.toml`; only the Windows numeric version and AppStream release date are derived.
4. Tag `v<version>` to publish.

No other file hard-codes `0.0.1` except documentation examples. The `installer` defaults (`0.0.0`) are only used when the workflow does not pass defines.

## 12. Release verification

The `release` job:

- downloads all artifacts with `merge-multiple`,
- asserts 22 filename patterns (see [11-ci-cd.md](11-ci-cd.md)),
- writes and verifies `SHA256SUMS`,
- uploads `release-bundle` for non-tag runs,
- for `v*` tags creates the GitHub Release if needed and uploads every file with `--clobber`.

Renaming an output file without updating the expected-pattern list makes the release job fail. Adding a new artifact is safe (it is hashed and uploaded), but it will not be asserted.

## 13. Known packaging gaps

| Gap | Impact |
|---|---|
| No `LICENSE` file | Debian/tarball/installer skip it conditionally; the package declares MIT in metadata only. |
| RPM has no `Requires` | Users may need to install X11/Wayland/GL/DBus libraries manually. |
| macOS app unsigned/notarized | Gatekeeper blocks or warns on first launch. |
| Windows installer is unsigned | SmartScreen warns. |
| apt repo publishes amd64 only | RISC-V users must use the GitHub Release `.deb`. |
| Flatpak uses `--filesystem=home` | Broad sandbox access. |
| AppImage downloads `appimagetool` at build time | Build depends on GitHub's continuous release. |
| "34+ fields" in RPM/README | Only 18 fields are editable. |
| `REPO_BASE` hard-coded | Forks must edit the Python and shell defaults. |

## 14. Manual packaging quickstart

```bash
cargo build --release --locked

packaging/debian/build-deb.sh                       # dist/xur-porisoy_<v>_<arch>.deb
VERSION=<v> ARCH=x86_64 packaging/linux/build-tarball.sh
VERSION=<v> ARCH=x86_64 packaging/linux/build-appimage.sh
VERSION=<v> RPM_ARCH=x86_64 packaging/rpm/build-rpm.sh
VERSION=<v> ARCH=x86_64 packaging/flatpak/build-flatpak.sh
VERSION=<v> ARCH=x86_64 packaging/arch/build-arch-pkg.sh
VERSION=<v> packaging/macos/build-dmg.sh            # macOS only, universal binary
python3 packaging/apt/build-repo.py --debs dist --out pages \
  --suite stable --component main --origin xur-porisoy --label "XurPorisoy"
```

All scripts accept `BINARY` and `OUT_DIR` overrides.
