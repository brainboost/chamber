# AGENTS.md - AI Agent Guide

**Quick reference for AI agents (Claude, Copilot, etc.) working on the Chamber project.**

## Project Snapshot

```
Name: Chamber
Type: Multi-model AI application with human-in-the-loop workflows
Stack: Tauri 2.0 + SvelteKit + Rust + Python (LangGraph + FastAPI)
Git: Worktree-based development workflow
```

## Quick Start

### 1. Understand the Architecture
```
SvelteKit UI → Tauri Commands (Rust) → Python Sidecar (LangGraph) → LLMs
     ↓                    ↓                           ↓
  Dexie (IndexedDB)  File System  State Machine (Checkpointing)
```

### 2. Worktree Workflow (CRITICAL)
```bash
# ALWAYS use worktrees for development
chamber/         [main]  ← Run dev servers here (never stop)
chamber-dev/     [dev]   ← Develop features here

# Check worktrees
git worktree list

# Create new worktree
git worktree add ../chamber-feature -b feature/my-feature
```

### 3. Key Commands
```bash
npm run dev          # SvelteKit dev server (localhost:5173)
npm run tauri:dev    # Tauri desktop app
npm run check        # Type check SvelteKit
npm run build        # Production build

cd python-sidecar
python -m chamber.main --host 127.0.0.1 --port 8765  # Run Python sidecar
pytest              # Run Python tests

cd src-tauri
cargo test          # Run Rust tests
cargo clippy        # Lint Rust code
```

## File Structure

```
chamber/
├── src/                          # SvelteKit frontend
│   ├── lib/
│   │   ├── components/          # UI components
│   │   │   ├── chat/            # Chat UI components
│   │   │   ├── layout/          # Header, Sidebar
│   │   │   └── ui/              # Button, Card, Dialog, etc.
│   │   ├── stores/              # Svelte state stores
│   │   ├── services/            # API/business logic
│   │   ├── types/               # TypeScript definitions
│   │   └── db/                  # Dexie schema
│   └── routes/                  # SvelteKit pages
├── src-tauri/                   # Rust backend
│   └── src/
│       ├── commands/            # Tauri command handlers
│       ├── services/            # Business logic
│       ├── models/              # Data structures
│       └── utils/               # Utilities
├── python-sidecar/              # Python backend
│   └── chamber/
│       ├── graph/               # LangGraph workflows
│       ├── models/              # LLM providers
│       ├── tools/               # ReAct tools
│       ├── server/              # FastAPI + WebSocket
│       └── state/               # State management
└── workspace/                   # User data (gitignored except config)
    ├── sessions/                # Session markdown files
    └── config/                  # Configuration YAML
```

## Important Patterns

### Adding LLM Providers
1. Create `python-sidecar/chamber/models/{provider}.py`
2. Inherit from `BaseModel`
3. Implement required methods
4. Register in `python-sidecar/chamber/models/__init__.py`

### Adding Tools
1. Create `python-sidecar/chamber/tools/{tool}.py`
2. Inherit from `BaseTool`
3. Implement `execute()` method
4. Register in `python-sidecar/chamber/tools/__init__.py`

### Adding UI Components
1. Add to `src/lib/components/{category}/`
2. Use TailwindCSS
3. Follow existing patterns
4. Export from index if needed

### Adding Tauri Commands
1. Create `src-tauri/src/commands/{name}.rs`
2. Register in `src-tauri/src/commands/mod.rs`
3. Expose in `src-tauri/src/lib.rs` with `#[tauri::command]`
4. Update TypeScript types

## Code Style

### TypeScript/Svelte
- Use strict TypeScript
- Follow existing component patterns
- TailwindCSS for styling
- Keep components focused

### Rust
- Use `cargo clippy`
- Handle errors with `Result<>`
- Use `?` operator
- Follow naming conventions

### Python
- PEP 8 style
- Use type hints
- Document complex functions
- Handle exceptions

## Critical Rules

1. **NEVER commit `.env` files** (already gitignored)
2. **ALWAYS use worktrees** for development
3. **Test before merging** to main
4. **Keep main stable** - develop in dev/feature branches
5. **Follow existing patterns** - consistency matters
6. **Read before editing** - understand the codebase

## Before Making Changes

1. ✅ Read the relevant files
2. ✅ Check for existing patterns
3. ✅ Use worktrees (don't stop dev servers)
4. ✅ Run tests after changes
5. ✅ Type check: `npm run check`
6. ✅ Commit with descriptive messages

## Troubleshooting

### Build Issues
```bash
# Clear dependencies
rm -rf node_modules && npm install

# Clear build artifacts
rm -rf build .svelte-kit

# Check versions
node --version  # Should be 18+
rustc --version # Should be 1.70+
python --version # Should be 3.10+
```

### Git Issues
```bash
git worktree prune    # Clean stale references
git worktree list     # See all worktrees
pwd                   # Check which worktree you're in
```

### Runtime Issues
- Check `.env` file has required keys
- Check `workspace/config/chamber-config.yaml`
- Verify Python dependencies: `cd python-sidecar && pip install -e .`

## Memory Files

- **AGENTS.md** (this file) - Quick reference for AI agents
- **MEMORY.md** - Detailed project memory
- **GIT_WORKFLOWS.md** - Git worktree workflows
- **CLAUDE.md** - Claude Code specific guidance
- **README.md** - User documentation

## Quick Decision Tree

```
Task Type → Action
-----------┬───────────────────────────────────────
Small fix  → Edit file, test, commit
Feature    → Use worktree, plan mode if complex
Refactor   → Plan mode, understand impact first
Experiment → Create experiment worktree
Docs       → Edit documentation, update memory
Hotfix     → Use main worktree, test thoroughly
```

## When to Ask for Help

- 🤔 Unsure about architecture
- 🤔 Task needs significant refactoring
- 🤔 Working on critical paths (auth, data)
- 🤔 Changes affect multiple subsystems
- 🤔 Can't find existing pattern

Otherwise: Use existing patterns, make reasonable decisions, test well.

## Summary

**You're working on a multi-model AI app with:**
- Frontend: SvelteKit + Tauri
- Backend: Rust + Python (LangGraph)
- Git: Worktree workflow
- Storage: Dexie + Markdown files

**Remember:**
- Use worktrees for development
- Follow existing patterns
- Test before committing
- Keep documentation updated

**Good luck! 🚀**
