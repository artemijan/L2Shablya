# L2Shablya ⚔️ 🇺🇦

![Shablya](shablya.svg)

*Inspired by the swift, light, and deadly Kozak weapon, L2Shablya embodies precision and speed in every line of code.*

**Disclaimer**: *The name "Shablya" reflects the project's focus on performance and elegance, paying homage to the
iconic Kozak weapon. It is used here metaphorically and holds no association with real-world conflict or violence.*

Current state: `Under development`

Ready features:

- [x] Login server
    - [x] Login process + auto create accounts
    - [x] Game server registration
    - [x] Kicking player
    - [ ] CMD login
    - [ ] EBPF filtering of banned IPs
- [x] Game server
    - [x] register o login server
    - [x] player can select game server
    - [ ] player can create char
    - [ ] player can delete char
    - [ ] player can enter the game world

## The client

[Here](https://drive.google.com/file/d/1sXMoENX7YuhqKTYSjJN8RfHMgJ6Yeu6s/view?usp=share_link) is google drive link where
you can download client. In order to play, you have to change ip in L2.ini file.

## Local deployment

- create .env file

```bash
DATABASE_URL=sqlite://local.sqlite?mode=rwc
```

- install cargo and rust
- install sea-orm-cli

```bash
cargo install sea-orm-cli
```

- run migrations

```bash
sea-orm-cli migrate up
```

- run login server

```bahs
cargo run -p login
```

- run game server

```bash
cargo run -p game
```
