# Chamber

Multi-Model AI Application with Human-in-the-Loop Workflows

## Overview

Chamber is a professional AI application that enables multi-model reasoning with human-in-the-loop workflows. Multiple LLM models collaborate on complex tasks, with one acting as orchestrator to synthesize decisions and advance progress.

## Tech Stack

- **Frontend**: Tauri 2.0 + SvelteKit (SPA) + TailwindCSS
- **Backend**: Rust (Tauri commands) + Python sidecar (LangGraph + FastAPI)
- **Storage**: IndexedDB (Dexie) + Markdown files (configurable workspace)
- **LLM Providers**: Anthropic, Google Gemini, Ollama, x.ai

## Features

- ✨ Multi-step reasoning with ReAct pattern
- 🤝 Multi-model chamber (parallel reasoning → orchestrator synthesis)
- 👤 Human-in-the-loop approval for tool usage
- ⏸️ Pause/resume capability with state persistence
- ⚙️ Configuration-driven (no code changes needed)
- 🔐 **Secure OAuth authentication** - Sign in with Claude/Google accounts instead of API keys
- 🔑 **Keychain storage** - Credentials stored securely in platform keychain

## Authentication

Chamber supports multiple authentication methods for LLM providers:

### OAuth Authentication (Recommended)

**Supported Providers:**
- Anthropic Claude (Sign in with Claude account)
- Google Gemini (Sign in with Google account)

**Benefits:**
- No need to manage API keys manually
- Credentials stored securely in your system keychain
- Automatic token refresh in the background
- No plain text credentials in config files

**How to Use:**
1. Open Chamber and go to **Settings**
2. Find the **Authentication** section
3. Click "Connect with OAuth" for your provider
4. Complete the sign-in flow in the authorization window
5. Your credentials are now stored securely!

### API Key Authentication

You can still use traditional API keys if you prefer:

**In Settings:**
1. Click "Use API Key" button for your provider
2. Paste your API key
3. Click Save

**Via Environment Variables:**
Create a `.env` file in the `python-sidecar` directory:

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Python 3.10+
- Tauri CLI

### Installation

```bash
# Install dependencies
npm install

# Install Python dependencies (using uv)
cd python-sidecar
uv sync
cd ..

# Setup environment variables
cp .env.example .env
# Add your API keys to .env
```

### Environment Variables

Create a `.env` file in the `python-sidecar` directory:

```env
ANTHROPIC_API_KEY=your_key_here
GOOGLE_API_KEY=your_key_here
XAI_API_KEY=your_key_here
OLLAMA_BASE_URL=http://localhost:11434
```

## Development

### Git Worktree Workflow

This project uses **git worktrees** for parallel development, allowing you to run long-lived dev servers while developing features.

**Current worktree structure:**
```
D:/Projects/chamber     [main]  ← Production-ready code
D:/Projects/chamber-dev [dev]   ← Active development
```

**Basic workflow:**
```bash
# Terminal 1: Main worktree - Run dev servers (keeps running!)
cd chamber/
npm run tauri:dev

# Terminal 2: Dev worktree - Develop features
cd ../chamber-dev/
# Make changes, commit, test without stopping the dev server
```

**Create additional worktrees:**
```bash
# For experiments, PRs, or parallel work
git worktree add ../chamber-experiment -b experiment-branch
git worktree add ../chamber-staging staging

# See all worktrees
git worktree list

# Remove when done
git worktree remove ../chamber-experiment
```

For detailed git workflows, see [GIT_WORKFLOWS.md](GIT_WORKFLOWS.md).

## Security

### OAuth Security

- **PKCE Flow**: Uses Proof Key for Code Exchange (PKCE) for secure OAuth authorization
- **Platform Keychain**: Credentials stored using OS keychain services:
  - **Windows**: Credential Manager
  - **macOS**: Keychain Access
  - **Linux**: Secret Service API (libsecret)
- **TLS Only**: All OAuth communication over HTTPS
- **Token Refresh**: Automatic background refresh before token expiry
- **Minimal Scopes**: Only requests necessary OAuth scopes

### Token Management

- OAuth tokens are automatically refreshed 5 minutes before expiry
- Background refresh task runs every 5 minutes
- No manual token management required
- Tokens are never written to disk in plain text

### API Key Security (if used)

If using API keys instead of OAuth:
- Store in `.env` file (already gitignored)
- Never commit `.env` to version control
- Consider using environment variable injection in production

## Troubleshooting

### OAuth Issues

**Authorization page won't load:**
- Check your internet connection
- Disable VPN temporarily
- Ensure popups are allowed for Chamber
- Try a different browser

**Authorization fails:**
- Make sure you're allowing all requested permissions
- Check that your account has access to the provider's API
- Try clearing your browser cache and cookies

**Token refresh errors:**
- Sign out and sign in again
- Check that your system keychain is accessible
- Verify your account is still active with the provider

## Platform Compatibility

Chamber's keychain storage is compatible with:

- **Windows 10/11**: Uses Windows Credential Manager
- **macOS 12+**: Uses Keychain Access
- **Linux**: Uses libsecret (gnome-keyring/kwallet)

For detailed platform-specific testing and verification procedures, see [CROSS_PLATFORM_KEYRING.md](CROSS_PLATFORM_KEYRING.md).

### Running the Application

Run the application in development mode:

```bash
# Terminal 1: Start SvelteKit dev server
npm run dev

# Terminal 2: Start Tauri (includes Rust hot reload)
npm run tauri:dev

# Terminal 3 (optional): Run Python sidecar standalone for testing
cd python-sidecar
uv run python -m chamber.main --host 127.0.0.1 --port 8765
```

## Configuration

Configuration is stored in `workspace/config/chamber-config.yaml`. Edit this file to:

- Change orchestrator and reasoning models
- Enable/disable specific models
- Configure tool approval settings
- Adjust workspace paths

## Architecture

```
User Input (SvelteKit UI)
  ↓
Svelte Stores + SessionManager
  ↓
Tauri Commands (Rust)
  ↓
Python Sidecar (FastAPI + LangGraph)
  ↓
Orchestrator → Parallel Reasoning → Synthesis → Tools
  ↓
WebSocket Stream ← Real-time Updates
  ↓
UI Updates (Reactive Svelte)
```

## Testing

```bash
# Frontend tests
npm run test

# Rust tests
cd src-tauri
cargo test

# Python tests
cd python-sidecar
uv run pytest
```

## Building

```bash
# Build for production
npm run build
npm run tauri:build
```

This creates platform-specific binaries in `src-tauri/target/release/bundle/`.

## Project Structure

```
chamber/
├── src/                  # SvelteKit frontend
│   ├── lib/
│   │   ├── components/  # UI components
│   │   ├── stores/      # Svelte stores
│   │   ├── services/    # Service layer
│   │   ├── types/       # TypeScript types
│   │   └── db/          # Dexie schema
│   └── routes/          # SvelteKit pages
├── src-tauri/           # Rust backend
│   └── src/
│       ├── commands/    # Tauri commands
│       ├── services/    # Business logic
│       └── models/      # Data structures
├── python-sidecar/      # Python LangGraph backend
│   └── chamber/
│       ├── graph/       # LangGraph workflows
│       ├── models/      # LLM providers
│       ├── tools/       # ReAct tools
│       └── server/      # FastAPI + WebSocket
└── workspace/           # User workspace
    ├── sessions/        # Session MD files
    └── config/          # Configuration
```

## License

MIT

## Contributing

Contributions welcome! Please read CONTRIBUTING.md for guidelines.
