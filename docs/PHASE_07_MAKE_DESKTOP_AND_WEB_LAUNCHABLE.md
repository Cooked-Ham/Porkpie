# Phase 07: Make Desktop and Web Launchable

## Binding

You are bound to Phase 07 only.

Your job is to turn wrapper crates into runnable Dioxus apps. Do not migrate stacks. Do not add Electron. Do not scaffold a TypeScript web app. The bar is low: launch the actual thing, not a README poem.

## Goal

Make desktop and web app targets launch with documented commands.

## Required Context

Read first:

- `apps/desktop/**`
- `apps/web/**`
- `crates/porkpie-ui/**`
- `Cargo.toml`
- README app launch docs

## Allowed Areas

- `apps/desktop/**`
- `apps/web/**`
- `crates/porkpie-ui/**` only if needed for launch integration
- `Cargo.toml`
- README/docs

## Forbidden

- No Electron.
- No React frontend.
- No TypeScript frontend foundation.
- No Vite main app.
- No fake launch command that only builds a library.
- No “works on my machine” undocumented startup step.

## Tasks

### Desktop

- Add a real Dioxus desktop entrypoint.
- Ensure this launches the app:

```bash
cargo run -p porkpie-desktop
```

### Web

- Add a real Dioxus web entrypoint.
- Document the exact web launch command.
- Ensure it uses the real Porkpie UI flow or clearly documented web storage mode.

### Docs

- Update README with exact commands.
- Add troubleshooting notes for required Dioxus tooling if applicable.

## Acceptance Criteria

- Desktop app launches.
- Web app launches.
- README commands work exactly as written.
- Both use real UI flow or clearly documented mode.
- No Electron/React/TypeScript/Vite main app introduced.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Desktop launch command/result
- Web launch command/result
- Remaining launch limitations
- Next recommended phase
