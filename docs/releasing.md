# Releasing SoundCraft

> **This fork (685ks/soundcraft).** Releases are made by
> [`.github/workflows/release.yml`](../.github/workflows/release.yml), which calls the shared
> Arthavan Projects workflow (`685ks/craft-actions`, `release-rust-app.yml`) with the ids in
> [`packaging/app.toml`](../packaging/app.toml) (bundle id `app.freedev.ainz.ks.SoundCraft`,
> publisher Arthavan Projects). Run it from **Actions › Release › Run workflow** with the version; it
> creates the GitHub Release `v<version>` (a draft unless `publish` is set), which the Arthavan
> Projects launcher installs and updates from. A copy started by the launcher
> (`ARTHAVAN_LAUNCHER=1`) says "Updates are managed by Arthavan Projects" in its About dialog;
> SoundCraft has no updater of its own. The upstream process below is kept for reference; its
> `release` branch trigger and the FreeBSD, Windows ARM64, packaging-lint and Nix workflows are
> not used here. This fork ships for Windows (and the web) only: the macOS, Linux and FreeBSD
> packaging was removed.

Upstream, every push to the `release` branch runs `.github/workflows/release.yml`. The workflow builds
signed installers for Windows and the web, then creates or updates a
**draft** GitHub Release named `SoundCraft v<version>`. Nobody sees a draft until a maintainer
publishes it.

This is SoundCraft's implementation of the shared [release playbook](release-playbook.md).
User-facing names say **SoundCraft**. Files, binaries and ids stay lowercase
(`soundcraft-<version>-<platform>-<arch>.<ext>`, `app.freedev.ainz.ks.SoundCraft`).

## Cutting a release

1. **Bump the version** on `main`. The only place it lives is `[workspace.package] version` in
   the root `Cargo.toml`; the app reads it from `CARGO_PKG_VERSION` (*About SoundCraft*,
   `soundcraft --version`, `soundcraft-cli --version`).

   ```sh
   cargo xtask version                 # prints the current version, e.g. 0.1.0
   cargo xtask version set 0.2.0       # or 0.2.0-rc.1; updates Cargo.toml and Cargo.lock
   ```

   Commit the change (`Cargo.toml` + `Cargo.lock`) through the normal review flow.
2. **Merge `main` into `release`** (or fast-forward it) and push. The workflow starts by itself.
3. **Wait for the draft.** After about 30 to 60 minutes (notarization and the FreeBSD VM are the
   slow parts), the Releases page has a draft `SoundCraft v0.2.0`, tagged `v0.2.0` on the pushed
   commit, with every artifact and `SHA256SUMS.txt`. The notes are generated from the merged PRs.
4. **Check it.** Download an installer or two and look at the job summaries. Any `::warning::`
   there means a signing secret was missing and that artifact is unsigned.
5. **Publish** the draft in the GitHub UI. Publishing creates the `v0.2.0` tag. Versions with a
   pre-release suffix (`-rc.1`) are marked as pre-releases.

Pushing to `release` again before you publish rebuilds the same draft and replaces its assets.
After the draft is published, the workflow refuses to touch that version again, so bump it first.

**Test runs:** *Actions → Release → Run workflow* runs the whole pipeline by hand. The optional
`version` input (such as `0.2.0-rc.1`) overrides `Cargo.toml` for that run only. The jobs apply it
with `cargo xtask version set` before building, so the binaries report it too. Only the signing
jobs (Windows) and the draft-release job use the `release` environment, which only the
`release` branch can use. On any other branch the run is a dry run: the web build and are checked, the signing jobs are refused, and no release is drafted.

## What gets built

| Platform | Artifacts | Built on |
|---|---|---|
| Windows 10+ x64 | `soundcraft-<v>-windows-x64.msi`, `soundcraft-<v>-windows-x64-portable.zip` | `windows-latest` |
| Windows 10+ x86 (32-bit) | `soundcraft-<v>-windows-x86.msi`, `soundcraft-<v>-windows-x86-portable.zip` | `windows-latest` |
| Windows 11 ARM64 | `soundcraft-<v>-windows-arm64.msi`, `soundcraft-<v>-windows-arm64-portable.zip` | `windows-latest` (cross-compiled; `windows-arm64.yml` installs and runs it on ARM64) |
| Web | `soundcraft-web-<v>.zip` (static site; see [`packaging/web/README.md`](../packaging/web/README.md)) | `ubuntu-latest` |

### Audio libraries

