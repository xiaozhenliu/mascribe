import { describe, it, expect } from "vitest";
import { I18N, t, tf, mapKey, toOllamaTagsUrl } from "./utils";
import type { UiLang } from "./utils";

// ── i18n translation functions ──

describe("t", () => {
  it("should return English translation for known key", () => {
    expect(t("save", "en")).toBe("Save");
  });

  it("should return Chinese translation for known key", () => {
    expect(t("save", "zh")).toBe("保存");
  });

  it("should return the key itself for unknown key", () => {
    expect(t("nonexistent_key_xyz", "en")).toBe("nonexistent_key_xyz");
  });

  it("should return page title in both languages", () => {
    expect(t("page_title", "en")).toBe("Settings");
    expect(t("page_title", "zh")).toBe("设置");
  });
});

describe("tf", () => {
  it("should substitute single placeholder", () => {
    const result = tf("detected_ollama_models", { count: 5 }, "en");
    expect(result).toBe("Detected 5 Ollama models.");
  });

  it("should substitute single placeholder in Chinese", () => {
    const result = tf("detected_ollama_models", { count: 3 }, "zh");
    expect(result).toBe("已识别 3 个 Ollama 模型。");
  });

  it("should substitute multiple placeholders", () => {
    const result = tf("connection_success", { time: 150 }, "en");
    expect(result).toBe("✓ Connection successful (response time: 150ms)");
  });

  it("should handle error placeholder", () => {
    const result = tf("connection_failed", { error: "timeout" }, "en");
    expect(result).toBe("✗ Connection failed: timeout");
  });

  it("should leave unreplaced placeholders intact", () => {
    const result = tf("detected_ollama_models", {}, "en");
    expect(result).toBe("Detected {count} Ollama models.");
  });
});

// ── i18n dictionary completeness ──

describe("I18N dictionary", () => {
  it("should have all keys in both en and zh", () => {
    const enKeys = Object.keys(I18N.en).sort();
    const zhKeys = Object.keys(I18N.zh).sort();
    expect(enKeys).toEqual(zhKeys);
  });

  it("should have tab labels in both languages", () => {
    const tabKeys = ["tab_basic", "tab_polish", "tab_vision", "tab_dictionary"];
    for (const key of tabKeys) {
      expect(I18N.en[key]).toBeDefined();
      expect(I18N.zh[key]).toBeDefined();
    }
  });

  it("should have no empty translation values", () => {
    for (const lang of ["en", "zh"] as UiLang[]) {
      for (const [key, value] of Object.entries(I18N[lang])) {
        expect(value, `Key "${key}" is empty in "${lang}"`).not.toBe("");
      }
    }
  });
});

// ── mapKey: keyboard event mapping ──

function fakeKeyEvent(key: string, code: string): KeyboardEvent {
  return { key, code } as KeyboardEvent;
}

