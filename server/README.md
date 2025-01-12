# Agarust Server

## Init database

```bash
cargo install sqlx-cli
```

```bash
sqlx migrate run --database-url sqlite:agarust_db.sqlite?mode=rwc
```

## Run

```bash
cargo run
```
