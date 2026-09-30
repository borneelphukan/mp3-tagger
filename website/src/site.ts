/** Single source of truth for links, filenames and install commands. */

export const APP_VERSION = import.meta.env.VITE_APP_VERSION ?? "0.1.0";
export const APP_TAG = `v${APP_VERSION}`;

export const GITHUB_REPO = "https://github.com/borneelphukan/mp3-tagger";
export const RELEASES_URL = `${GITHUB_REPO}/releases`;
export const ISSUES_URL = `${GITHUB_REPO}/issues`;
export const SOURCE_URL = GITHUB_REPO;

export const PAGES_BASE = "https://borneelphukan.github.io/mp3-tagger";
export const INSTALL_SH_URL = `${PAGES_BASE}/install.sh`;
export const KEYRING_URL = `${PAGES_BASE}/mp3-tagger-archive-keyring.gpg`;
export const APT_POOL_URL = `${PAGES_BASE}/apt/pool/main/m/mp3-tagger/`;

export const SETUP_EXE = `MP3Tagger-Setup-${APP_VERSION}.exe`;
export const PORTABLE_ZIP = `MP3Tagger-${APP_VERSION}-windows-x64.zip`;
export const DEB_FILE = `mp3-tagger_${APP_VERSION}_amd64.deb`;

export const releaseAssetUrl = (file: string) =>
  `${GITHUB_REPO}/releases/download/${APP_TAG}/${file}`;

export const APT_ONELINER = `curl -fsSL ${INSTALL_SH_URL} | sudo bash`;
export const APT_UPGRADE = `sudo apt update && sudo apt install --upgrade mp3-tagger`;

export const APT_MANUAL = `sudo mkdir -p /usr/share/keyrings
curl -fsSL ${KEYRING_URL} \\
  | sudo tee /usr/share/keyrings/mp3-tagger.gpg >/dev/null

sudo tee /etc/apt/sources.list.d/mp3-tagger.sources >/dev/null <<'EOF'
Types: deb
URIs: ${PAGES_BASE}/apt/
Suites: stable
Components: main
Architectures: amd64
Signed-By: /usr/share/keyrings/mp3-tagger.gpg
EOF

sudo apt update
sudo apt install mp3-tagger`;
