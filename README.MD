# L2Shablya ⚔️ 🇺🇦

![Shablya](shablya.svg)

*Inspired by the swift, light, and deadly Kozak weapon, L2Shablya embodies precision and speed in every line of code.*

**Disclaimer**: *The name "Shablya" reflects the project's focus on performance and elegance, paying homage to the
iconic Kozak weapon. It is used here metaphorically and holds no association with real-world conflict or violence.*

Current state: `Under development`

About: This implementation is based on L2J, e.g. I reimplement everything in rust from java + I optimize things,
like DB denormalization for performance boost, removing unnecessary operations and so on.

## Status

- [![codecov](https://codecov.io/gh/artemijan/L2Shablya/branch/master/graph/badge.svg)](https://codecov.io/gh/artemijan/L2Shablya) 
- [![Builds and tests](https://github.com/artemijan/L2Shablya/actions/workflows/rust.yml/badge.svg)](https://github.com/artemijan/L2Shablya/actions/workflows/rust.yml)

Always remember "Having tests opens doors to refactoring! More tests - more doors are open."


## Ready features:

- [x] Login server
    - [x] Login process + auto create accounts
    - [x] Game server registration
    - [x] Kicking player
    - [x] CMD login
    - [ ] EBPF filtering of banned IPs
- [x] Game server
    - [x] register o login server
    - [x] player packet encryption support (shifting key)
    - [x] player can select the game server
    - [x] player can create char
    - [x] player can delete char
    - [x] player can restore char
    - [x] player can select char
    - [x] player can enter the game world
    - [ ] player can move
    - [ ] player can select target
    - [ ] player can send chat messages
    - ..... a lot more

## The client (Interlude Classic)

I don't use an old client of Interlude because graphics there is pretty old.\

[Here](https://drive.google.com/file/d/1lL3Gv9p4v2yGGSiqRxShXS0xWU2LXcq8/view?usp=share_link) is the google drive link where
you can download the client. To play, you have to change IP in L2.ini file.

NOTE: if you disable CMD login in client, and try to run game from windows bash script it will not send "CreateChar" packet in lobby.

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