describe("mapKey", () => {
  it("should return null for lone modifier keys", () => {
    expect(mapKey(fakeKeyEvent("Control", "ControlLeft"))).toBeNull();
    expect(mapKey(fakeKeyEvent("Meta", "MetaLeft"))).toBeNull();
    expect(mapKey(fakeKeyEvent("Alt", "AltLeft"))).toBeNull();
    expect(mapKey(fakeKeyEvent("Shift", "ShiftLeft"))).toBeNull();
  });

  it("should map Space key", () => {
    expect(mapKey(fakeKeyEvent(" ", "Space"))).toBe("Space");
  });

  it("should return null for Escape", () => {
    expect(mapKey(fakeKeyEvent("Escape", "Escape"))).toBeNull();
  });

  it("should map ContextMenu key", () => {
    expect(mapKey(fakeKeyEvent("ContextMenu", "ContextMenu"))).toBe("ContextMenu");
  });

  it("should map arrow keys", () => {
    expect(mapKey(fakeKeyEvent("ArrowUp", "ArrowUp"))).toBe("ArrowUp");
    expect(mapKey(fakeKeyEvent("ArrowDown", "ArrowDown"))).toBe("ArrowDown");
    expect(mapKey(fakeKeyEvent("ArrowLeft", "ArrowLeft"))).toBe("ArrowLeft");
    expect(mapKey(fakeKeyEvent("ArrowRight", "ArrowRight"))).toBe("ArrowRight");
  });

  it("should map F1-F12 keys", () => {
    expect(mapKey(fakeKeyEvent("F1", "F1"))).toBe("F1");
    expect(mapKey(fakeKeyEvent("F12", "F12"))).toBe("F12");
  });

  it("should map letter keys using code", () => {
    expect(mapKey(fakeKeyEvent("a", "KeyA"))).toBe("A");
    expect(mapKey(fakeKeyEvent("Z", "KeyZ"))).toBe("Z");
  });

  it("should map digit keys using code", () => {
    expect(mapKey(fakeKeyEvent("1", "Digit1"))).toBe("1");
    expect(mapKey(fakeKeyEvent("9", "Digit9"))).toBe("9");
  });

  it("should map special keys", () => {
    expect(mapKey(fakeKeyEvent("Tab", "Tab"))).toBe("Tab");
    expect(mapKey(fakeKeyEvent("Enter", "Enter"))).toBe("Enter");
    expect(mapKey(fakeKeyEvent("Backspace", "Backspace"))).toBe("Backspace");
    expect(mapKey(fakeKeyEvent("Delete", "Delete"))).toBe("Delete");
  });

  it("should map punctuation keys", () => {
    expect(mapKey(fakeKeyEvent("[", "BracketLeft"))).toBe("[");
    expect(mapKey(fakeKeyEvent("]", "BracketRight"))).toBe("]");
    expect(mapKey(fakeKeyEvent(";", "Semicolon"))).toBe(";");
    expect(mapKey(fakeKeyEvent("'", "Quote"))).toBe("'");
    expect(mapKey(fakeKeyEvent(",", "Comma"))).toBe(",");
    expect(mapKey(fakeKeyEvent(".", "Period"))).toBe(".");
    expect(mapKey(fakeKeyEvent("/", "Slash"))).toBe("/");
    expect(mapKey(fakeKeyEvent("-", "Minus"))).toBe("-");
    expect(mapKey(fakeKeyEvent("=", "Equal"))).toBe("=");
    expect(mapKey(fakeKeyEvent("`", "Backquote"))).toBe("`");
  });

  it("should map digit codes even when shifted", () => {
    // When Shift+1 is pressed, key="!" but code="Digit1" — function uses code for digits
    expect(mapKey(fakeKeyEvent("!", "Digit1"))).toBe("1");
    expect(mapKey(fakeKeyEvent("@", "Digit2"))).toBe("2");
  });
});

// ── toOllamaTagsUrl ──

describe("toOllamaTagsUrl", () => {
  it("should convert v1 endpoint to tags URL", () => {
    expect(toOllamaTagsUrl("http://localhost:11434/v1")).toBe(
      "http://localhost:11434/api/tags"
    );
  });

  it("should convert chat completions endpoint to tags URL", () => {
    expect(toOllamaTagsUrl("https://api.example.com/v1/chat/completions")).toBe(
      "https://api.example.com/api/tags"
    );
  });

  it("should handle trailing slashes", () => {
    expect(toOllamaTagsUrl("http://localhost:11434/v1/")).toBe(
      "http://localhost:11434/api/tags"
    );
  });

  it("should use default endpoint for empty input", () => {
    expect(toOllamaTagsUrl("")).toBe("http://localhost:11434/api/tags");
  });

  it("should use default endpoint for whitespace-only input", () => {
    expect(toOllamaTagsUrl("   ")).toBe("http://localhost:11434/api/tags");
  });

  it("should handle base URL without v1 suffix", () => {
    expect(toOllamaTagsUrl("http://localhost:11434")).toBe(
      "http://localhost:11434/api/tags"
    );
  });

  it("should handle custom port endpoint", () => {
    expect(toOllamaTagsUrl("http://192.168.1.100:8080/v1")).toBe(
      "http://192.168.1.100:8080/api/tags"
    );
  });
});
