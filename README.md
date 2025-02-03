# Agarust

_Agarust_ is a server-authoritative real-time multiplayer online game powered by Godot 4 and Rust 🤖🦀 inspired by agar.io

Play now on itch.io: [jerryshell.itch.io/agarust](https://jerryshell.itch.io/agarust)

- Use the mouse to control the direction of movement
- Press the left mouse button to sprint
  - Sprinting drops 20% of your mass
  - Players with too little mass can't sprint
- You can only eat another player if the difference in mass is greater than 1.2 times
- The player's mass will slowly drop over time, the higher the mass, the higher the chance of dropping
- The formula for converting mass to radius: `Mass = PI * Radius * Radius`

## Tech stack

- Godot 4
- Rust
- Protocol Buffers
- WebScoket
- SQLite

## Setup server

```bash
cd server
```

### Init database

**Note**: You **MUST** initialise the database before you can compile the source code, for more details see: [sqlx - Compile-time verification](https://github.com/launchbadge/sqlx?tab=readme-ov-file#compile-time-verification)

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

### Change server URL

Change `debug_server_url` and `release_server_url` in [client/global/global.gd](client/global/global.gd)

## Credits

- [Godot 4 + Golang MMO Tutorial Series by Tristan Batchler](https://www.tbat.me/projects/godot-golang-mmo-tutorial-series)
- [Actors with Tokio by Alice Ryhl](https://draft.ryhl.io/blog/actors-with-tokio)
