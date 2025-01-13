# Agarust

_Agarust_ is a multiplayer online game powered by Godot 4 and Rust 🤖🦀 inspired by agar.io

Play now: [jerryshell.itch.io/agarust](https://jerryshell.itch.io/agarust)

## Setup server

```bash
cd server
```

### Init database

```bash
cargo install sqlx-cli
```

```bash
sqlx migrate run --database-url sqlite:agarust_db.sqlite?mode=rwc
```

### Run

```bash
cargo run
```

## Setup client

Open the `client` folder using [Godot 4](https://godotengine.org)

### Change server url

Change `debug_server_url` and `release_server_url` in [client/global/global.gd](client/global/global.gd)
