# Agarust

_Agarust_ is a multiplayer online game powered by Godot 4 and Rust 🤖🦀 inspired by agar.io

Play now on itch.io: [jerryshell.itch.io/agarust](https://jerryshell.itch.io/agarust)

- Use the mouse to control the direction of movement
- Press the left mouse button to sprint, sprinting will consume 20% of the mass, the mass of the player is too small to sprint
- Mass difference is more than 1.5 times to eat another player
- Mass and radius conversion formula: `Mass = PI * Radius * Radius`

## Tech stack

- Godot 4
- Rust (Tokio asynchronous runtime)
- Protocol Buffers
- WebScoket
- SQLite

## Setup server

```bash
cd server
```

### Init database

```bash
cargo install sqlx-cli
```

```bash
sqlx migrate run --database-url "sqlite:agarust_db.sqlite?mode=rwc"
```

### Run

```bash
cargo run
```

## Setup client

Import the `client` folder using [Godot 4](https://godotengine.org)

### Change server url

Change `debug_server_url` and `release_server_url` in [client/global/global.gd](client/global/global.gd)
