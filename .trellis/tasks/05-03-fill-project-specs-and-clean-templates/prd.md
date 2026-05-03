# PRD: Fill Project Specs and Clean Templates

## Goal

Populate `.trellis/spec/` with code conventions extracted from the actual codebase, and remove template files that don't apply to this project.

## Background

- The project is a Tauri 2 app (Rust backend + vanilla TypeScript frontend, no React/Vue)
- `.trellis/spec/frontend/` has template files, all empty placeholders
- No Rust/backend spec exists
- CLAUDE.md already has some conventions (inline styles, Tauri command flow, anyhow error handling)

## Scope

### 1. Rewrite `.trellis/spec/frontend/index.md`
- Single file replacing the scattered template files
- Content: actual conventions from the codebase (vanilla TS, DOM manipulation, inline styles, invoke pattern, naming)

### 2. Create `.trellis/spec/rust/index.md`
- New file for Rust backend conventions
- Content: error handling (anyhow vs String), module structure, command registration, platform-specific code, state management

### 3. Delete inapplicable template files
- Remove: hook-guidelines.md, component-guidelines.md, state-management.md, type-safety.md, quality-guidelines.md, directory-structure.md
- These are React/framework-oriented templates that don't apply to this vanilla TS project

## Out of Scope

- No code changes, only spec/documentation files
- No changes to workflow.md or config.yaml
