# Claude Code Project Guide

This file contains project-specific guidance for Claude Code when working on the Chamber project.

## Project Context

**Chamber** is a multi-model AI application using Tauri 2.0 + SvelteKit + Rust + Python (LangGraph). It enables human-in-the-loop AI workflows with multiple LLM providers.

**Package Managers:**
- Node.js: npm
- Python: **uv** (fast Python package installer, not pip)
- Rust: cargo

## Quick Reference

- **Stack**: Tauri 2.0, SvelteKit, Rust, Python (FastAPI + LangGraph)
- **Package manager**: npm (Node.js), pip (Python), cargo (Rust)
- **Git workflow**: Worktree-based (see GIT_WORKFLOWS.md)
- **Testing**: pytest (Python), cargo test (Rust), npm test (JS)
- **Linting**: cargo clippy (Rust), svelte-check (Svelte)

## When to Use Different Approaches

### Use Worktrees For:
- Developing features while dev servers are running
- Testing PRs without switching branches
- Isolating experimental changes
- Parallel development workflows

### Use Regular Branches For:
- Quick fixes in the same worktree
- Small changes that don't require isolation

### Use Plan Mode For:
- New feature implementation
- Multi-file refactoring
- Architecture changes
- Complex bug fixes

### Skip Plan Mode For:
- Single-line fixes
- Obvious bug fixes
- Simple additions
- Documentation updates

## File Organization Patterns

### Frontend (SvelteKit)
```
src/lib/components/
├── chat/          # Chat-related components
├── layout/        # Layout components (Header, Sidebar)
└── ui/            # Reusable UI components (Button, Card, etc.)

src/lib/stores/    # Svelte stores for state management
src/lib/services/  # Service layer (API calls, business logic)
src/lib/types/     # TypeScript type definitions
```

### Backend (Rust)
```
src-tauri/src/
├── commands/      # Tauri command handlers
├── services/      # Business logic
├── models/        # Data structures
└── utils/         # Utility functions
```

### Python Sidecar
```
python-sidecar/chamber/
├── graph/         # LangGraph workflows
├── models/        # LLM provider implementations
├── tools/         # ReAct tool implementations
├── server/        # FastAPI + WebSocket server
└── state/         # State management and checkpointing
```

## Common Tasks

### Adding a New LLM Provider
1. Create file in `python-sidecar/chamber/models/{provider}.py`
2. Inherit from `BaseModel` class
3. Implement required methods
4. Register in `python-sidecar/chamber/models/__init__.py`
5. Update `workspace/config/chamber-config.yaml`

### Adding a New Tool
1. Create file in `python-sidecar/chamber/tools/{tool}.py`
2. Inherit from `BaseTool` class
3. Implement `execute()` method
4. Register in `python-sidecar/chamber/tools/__init__.py`
5. Add configuration if needed

### Adding UI Components
1. Add to appropriate `src/lib/components/` subdirectory
2. Use TailwindCSS for styling
3. Follow existing component patterns
4. Export from `src/lib/components/index.ts` if needed

### Adding Tauri Commands
1. Create command in `src-tauri/src/commands/{name}.rs`
2. Register in `src-tauri/src/commands/mod.rs`
3. Expose in `src-tauri/src/lib.rs` with `#[tauri::command]`
4. Update TypeScript types in `src/lib/types/`

## Development Workflow

### Making Changes
1. Use worktree workflow: `cd ../chamber-dev/` for development
2. Make changes and test locally
3. Run type checking: `npm run check`
4. Run tests: `npm test`, `cargo test`, or `pytest`
5. Commit with descriptive message

### Testing Changes
```bash
# Frontend
npm run dev          # Dev server
npm run build        # Production build
npm run check        # Type check

# Tauri
npm run tauri:dev    # Development build
npm run tauri:build  # Production build

# Python (using uv)
cd python-sidecar
uv sync                           # Install dependencies
uv run python -m chamber.main     # Run server
uv run pytest                     # Run tests

# Rust
cd src-tauri
cargo test
cargo clippy
```

### Before Committing
1. Run all tests
2. Check for linting errors
3. Ensure code follows existing patterns
4. Test in the actual application if possible

## Code Style

### TypeScript/Svelte
- Use TypeScript strict mode
- Follow existing component patterns
- Use TailwindCSS for styling
- Keep components small and focused

### Rust
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Handle errors properly with `Result<>`
- Use `?` operator for error propagation

### Python
- Follow PEP 8 style guide
- Use type hints where appropriate
- Document complex functions
- Handle exceptions gracefully

## Important Notes

- **Never commit** `.env` files (already gitignored)
- **Always use worktrees** for development to avoid stopping dev servers
- **Test thoroughly** before merging to main branch
- **Keep main branch stable** - develop in dev or feature branches
- **Follow existing patterns** - consistency is key
- **Document changes** - update relevant documentation

## Git Worktree Reminders

```bash
# List worktrees
git worktree list

# Create new worktree
git worktree add ../chamber-feature -b feature/my-feature

# Remove worktree
git worktree remove ../chamber-feature

# Current structure
chamber/         [main]  ← Stable, run dev servers here
chamber-dev/     [dev]   ← Active development
```

## Getting Unstuck

### Build Errors
- Check Node.js version (18+)
- Check Rust version (1.70+)
- Check Python version (3.10+)
- Clear node_modules: `rm -rf node_modules && npm install`
- Clear Python venv: `cd python-sidecar && rm -rf .venv && uv sync`
- Clear build artifacts: `rm -rf build .svelte-kit`

### Runtime Errors
- Check Python dependencies: `cd python-sidecar && uv sync`
- Check environment variables in `.env`
- Check configuration in `workspace/config/chamber-config.yaml`
- Ensure uv is installed: `uv --version`

### Git Issues
- Use `git worktree prune` to clean stale references
- Use `git worktree list` to see all worktrees
- Check you're in the correct worktree directory

## Project-Specific Commands

```bash
# Quick development setup
npm run dev &           # SvelteKit dev server
npm run tauri:dev       # Tauri desktop app

# Full rebuild
npm run build
npm run tauri:build

# Testing
npm test                # Frontend tests
cd src-tauri && cargo test    # Rust tests
cd python-sidecar && pytest   # Python tests

# Type checking
npm run check
npm run check:watch
```

## Memory Files

- **MEMORY.md** - Project overview and quick reference
- **GIT_WORKFLOWS.md** - Comprehensive git and worktree workflows
- **README.md** - User-facing documentation
- **IMPLEMENTATION_STATUS.md** - Feature implementation status
- **PHASE_5_SUMMARY.md** - Development phase summary

## Ask for Help When

- You're unsure about architectural decisions
- A task seems to require significant refactoring
- You need to understand existing code patterns
- You're working on critical paths (auth, data persistence)
- Changes affect multiple subsystems

Otherwise, use existing patterns and make reasonable decisions!
