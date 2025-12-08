# Bubble - Decentralized TUI Nostr Client

Bubble is a terminal-based Nostr client built in Rust, focusing on misinformation prevention through Web of Trust (WoT) and decentralized AI fact-checking.

## Features
- **TUI Interface**: Lightweight, efficient terminal UI.
- **Web of Trust (WoT)**: Calculates and displays trust scores based on your social graph.
- **Decentralized Fact-Checking**: Integrates with AI bots via NIP-32 labels to warn about potential misinformation.
- **Separated Architecture**: Core logic (`bubble-core`) is decoupled from UI (`bubble-tui` integrated in root), enabling future GUI expansion.

## Components
- `src/`: TUI Application (Root crate)
- `bubble-core/`: Core logic library (Nostr client, DB, WoT)
- `bubble-sim/`: Bot simulator for testing fact-checking

## Getting Started
1. Run the TUI:
   ```bash
   cargo run
   ```
   (Keys are generated in `bubble_keys.json` on first run)

2. Run the Bot Simulator (in another terminal):
   ```bash
   cargo run -p bubble-sim
   ```

## Development Workflow
We follow a Git Flow-like workflow:
- **develop**: Main development branch. All feature branches should branch off from here.
- **main**: Stable releases.
- **Feature Branches**: Create branches like `feature/foo-bar` from `develop` for new work.

## License
MIT
