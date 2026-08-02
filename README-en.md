<div align="center">

<img src="src/assets/logo.png" alt="logo" width="200" height="200">

## SeaLantern-Connect

A lightweight companion client for [SeaLantern](https://github.com/SeaLantern-Studio/SeaLantern)

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
</div>

<kbd>[简体中文](README.md)</kbd> <kbd>English</kbd>

</div>

## What it does

Minecraft multiplayer should be simple: create a room, share a link, and start playing.

No public IP, no port forwarding. Just open your world to LAN, share an invite link, and let your friends join instantly.

Features include invite management, automatic connection, reconnect support, connection status monitoring, lightweight mode, and customizable themes with bilingual UI support.

## Development

This project uses [only](https://github.com/KercyDing/only) as its development toolkit. See [here](https://github.com/KercyDing/only#install) for installation instructions.

### Common commands

Start the dev server:

```bash
only dev
```

Build the application:

```bash
only build
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

[GNU Affero General Public License v3.0](LICENSE)
