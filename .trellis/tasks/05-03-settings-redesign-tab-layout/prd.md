# Settings Page Redesign — Tab Layout

## Goal

Redesign the settings page from a single-column scrollable layout to a left-side tab navigation + right-side content area layout, covering Phase 1 (core structure), Phase 2 (content migration), and Phase 3 (interaction polish) from the design doc. Also add comprehensive testing infrastructure and conventions.

## Requirements

### UI Redesign
1. Implement left-side tab navigation (140px fixed width) with 4 tabs: Basic, AI Polish, Vision, Dictionary
2. Implement right-side content area (flex: 1, max-width 600px centered)
3. Fixed top header bar (64px) with title + language selector
4. Fixed bottom action bar (64px) with Save button
5. Tab switching with fade animation (150ms)
6. Migrate existing settings sections to corresponding tabs (no logic changes to settings.ts)
7. Responsive fallback: tabs go horizontal when width < 700px (Phase 4 from design doc, included in this task)

### Window Config
8. Update settings window size in `tray.rs`: `inner_size(800.0, 600.0)`, `min_inner_size(700.0, 500.0)`

### Testing
9. Create `.trellis/spec/testing/index.md` with testing conventions (what to test, tools, patterns)
10. Set up Vitest for frontend unit tests
11. Add Rust unit tests (`#[cfg(test)]`) for core modules (config, correction dictionary, etc.)
12. Write tests for the settings tab switching logic (as a reference test)

## Decisions (Confirmed)

- **CSS**: Keep `settings.css` as a separate file (existing pattern, CLAUDE.md updated to remove inline-only rule)
- **Testing scope**: Comprehensive — conventions doc + Vitest (frontend) + Rust unit tests

## Acceptance Criteria

- [ ] 4 tabs visible on left side, clicking switches content correctly
- [ ] All existing settings functionality preserved (load, save, shortcut recording, browse, corrections, detect Ollama, test connection, etc.)
- [ ] i18n works for all new tab labels (en + zh)
- [ ] Save button always visible at bottom (fixed position)
- [ ] Top header always visible (fixed position)
- [ ] Settings window opens at 800x600
- [ ] Tab content scrolls independently (not the whole page)
- [ ] Responsive: horizontal tabs at < 700px width
- [ ] Tab label i18n updates when language changes
- [ ] Testing conventions documented in spec
- [ ] Vitest configured and at least one frontend test passes
- [ ] At least one Rust unit test passes

## Definition of Done

- All acceptance criteria met
- `npm run tauri dev` — settings window works correctly
- Tests pass (`npx vitest run` for frontend, `cargo test` for Rust)
- Design doc followed for layout specs (colors, spacing, typography)
- CHANGELOG.md updated

## Out of Scope

- New settings features (only restructure existing)
- Changes to main overlay window
- E2E / integration tests (desktop app testing is complex, defer)
- Performance optimization

## Technical Notes

- Design doc: `docs/settings_design.md` — detailed layout specs, CSS values, interaction design
- HTML element IDs must stay the same (settings.ts references them heavily)
- The `.hidden` class pattern used for conditional sections must be preserved
- `settings.ts` initialization order: detect language → detect platform → apply language → setup radios → load config → load corrections → setup event listeners
- Toast notification is dynamically created in TS, not in HTML
- Tab navigation is pure HTML/CSS/TS — no framework
- Existing radio-group segmented control CSS can be reused as-is
