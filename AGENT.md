# AGENT Instructions

## Project Overview
Bubble is a rust-based TUI Nostr client with WoT and Fact-Checking features.

## Development Rules
1. **Branching Strategy**:
   - Always base new work on the `develop` branch.
   - Create feature branches named `feature/description` or `fix/issue-name`.
   - Never push directly to `main` unless it's a hotfix or release.

2. **Code Style**:
   - Use standard Rust formatting (`cargo fmt`).
   - Ensure clean separation between `bubble-core` (logic) and `src` (UI).

3. **Key Components**:
   - `bubble-core`: Handles all Nostr protocol interactions, Database, and Trust logic.
   - `src` (TUI): Handles Rendering and User Input. **Do not put business logic here.**
   - `bubble-sim`: Use this to verify Bot interactions.

4. **Testing**:
   - Verify TUI changes by running `cargo run`.
   - Verify Logic changes by running tests in `bubble-core`.
