# Pywalfox Native Host (Rust)

This repository contains a Linux-only native messaging host for the Pywalfox Firefox add-on. It bridges Firefox with your current Pywal color scheme and exposes control commands (update colors, switch theme mode, etc.). It communicates with the browser via the Native Messaging API and with local helper processes via a Unix domain socket.

Upstream project: https://github.com/Frewacom/pywalfox

## High-level overview

Components in this repo:
- Native messaging host (stdin/stdout) that talks to Firefox.
- Local Unix socket server at /tmp/pywalfox.sock to accept control commands.
- CLI utilities to install/uninstall the Firefox native host manifest and to send commands to the running server.

Data flow:
1. The server binds a Unix domain socket at /tmp/pywalfox.sock and relays any line received from one client to all other connected clients.
2. The extension client connects to that socket and listens for commands (update, dark, light, auto). When it receives one, it emits the appropriate native message back to Firefox via stdout.
3. The native host also listens for requests from the browser (e.g., debug:version and action:colors) and returns responses, including current Pywal colors.

Pywal integration:
- Colors (and optional wallpaper) are read from ~/.cache/wal/pywalfox.json.

## Commands and usage

Build requirements: recent Rust toolchain (stable), cargo.

Build:
- cargo build --release

Run (common tasks):
- Start the socket server:
  - cargo run -- start
- Start the native host process that connects Firefox to the local socket (typically launched by Firefox via the manifest):
  - cargo run -- connect
- Install the Firefox native messaging manifest (user scope):
  - cargo run -- install
- Uninstall the manifest:
  - cargo run -- uninstall
- Trigger a refresh of colors (broadcast to connected clients; the extension client will forward to Firefox):
  - cargo run -- update
- Set theme mode to dark/light/auto:
  - cargo run -- dark
  - cargo run -- light
  - cargo run -- auto
- Print the Firefox native messaging manifest JSON (no file changes):
  - cargo run -- print-manifest

Note: In normal usage, Firefox launches the native host according to the manifest. The server should be started by the user (or a service) so CLI commands can be delivered to the extension client.

## Native message schema (current)

Incoming requests from the browser (stdin) are JSON with a top-level type field (serde adjacently tagged) and an action string. Recognized actions:
- debug:version → returns the program version string.
- action:colors → returns the Pywal colors and optional wallpaper path.

Outgoing messages to the browser:
- action: debug:version | action:colors | theme:mode
- success: boolean
- error: optional string
- data: payload

Example successful colors response:
{
  "type": "response",
  "action": "action:colors",
  "success": true,
  "error": null,
  "data": {
    "colors": ["#111111", "#222222", "..."],
    "wallpaper": "/path/to/wallpaper.jpg"
  }
}

Example theme mode response (when a CLI command is received via the socket):
{
  "type": "response",
  "action": "theme:mode",
  "success": true,
  "error": null,
  "data": "dark" | "light" | "auto"
}

Note: The code uses serde tagging for internal structs; the wire format includes a type field due to #[serde(tag = "type", rename_all = "snake_case")].

## Modules overview

- src/main.rs: Entry point; parses CLI with clap and dispatches to subcommands. Also initializes file-based tracing.
- src/cli.rs: clap CLI definitions and available subcommands.
- src/config.rs: Constants and filesystem paths (host name, allowed extension ID, socket path, manifest locations).
- src/install.rs: Install/uninstall the Firefox native messaging manifest (user scope).
- src/server.rs: Unix domain socket server that broadcasts line-based commands to all connected clients except the sender.
- src/cli_client.rs: Simple client that connects to the socket and sends a single command line (update/dark/light/auto) or runs install/uninstall.
- src/extension_client.rs: Connects to the socket and handles two channels:
  - A background thread reads line-based commands from the socket and converts them into native messages sent to the browser (theme:mode or colors update).
  - The main loop reads messages from Firefox (stdin) and responds with version or current colors.
- src/native_messaging.rs: Helper to encode/decode Native Messaging frames, read/write JSON, and build response payloads with proper tagging.
- src/themes.rs: Reads ~/.cache/wal/pywalfox.json to extract colors and wallpaper.

## Logging

- Controlled by RUST_LOG env (e.g., `RUST_LOG=info` or `RUST_LOG=pywalfox_native=debug`).
- Destination: if PYWALFOX_LOG is set, logs go to that file; otherwise we try the XDG state directory (e.g., `$XDG_STATE_HOME/org/pywalfox/pywalfox-native/pywalfox.log`). If that’s unavailable, logs fall back to stderr.

## Paths and configuration

- Socket path resolution precedence:
  1) `PYWALFOX_SOCKET` (exact path)
  2) `$XDG_RUNTIME_DIR/pywalfox/pywalfox.sock` (dir created with 0700)
  3) `/tmp/pywalfox.sock` (fallback)
- Log file path resolution precedence:
  1) `PYWALFOX_LOG`
  2) XDG state dir as above
  3) stderr fallback

## Limitations

- Linux/Unix only (uses Unix domain sockets and Linux-specific paths).
- Only user-scope manifest install/uninstall is implemented (no system-wide option yet).
- No authentication/authorization on the socket; any local process can connect and send commands. Socket permissions are set to 0600; prefer `$XDG_RUNTIME_DIR` for best isolation.
- No Windows/macOS support.

## Development

- Format/lint: standard Rust tooling (rustfmt, clippy).
- Tests: basic unit tests included; CI runs fmt, clippy, and tests on push/PR.

## License

See LICENSE in this repository.
