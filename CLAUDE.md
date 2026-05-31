# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

L2Shablya is a Rust reimplementation of the L2J Lineage 2 game server (Interlude Classic client). It consists of a login server and game server that communicate over TCP using custom packet protocols with Blowfish encryption.

## Workspace Structure

The workspace has 6 members:
- `entities` - SeaORM database entities and DAOs
- `game` - Game server implementation
- `login` - Login server implementation
- `l2-core` - Shared core functionality (crypto, config, network, game objects)
- `macro-common` - Custom procedural macros for config loading and packet handling
- `test-utils` - Test utilities

## Key Commands

### Setup
```bash
# Install sea-orm-cli for migrations
cargo install sea-orm-cli

# Run migrations
sea-orm-cli migrate up
# or
cargo run -p migration

# View migration status
cargo run -p migration status
```

### Running Servers
```bash
# Login server
cargo run -p login

# Game server
cargo run -p game
```

### Testing
```bash
# Run all tests
cargo test --workspace --exclude macro-common --exclude migration

# Run tests for a specific package
cargo test -p game
cargo test -p login
cargo test -p entities

# Run tests with coverage (using tarpaulin)
cargo tarpaulin --workspace --exclude test-utils --exclude macro-common --exclude migration --out Lcov
```

### Linting
```bash
cargo clippy --workspace
cargo fmt --check
```

## Configuration

Configuration files are loaded from the `L2_CONFIG` environment variable (defaults to `./`):

- `config/game.yaml` - Game server configuration (port 7777, login server on 9014)
- `config/login.yaml` - Login server configuration (port 2106, game servers on 9014)
- `config/schema/*.yml` - Game data schemas (skill trees, character templates, etc.)
- `config/data/*.yaml` - Game data files (action lists, skill trees, stats)

Database URL is configured via `DATABASE_URL` environment variable (e.g., `sqlite://local.sqlite?mode=rwc`).

## Architecture

### Packet Architecture
- Packets are defined as tuple structs with a single field
- Use `#[derive(PacketEnum)]` for packet enums to auto-generate `accept()` method
- Use `#[derive(SendablePacket)]` for packet types to implement `SendablePacket` trait
- Packets are organized by direction: `from_client/` and `to_client/`

### Config Loading
- Use `#[config_file(path = "...")]` attribute for single config files
- Use `#[config_dir(path = "...")]` attribute for config directories
- Configs implement `ConfigFileLoader` or `ConfigDirLoader` traits
- Configs support `post_load_message` for logging during load

### Network Layer
- Uses `kameo` actor model for concurrent packet handling
- `ConnectionListener` - TCP server for incoming connections
- `Connector` - TCP client for connecting to other servers
- Players are represented as `PlayerClient` actors

### Database
- Uses SeaORM for database access
- Entities are in `entities/src/entities/`
- DAOs are in `entities/src/dao/`
- Test factories available via `test-factories` feature flag

## Development Notes

- Test factories must be enabled explicitly via `#[cfg(feature = "test-factories")]`
- Login server runs on port 2106, game server on port 7777
- Both servers connect to login server on port 9014
- Blowfish key is configured in both `game.yaml` and `login.yaml`
