# L2Shablya ⚔️ 🇺🇦

<div align="center">
    <img src="shablya.svg" alt="shablya" width="250"/>
</div>
*Inspired by the swift, light, and deadly Kozak weapon, L2Shablya embodies precision and speed in every line of code.*

**Disclaimer**: *The name "Shablya" reflects the project's focus on performance and elegance, paying homage to the
iconic Kozak weapon. It is used here metaphorically and holds no association with real-world conflict or violence.*

Current state: `Under development`

Ready features:

- [x] Login server
- [ ] Game server

## The login process

```mermaid
sequenceDiagram
    participant Client
    participant Login Server
    participant Game Server

    Client->>Login Server: Establish TCP connection (port 2106)
    Login Server->>Client: Send InitPacket
    Client->>Login Server: RequestAuthGG
    Login Server->>Client: AuthGGOk(session_id)
    Client->>Login Server: RequestAuthLogin
    Login Server->>Game Server: RequestChars
    Game Server->>Login Server: ReplyChars
    Login Server->>Client: LoginOk
    Client->>Login Server: RequestServerList
    Login Server->>Client: ServerList

```

## Game server connection process

```mermaid
sequenceDiagram
    participant Login Server
    participant Game Server

    Game Server->>Login Server: Establish TCP connection
    Client->>Login Server: RequestServerList
```

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
cargo run --bin login
```

- run game server

```bash
cargo run --bin game
```
