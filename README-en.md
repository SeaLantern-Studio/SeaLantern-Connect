<div align="center">

<img src="src/assets/logo.png" alt="logo" width="200" height="200">

## SeaLantern-Connect

A lightweight companion client for [SeaLantern](https://github.com/SeaLantern-Studio/SeaLantern)

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
</div>

<kbd>[简体中文](README.md)</kbd> <kbd>English</kbd>

</div>

## What it does

After a host opens a single-player Minecraft world to LAN, SeaLantern Connect can discover its port, create a P2P room, and generate a shareable HTTPS invitation on the official website. Hosts can configure invitation refresh intervals and maximum player counts.

Players can open the invitation page and click to launch SeaLantern Connect, or paste either the HTTPS share link or the raw `sculk://` invitation. The app asks for confirmation before joining or switching rooms. A successfully used invitation is saved locally and automatically filled into the input the next time the app starts.

It establishes a direct P2P tunnel to the host, then exposes the remote world as a LAN server in your Minecraft multiplayer menu.

You can also connect manually using the local address shown in the app.

> No public IP. No router port forwarding. No fuss.

The app also shows direct or relay routes, latency and traffic statistics, supports reconnection, system tray and lightweight modes, and provides Chinese and English interfaces with customizable themes.

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
