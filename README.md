<div align="center">

<img src="./assets/icon.png" alt="Quirks" width="256">

# Alaydriem's Quirks

[![Release](https://img.shields.io/github/v/release/alaydriem/amml-quirks?include_prereleases)](https://github.com/alaydriem/amml-quirks/releases)
[![License](https://img.shields.io/github/license/alaydriem/amml-quirks)](./LICENSE)
![Minecraft Bedrock](https://img.shields.io/badge/Minecraft_Bedrock-1.26.62-62B47A)
[![AMML](https://img.shields.io/badge/AMML-mod-4A90D9)](https://github.com/alaydriem/amml)

</div>

Small fixes for things in Minecraft Bedrock that annoy players.

Client-side only.

## Features

- Stop idle timeout. After 15 minutes with no input, the game shows an away screen and lowers the frame rate. This mod stops both.

The mod sends no key, mouse, button or packet. It never moves your player.

## Installation

Install it with the [AMML launcher](https://github.com/alaydriem/amml/releases/latest). The launcher also installs the AMML loader and Alaydriem's Mod Manager.

Without the Mod Manager, the mod still works. The default setting (on) applies.

## Configuration

Settings are in the mod manager panel and saved to `mods/amml/@alaydriem/quirks/config/config.hcl`. A change applies at once.

| Setting | Default | Description |
| --- | --- | --- |
| `idle.enabled` | `true` | Stops the away screen and the low frame rate after 15 minutes with no input. Does not move your player |

## FAQ

**Does this stop AFK kicks on servers?** No. It does not move your player. Server plugins that kick for no movement still work.

**Does it work with DejaVu?** Yes. When the DejaVu studio is open, DejaVu also blocks the idle screen. This is true whatever the `idle.enabled` setting is.

## Building

Requires Windows x64, [mise](https://mise.jdx.dev), rustup and [cargo-nextest](https://nexte.st). The AMML SDK and the mod manager crates are git dependencies on their public release tags (`Cargo.toml`); cargo fetches them. No sibling repos are needed.

```powershell
mise trust
mise run preflight
mise run package
mise run test
```

`mise run package` writes `dist/@alaydriem/quirks/` (`alaydriem_quirks.dll`, `amml.hcl` and `icon.png`). `mise run install [--instance <name>]` runs `package`, then copies it into `%LOCALAPPDATA%\amml\instances\<name>\mods\amml\@alaydriem\quirks\` (default instance `1.26.62.01`). Close the game first.

The game functions Quirks hooks are found by AMML recipes in the `amml` repository (`recipes/`).

## License

BSD 3-Clause. Copyright (c) 2026 Charles R. Portwood II. See [LICENSE](./LICENSE).
