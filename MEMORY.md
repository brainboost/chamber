# Chamber Project Memory

This file contains important project information and workflows for Claude Code to reference across sessions.

## Project Overview

**Chamber** is a multi-model AI application with human-in-the-loop workflows.

- **Stack**: Tauri 2.0 + SvelteKit + Rust + Python (LangGraph + FastAPI)
- **Purpose**: Multi-model AI orchestration with human oversight
- **Repository**: Git with worktree-based development workflow

## Git Worktree Setup

This project uses **git worktrees** for parallel development. This allows running long-lived dev servers while developing features.

### Active Worktrees

```
D:/Projects/chamber     [main]  ← Stable production code
D:/Projects/chamber-dev [dev]   ← Active development
```

### Worktree Workflow

**1. Primary Development Pattern**
```bash
# Keep dev servers running on main (chamber/)
cd chamber/
npm run tauri:dev  # Long-lived, don't stop

# Develop features in worktree (chamber-dev/)
cd ../chamber-dev
# Make changes, test, commit
```

**2. Creating Feature Branches**
```bash
# From dev worktree
cd ../chamber-dev
git checkout -b feature/my-feature
# Work on feature
git add . && git commit -m "Add feature"

# Merge to dev when ready
git checkout dev
git merge feature/my-feature
```

**3. Merging to Main**
```bash
# From main worktree
cd chamber/
git merge dev
# Test thoroughly, then push
```

**4. Creating Additional Worktrees**
```bash
# For experiments, PRs, or parallel work
git worktree add ../chamber-experiment experiment-branch
git worktree add ../chamber-staging staging
```

### Worktree Management Commands

```bash
# List all worktrees
git worktree list

# Move a worktree
git worktree move ../old-path ../new-path

# Remove a worktree (use with care!)
git worktree remove ../worktree-path

# Clean up stale references
git worktree prune
```

## Development Commands

### Frontend (SvelteKit)
```bash
npm run dev          # Dev server (localhost:5173)
npm run build        # Production build
npm run check        # Type checking
npm run check:watch  # Type checking with watch mode
```

### Desktop App (Tauri)
```bash
npm run tauri:dev    # Development desktop app
npm run tauri:build  # Build production binaries
```

### Python Sidecar
```bash
cd python-sidecar
uv sync                             # Install dependencies (using uv)
uv run python -m chamber.main --host 127.0.0.1 --port 8765  # Run standalone
uv run pytest                       # Run tests
```

### Rust Backend
```bash
cd src-tauri
cargo test          # Run tests
cargo build         # Build
cargo clippy        # Lint
```

## Project Structure

```
chamber/
├── src/                  # SvelteKit frontend
│   ├── lib/
│   │   ├── components/  # UI components (chat, layout, ui)
│   │   ├── stores/      # Svelte stores (chat, config, session, ui)
│   │   ├── services/    # Service layer (indexeddb, session-manager, tauri)
│   │   ├── types/       # TypeScript types
│   │   └── db/          # Dexie schema
│   └── routes/          # SvelteKit pages
├── src-tauri/           # Rust backend
│   └── src/
│       ├── commands/    # Tauri commands (config, session, sidecar, workspace)
│       ├── services/    # Business logic
│       └── models/      # Data structures
├── python-sidecar/      # Python LangGraph backend
│   └── chamber/
│       ├── graph/       # LangGraph workflows
│       ├── models/      # LLM providers (anthropic, gemini, ollama, xai)
│       ├── tools/       # ReAct tools (calculator, file_ops, web_search)
│       └── server/      # FastAPI + WebSocket
└── workspace/           # User workspace
    ├── sessions/        # Session MD files
    └── config/          # Configuration (chamber-config.yaml)
```

## Configuration

### Environment Variables
- `.env` file in root directory
- Required: `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`, `XAI_API_KEY`, `OLLAMA_BASE_URL`
- Note: This project uses `uv` for Python dependency management (not pip)

### App Configuration
- Located at: `workspace/config/chamber-config.yaml`
- Controls: Models, tools, workspace paths, approval settings

## Key Patterns

### LLM Providers
- **Anthropic**: Primary LLM provider
- **Google Gemini**: Secondary LLM
- **Ollama**: Local LLM support
- **x.ai**: Grok model integration

### Architecture Flow
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

## Common Tasks

### Adding a New LLM Provider
1. Create provider file in `python-sidecar/chamber/models/`
2. Inherit from `BaseModel` class
3. Register in `python-sidecar/chamber/models/__init__.py`
4. Add configuration to `workspace/config/chamber-config.yaml`

### Adding a New Tool
1. Create tool file in `python-sidecar/chamber/tools/`
2. Inherit from `BaseTool` class
3. Implement `execute()` method
4. Register in `python-sidecar/chamber/tools/__init__.py`

### Creating New UI Components
1. Add to `src/lib/components/` (organized by category)
2. Export from `src/lib/components/index.ts` if needed
3. Use TailwindCSS for styling
4. Follow existing component patterns

## Important Notes

- **Always use worktrees** for development to avoid stopping dev servers
- **Never commit** `.env` files (already in .gitignore)
- **Test thoroughly** before merging to main
- **Keep workspace clean** - sessions and config are gitignored except for chamber-config.yaml
- **Rust requires Rust 1.70+** and **Node.js 18+**
- **Python requires 3.10+**

## Session Files

For detailed documentation, see:
- `AGENTS.md` - Quick reference guide for AI agents
- `GIT_WORKFLOWS.md` - Comprehensive git and worktree workflows
- `CLAUDE.md` - Claude Code-specific project guidance
- `README.md` - Project overview and setup
- `IMPLEMENTATION_STATUS.md` - Feature implementation status
- `PHASE_5_SUMMARY.md` - Development phase summary
