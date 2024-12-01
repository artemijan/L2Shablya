# L2Shablya ⚔️  

*Inspired by the swift, light, and deadly Kozak weapon, L2Shablya embodies precision and speed in every line of code.*

**Disclaimer**: *The name "Shablya" reflects the project's focus on performance and elegance, paying homage to the iconic Kozak weapon. It is used here metaphorically and holds no association with real-world conflict or violence.*

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