SoundCraft plays and records through [cpal](https://github.com/RustAudio/cpal): WASAPI on
Windows and WebAudio in the browser.

- **Web**: browsers start audio only after a user gesture, so the first click in the page may be
  needed before sound plays. The web build has no recording.

### Windows

`packaging/windows/package.ps1 -Arch x64|x86|arm64` builds with `-C target-feature=+crt-static`
(in `CARGO_TARGET_<TRIPLE>_RUSTFLAGS`, so host build scripts aren't affected), so neither the MSI
nor the portable zip needs the Visual C++ redistributable.

- `apps/soundcraft/build.rs` embeds the icon (`assets/app-icon/soundcraft.ico`) and VERSIONINFO
  with `winresource`, only when targeting Windows. Release packaging sets
  `SOUNDCRAFT_REQUIRE_WINRES=1` so a missing resource compiler fails the build.
- The script checks both PE headers: the machine type matches `-Arch`, `soundcraft.exe` is a GUI
  app (no console window) and `soundcraft-cli.exe` a console app.
- `soundcraft.wxs` (WiX v5) is a per-machine install into Program Files with an advertised Start
  Menu shortcut. SoundCraft becomes the default app for `.scraft` and is listed under *Open with*
  for `.wav`, `.aif`, `.aiff`, `.flac`, `.mp3`, `.mid` and `.midi`. It also registers App Paths
  (Win+R `soundcraft`). The MSI version is the numeric `X.Y.Z`; same-version upgrades let release
  candidates replace each other. The upgrade code `EE53D3E5-D0AE-40F9-808D-9417FDEBF61A` is fixed
  forever.
- **Signing:** `packaging/windows/sign.ps1` signs both `.exe` files and then the `.msi` with
  `signtool` (SHA-256, RFC 3161 timestamp), using a `.pfx` (`WINDOWS_CERTIFICATE*`) if present,
  otherwise Azure Trusted Signing (all six `AZURE_*`), otherwise nothing (a warning).

Locally on Windows: `dotnet tool install -g wix --version 5.0.2`, then
`pwsh packaging/windows/package.ps1 -Arch x64`.

### Web

`packaging/web/package.sh` runs `trunk build --release` in `apps/soundcraft-web` (output in
`dist/web`, `public_url = "./"`) and zips the site with sample `_headers` and `.htaccess` files and
the hosting guide. `apps/soundcraft-web` runs the same `SoundApp` as the desktop app on the
synthesised demo session, rendering with wgpu (WebGPU, or WebGL2 with `?webgl`) and playing
through WebAudio.

## Secrets

All secrets live in the repository's **`release` environment** (*Settings → Environments →
release*), restricted to the `release` branch. Only the jobs that sign (Windows) and the
draft-release job declare `environment: release`; the others get no secrets. Each secret is optional: if one is missing, that platform's artifacts
are unsigned and the run shows a `::warning::`. Scripts never echo secret values; certificates
are written with `umask 077` and deleted in `always()`/`finally` steps.

| Secret | Used for |
|---|---|
| `APPLE_CERTIFICATE` | base64 `.p12` with a "Developer ID Application" certificate |
| `APPLE_CERTIFICATE_PASSWORD` | password for that `.p12` |
| `KEYCHAIN_PASSWORD` | password for the temporary CI keychain (random if unset) |
| `APPLE_ID` | Apple ID used by `notarytool` |
| `APPLE_PASSWORD` | app-specific password for that Apple ID |
| `APPLE_TEAM_ID` | `DJ6XS33FX8` |
| `WINDOWS_CERTIFICATE` / `WINDOWS_CERTIFICATE_PASSWORD` | base64 `.pfx` (unused: Windows uses Azure) |
| `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` | service principal for Azure Trusted Signing |
| `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE` | Trusted Signing endpoint, account and profile |

`GITHUB_TOKEN` creates the release. Only the final job gets `contents: write`. Provisioning the
environment, ruleset and secrets for a new repo: craftrules `release/signing-setup.md`.

## Icons

`assets/app-icon/icon.svg` is the canonical icon: a teal waveform glyph in the Arthavan Projects
house style, on a dark teal tile, drawn as original SVG; see
`assets/app-icon/README.md`. `packaging/icons.sh` regenerates the 1024 px PNG, the macOS 512 px
window icon, the `.icns`, the `.ico` (packed by `cargo xtask ico`) and the hicolor PNGs from it.
It needs `resvg`, plus `iconutil` on macOS. The outputs are committed, so packaging never needs
those tools. `apps/soundcraft/src/main.rs` sets the runtime window/Dock/taskbar icon from them.

## Checks

`.github/workflows/packaging-lint.yml` runs in seconds on any change to `packaging/`, the
workflows or the icons: actionlint, shellcheck, a PowerShell parse, xmllint (WiX, plists,
AppStream, MIME), `desktop-file-validate`, `appstreamcli validate`, and a check that the two
Flatpak manifests agree on runtime, finish-args and plugin extensions.
