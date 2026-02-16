# flappy-tui

A Flappy Bird clone that runs in your terminal, with pixel graphics and sound.

![flappy-tui demo](https://raw.githubusercontent.com/NiltonVolpato/flappy-tui/main/assets/the-bird-is-the-word.gif)

## Controls

| Key | Action |
|---|---|
| `Space` / `Up` / `Enter` | Flap |
| `q` / `Esc` | Quit |

## Install

```
cargo install flappy-tui
```

## Run

```
flappy-tui
```

### Environment variables

| Variable | Description |
|---|---|
| `FLAPPY_SEED` | Force a specific RNG seed for reproducible pipe layouts |

## Build from source

```
cargo build --release
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT License](LICENSE-MIT) at your option.
