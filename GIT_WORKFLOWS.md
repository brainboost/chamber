# Git Workflows for Chamber

Complete guide to git operations and worktree management for the Chamber project.

## Table of Contents

1. [Worktree Overview](#worktree-overview)
2. [Basic Worktree Operations](#basic-worktree-operations)
3. [Development Workflows](#development-workflows)
4. [Advanced Patterns](#advanced-patterns)
5. [Troubleshooting](#troubleshooting)
6. [Best Practices](#best-practices)

---

## Worktree Overview

Git worktrees allow you to have multiple working directories checked out from different branches of the same repository.

### Why Use Worktrees?

**Without Worktrees:**
```bash
# You have to stop dev server to switch branches
npm run tauri:dev  # Running...
# User wants to test a feature → must stop server, switch branches, restart
```

**With Worktrees:**
```bash
# Main worktree: Dev server keeps running
chamber/          → npm run tauri:dev  # Never stops!

# Dev worktree: Work on features freely
chamber-dev/      → Make changes, test, commit

# Feature worktree: Test a PR
chamber-feature/  → Checkout and test specific changes
```

### Current Worktree Structure

```
D:/Projects/
├── chamber/         [main]  ← Production-ready code
└── chamber-dev/     [dev]   ← Active development
```

---

## Basic Worktree Operations

### Creating Worktrees

**Create from existing branch:**
```bash
git worktree add ../chamber-staging staging
```

**Create with new branch:**
```bash
git worktree add ../chamber-dev -b dev
```

**Create from specific commit:**
```bash
git worktree add ../chamber-experiment HEAD~1 -b experiment
```

### Listing Worktrees

```bash
git worktree list
# Output:
# D:/Projects/chamber       c06a9fa [main]
# D:/Projects/chamber-dev   c06a9fa [dev]
```

### Removing Worktrees

```bash
# Remove worktree and delete directory
git worktree remove ../chamber-dev

# After manually deleting directory, clean up reference
git worktree prune
```

### Moving Worktrees

```bash
git worktree move ../chamber-dev ../chamber-development
```

---

## Development Workflows

### Workflow 1: Parallel Development

**Use case:** Run dev servers while developing features

```bash
# Terminal 1: Main worktree - Run dev servers
cd chamber/
npm run tauri:dev
# Keep this running forever!

# Terminal 2: Dev worktree - Active development
cd ../chamber-dev/
# Make changes to code
git add .
git commit -m "Add new feature"

# Terminal 3: Test in main worktree
# Changes are isolated - main is unaffected
# When ready, merge dev to main
```

### Workflow 2: Feature Branch Development

**Use case:** Isolate features before merging to dev

```bash
# In dev worktree
cd chamber-dev/
git checkout -b feature/add-user-auth

# Work on feature
# Make changes, commit

# Test thoroughly
npm run build

# Merge to dev when ready
git checkout dev
git merge feature/add-user-auth
git branch -d feature/add-user-auth
```

### Workflow 3: PR Review and Testing

**Use case:** Test someone else's PR without affecting your work

```bash
# Create worktree for PR
git worktree add ../chamber-pr-123 -b pr-123 origin/pr-123

# Test the PR
cd ../chamber-pr-123/
npm install
npm run tauri:dev

# When done, remove worktree
cd ../chamber/
git worktree remove ../chamber-pr-123
```

### Workflow 4: Safe Experiments

**Use case:** Try risky changes without affecting your main work

```bash
# Create experiment worktree
git worktree add ../chamber-experiment -b crazy-experiment

cd ../chamber-experiment/
# Try wild ideas, break things, learn

# If it works: merge to dev
# If it fails: just delete the worktree
cd ../chamber/
git worktree remove ../chamber-experiment
git branch -D crazy-experiment
```

### Workflow 5: Release Management

**Use case:** Prepare release while continuing development

```bash
# Current structure
chamber/           [main]     ← Production
chamber-dev/       [dev]      ← Active development
chamber-release/   [release]  ← Release preparation (NEW)

# Create release worktree
git worktree add ../chamber-release -b release/v1.0.0

# In release worktree
cd ../chamber-release/
# Bump version, update changelog, final tests
# Tag release
git tag -a v1.0.0 -m "Release v1.0.0"

# Merge release to main and dev
cd ../chamber/  # main
git merge release/v1.0.0

cd ../chamber-dev/  # dev
git merge release/v1.0.0
```

---

## Advanced Patterns

### Pattern 1: CI/CD Integration

```bash
# Worktree for CI testing
git worktree add ../build-ci -b ci/test-build

# Run full build and test suite
cd ../build-ci/
npm run build
npm run test
cargo test --manifest-path=src-tauri/Cargo.toml

# Clean up
cd ../chamber/
git worktree remove ../build-ci
```

### Pattern 2: Hotfix Workflow

```bash
# Production issue detected!
cd chamber/  # main
git worktree add ../chamber-hotfix -b hotfix/critical-bug

cd ../chamber-hotfix/
# Fix the bug
git commit -am "Fix critical bug"

# Test and deploy from main
cd ../chamber/
git merge hotfix/critical-bug
npm run tauri:build

# Backport to dev
cd ../chamber-dev/
git merge hotfix/critical-bug
```

### Pattern 3: Documentation Updates

```bash
# Keep docs on separate branch
git worktree add ../chamber-docs -b docs/update-guide

cd ../chamber-docs/
# Update documentation
git commit -am "Update user guide"

# Merge when ready
cd ../chamber/
git merge docs/update-guide
```

### Pattern 4: Dependency Updates

```bash
# Isolate dependency updates
git worktree add ../chamber-deps -b deps/update-dependencies

cd ../chamber-deps/
npm update
cargo update
# Test everything

# If tests pass, merge
# If tests fail, delete worktree and try again
```

---

## Troubleshooting

### Problem: Worktree in weird state

```bash
# Clean up stale references
git worktree prune

# Force remove if directory is gone
git worktree list --porcelain | grep -A1 "worktree ../broken-path"
# Then remove the reference
```

### Problem: Can't checkout branch (already in worktree)

```bash
# See which worktree has the branch
git worktree list

# Either:
# 1. Use that worktree
# 2. Remove that worktree first
git worktree remove ../other-worktree
```

### Problem: Want to see differences between worktrees

```bash
# From any worktree, compare with main
git diff main

# Compare with dev
git diff dev

# See commits in dev but not main
git log main..dev
```

### Problem: Merge conflicts

```bash
# In worktree receiving merge
cd chamber-dev/
git merge main

# Resolve conflicts
# Edit files, resolve markers
git add .
git commit -m "Merge main into dev"
```

---

## Best Practices

### 1. Worktree Naming

```bash
# Good names
../chamber-dev
../chamber-staging
../chamber-feature-auth
../chamber-pr-123

# Avoid names that might conflict with project directories
../dev          # Too generic
../test         # Ambiguous
../tmp          # Not descriptive
```

### 2. Branch Naming

```bash
# Feature branches
feature/add-user-auth
feature/improve-ui
feature/refactor-state

# Fix branches
fix/async-bug
fix/memory-leak
fix/typo-readme

# Release branches
release/v1.0.0
release/v1.1.0

# Hotfix branches
hotfix/critical-security-issue
hotfix/data-loss-bug
```

### 3. Worktree Hygiene

```bash
# Clean up worktrees regularly
git worktree list
# Remove old/experimental worktrees

# Always remove worktrees properly
git worktree remove ../old-worktree
# Don't just delete directories

# Prune stale references
git worktree prune
```

### 4. Commit Discipline

```bash
# Always commit before switching worktrees
cd chamber-dev/
git commit -am "Work in progress"

# Use descriptive commits
git commit -m "Add user authentication with JWT tokens"

# Don't leave uncommitted work across worktrees
# It can get confusing!
```

### 5. Testing Before Merging

```bash
# In feature worktree
cd chamber-feature-auth/
npm run build
npm test
cargo test

# Only merge if tests pass
cd ../chamber-dev/
git merge feature/add-user-auth
```

### 6. Main Branch Protection

```bash
# Keep main stable
# Don't develop directly on main
# Always merge from dev/feature branches

# Before merging to main:
cd chamber-dev/
# Ensure tests pass
# Code review
# Then merge to main
cd ../chamber/
git merge dev
```

---

## Quick Reference

```bash
# List worktrees
git worktree list

# Create worktree
git worktree add ../path -b branch-name

# Remove worktree
git worktree remove ../path

# Move worktree
git worktree move ../old-path ../new-path

# Clean up
git worktree prune

# See branches
git branch -a

# Merge branches
git merge branch-name

# Compare branches
git diff main..dev
git log main..dev
```

---

## Summary

Worktrees enable:
- ✅ Parallel development without switching branches
- ✅ Long-lived dev servers while coding features
- ✅ Safe experimentation in isolated environments
- ✅ Easy PR review and testing
- ✅ Efficient release management

**Remember:** Each worktree is a full working directory with its own files, but they all share the same git repository history.
