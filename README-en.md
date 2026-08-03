<div align="center">

<img src="src/assets/logo.png" alt="logo" width="200" height="200">

## SeaLantern-Connect

A lightweight companion client for [SeaLantern](https://github.com/SeaLantern-Studio/SeaLantern)

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
</div>

<kbd>[简体中文](README.md)</kbd> <kbd>English</kbd>

</div>

## What it does

SeaLantern Connect makes Minecraft: Java Edition multiplayer simple: create a room, share an invite, and enter the world together.
No public IP or manual port forwarding is required.

## Highlights

- **Easy hosting**: Open your world to LAN and create a multiplayer room in moments.
- **Invite links**: Share a link that friends can open to join.
- **Reliable connections**: Automatic connection, reconnection, and clear status updates.
- **Lightweight mode**: Stay quietly available in the background when not in use.
- **Native look and feel**: Mica and Acrylic on Windows, with Vibrancy and Liquid Glass on macOS.
- **Personalization**: Bilingual UI, light and dark modes, custom color themes, and font options.

## Development

This project uses [only](https://github.com/KercyDing/only) as its development toolkit. See [here](https://github.com/KercyDing/only#install) for installation instructions.

### Common commands

Start the dev server:

```bash
only dev
```

Enable DEBUG-level logging:

```bash
only dev debug
```

Build the application:

```bash
only build
```

On Arch Linux and Arch-based distributions, build and install a local pacman package:

```bash
only arch install
```

### Local CI

Please run the local CI suite before committing:

```bash
only ci
```

### Testing deep links during development

Development builds register the `sculk` scheme on Windows and Linux. Use a real room invitation to test the complete flow.

On macOS, protocol activation can only be tested with a bundled app installed in `/Applications`.

## License

[Apache License 2.0](LICENSE)
