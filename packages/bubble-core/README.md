# bubble-core

The core business logic and data models for the Bubbles social network ecosystem. This crate handles database interactions, protocol logic (Nostr/XQ), and AI-driven features.

## Setup & Configuration

### AI Auto-Labeling (Fact Checking)

This library includes an `ai_labeler` module that uses OpenRouter to verify posts for misinformation. To enable this feature, you must set the following environment variable:

```bash
export OPENROUTER_API_KEY="sk-or-..."
```

Without this key, the `AutoLabeler` will fail to initialize.

## Features

- **Nostr SDK Integration**: Built-in support for Nostr protocols.
- **Fact Checking**: Logic for verifying reliable sources.
- **AI Labeler**: Automated misinformation detection using SLMs (Llama 3, Mistral) via OpenRouter.
