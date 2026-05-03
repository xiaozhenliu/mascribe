# Testing Conventions

> Conventions for testing in Mascribe: Vitest (frontend) + Rust built-in test framework (backend).

---

## General Principles

- **No TDD** — tests verify existing behavior, not drive design (per CLAUDE.md)
- Tests should be deterministic and fast
- Test pure functions and logic, not UI rendering or system-level integration
- Each test file tests one module/concern
- Use descriptive test names that explain the expected behavior

---

## Frontend (TypeScript)

### Tool

- **Vitest** — lightweight, Vite-native test runner
- Test environment: `jsdom` (provides `document`, `navigator`, `KeyboardEvent`, etc.)
- Run command: `npx vitest run`

### What to Test

- Pure logic functions (e.g., `mapKey()`, `t()`, `tf()`, `toOllamaTagsUrl()`)
- Data transformations and state transitions
- i18n key completeness (all keys present in both `en` and `zh`)
- URL/string manipulation utilities

### What NOT to Test

- DOM rendering (no framework — vanilla DOM is hard to unit test)
- Tauri `invoke()` calls (require native runtime; mock if needed)
- Visual layout / CSS (manual verification)
- Async workflows that depend on native APIs

### File Organization

Test files are **colocated** next to the source file they test:

```
src/
  utils.ts          # Pure utility functions
  utils.test.ts     # Tests for utils.ts
  settings.ts       # Settings page logic (not tested directly — too many DOM deps)
```

- Test files use the `.test.ts` suffix
- Each test file imports directly from the source module
- Keep tests self-contained — no shared test fixtures between files

### Test Structure

```ts
import { describe, it, expect } from "vitest";

describe("functionName", () => {
  it("should handle basic case", () => {
    expect(functionName(input)).toBe(expected);
  });

  it("should handle edge case", () => {
    expect(functionName(edgeInput)).toBe(expected);
  });
});
```

### DOM Dependencies

When testing functions that use DOM APIs (e.g., `KeyboardEvent`), create minimal mock objects:

```ts
it("should map letter keys", () => {
  const event = { key: "a", code: "KeyA" } as KeyboardEvent;
  expect(mapKey(event)).toBe("A");
});
```

### i18n Testing

Test that translation dictionaries are complete:

```ts
it("should have all keys in both languages", () => {
  const enKeys = Object.keys(I18N.en).sort();
  const zhKeys = Object.keys(I18N.zh).sort();
  expect(enKeys).toEqual(zhKeys);
});
```

---

## Rust

### Tool

- Standard Rust test framework (`#[cfg(test)]` + `#[test]`)
- Run command: `cargo test` (from `src-tauri/`)

### What to Test

- `AppConfig` serialization round-trip (serialize → deserialize → compare)
- Default config values (field types, non-empty required fields)
- Forward-compatible loading (config with missing fields gets defaults)
- `CorrectionDictionary` logic (apply, longest-match-first sorting, case-insensitive replacement)
- Pure data processing functions

### What NOT to Test

- Tauri commands (require full app context with `State<>`, `AppHandle`)
- Native macOS/Windows APIs (`objc2`, CGEventTap)
- Audio capture (requires hardware)
- Network calls (requires running services)

### Test Module Pattern

Add `#[cfg(test)]` module at the bottom of the source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        let result = function_under_test(input);
        assert_eq!(result, expected);
    }
}
```

### Config Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_valid_values() {
        let config = AppConfig::default();
        assert!(!config.language.is_empty());
        assert!(config.num_threads > 0);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.language, restored.language);
    }

    #[test]
    fn config_load_with_missing_fields_uses_defaults() {
        // Simulate loading a partial config
        let partial = r#"{"language": "en"}"#;
        let defaults = AppConfig::default();
        let mut value = serde_json::to_value(&defaults).unwrap();
        if let (Some(base), Some(overlay)) = (
            value.as_object_mut(),
            serde_json::from_str::<serde_json::Value>(partial)
                .unwrap()
                .as_object(),
        ) {
            for (k, v) in overlay {
                base.insert(k.clone(), v.clone());
            }
        }
        let config: AppConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config.language, "en"); // from saved
        assert_eq!(config.num_threads, 4); // from default
    }
}
```

### Correction Dictionary Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_replaces_exact_match() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("hello".to_string(), "world".to_string()),
        ]);
        assert_eq!(dict.apply("hello"), "world");
    }

    #[test]
    fn apply_is_case_insensitive() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("hello".to_string(), "world".to_string()),
        ]);
        assert_eq!(dict.apply("HELLO"), "world");
        assert_eq!(dict.apply("Hello"), "world");
    }

    #[test]
    fn longest_match_first() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("ab".to_string(), "XY".to_string()),
            ("abc".to_string(), "Z".to_string()),
        ]);
        // "abc" should match before "ab" because it's longer
        assert_eq!(dict.apply("abc"), "Z");
    }
}
```

---

## Running Tests

| Scope | Command | Directory |
|-------|---------|-----------|
| Frontend (all) | `npx vitest run` | Project root |
| Frontend (watch) | `npx vitest` | Project root |
| Rust (all) | `cargo test` | `src-tauri/` |
| Rust (specific) | `cargo test test_name` | `src-tauri/` |
