//! Platform-independent keycode definitions and parsing.
//!
//! Provides a unified way to represent hotkeys across macOS and Windows.
//! Automatically normalizes platform-specific key names (Command -> Ctrl on Windows).

use std::fmt;

/// A hotkey definition with modifiers and key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotkeyDefinition {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

/// Modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,   // Control on macOS, Ctrl on Windows
    Shift,
    Alt,    // Option on macOS, Alt on Windows
    Meta,   // Command on macOS, Windows key on Windows
}

/// Key definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    // Function keys
    FKey(u8),  // F1-F24
    // Special keys
    Escape,
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    ContextMenu,
    // Numpad keys
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadDecimal,   // .
    NumpadMultiply,  // *
    NumpadPlus,      // +
    NumpadMinus,     // -
    NumpadDivide,    // /
    NumpadEnter,
    // Punctuation
    Comma,      // ,
    Period,     // .
    Slash,      // /
    Semicolon,  // ;
    Quote,      // '
    Backslash,  // \
    BracketLeft,  // [
    BracketRight, // ]
    Minus,      // -
    Equal,      // =
    Backtick,   // `
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Ctrl => write!(f, "Ctrl"),
            Modifier::Shift => write!(f, "Shift"),
            Modifier::Alt => write!(f, "Alt"),
            Modifier::Meta => write!(f, "Meta"),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::A => write!(f, "A"),
            Key::B => write!(f, "B"),
            Key::C => write!(f, "C"),
            Key::D => write!(f, "D"),
            Key::E => write!(f, "E"),
            Key::F => write!(f, "F"),
            Key::G => write!(f, "G"),
            Key::H => write!(f, "H"),
            Key::I => write!(f, "I"),
            Key::J => write!(f, "J"),
            Key::K => write!(f, "K"),
            Key::L => write!(f, "L"),
            Key::M => write!(f, "M"),
            Key::N => write!(f, "N"),
            Key::O => write!(f, "O"),
            Key::P => write!(f, "P"),
            Key::Q => write!(f, "Q"),
            Key::R => write!(f, "R"),
            Key::S => write!(f, "S"),
            Key::T => write!(f, "T"),
            Key::U => write!(f, "U"),
            Key::V => write!(f, "V"),
            Key::W => write!(f, "W"),
            Key::X => write!(f, "X"),
            Key::Y => write!(f, "Y"),
            Key::Z => write!(f, "Z"),
            Key::Num0 => write!(f, "0"),
            Key::Num1 => write!(f, "1"),
            Key::Num2 => write!(f, "2"),
            Key::Num3 => write!(f, "3"),
            Key::Num4 => write!(f, "4"),
            Key::Num5 => write!(f, "5"),
            Key::Num6 => write!(f, "6"),
            Key::Num7 => write!(f, "7"),
            Key::Num8 => write!(f, "8"),
            Key::Num9 => write!(f, "9"),
            Key::FKey(n) => write!(f, "F{}", n),
            Key::Escape => write!(f, "Escape"),
            Key::Space => write!(f, "Space"),
            Key::Enter => write!(f, "Enter"),
            Key::Tab => write!(f, "Tab"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Delete => write!(f, "Delete"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::Up => write!(f, "Up"),
            Key::Down => write!(f, "Down"),
            Key::Left => write!(f, "Left"),
            Key::Right => write!(f, "Right"),
            Key::ContextMenu => write!(f, "ContextMenu"),
            Key::Comma => write!(f, ","),
            Key::Period => write!(f, "."),
            Key::Slash => write!(f, "/"),
            Key::Semicolon => write!(f, ";"),
            Key::Quote => write!(f, "'"),
            Key::Backslash => write!(f, "\\"),
            Key::BracketLeft => write!(f, "["),
            Key::BracketRight => write!(f, "]"),
            Key::Minus => write!(f, "-"),
            Key::Equal => write!(f, "="),
            Key::Backtick => write!(f, "`"),
            // Numpad
            Key::Numpad0 => write!(f, "Numpad0"),
            Key::Numpad1 => write!(f, "Numpad1"),
            Key::Numpad2 => write!(f, "Numpad2"),
            Key::Numpad3 => write!(f, "Numpad3"),
            Key::Numpad4 => write!(f, "Numpad4"),
            Key::Numpad5 => write!(f, "Numpad5"),
            Key::Numpad6 => write!(f, "Numpad6"),
            Key::Numpad7 => write!(f, "Numpad7"),
            Key::Numpad8 => write!(f, "Numpad8"),
            Key::Numpad9 => write!(f, "Numpad9"),
            Key::NumpadDecimal => write!(f, "Numpad."),
            Key::NumpadMultiply => write!(f, "Numpad*"),
            Key::NumpadPlus => write!(f, "Numpad+"),
            Key::NumpadMinus => write!(f, "Numpad-"),
            Key::NumpadDivide => write!(f, "Numpad/"),
            Key::NumpadEnter => write!(f, "NumpadEnter"),
        }
    }
}

impl fmt::Display for HotkeyDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts: Vec<String> = self.modifiers.iter().map(|m| m.to_string()).collect();
        parts.push(self.key.to_string());
        write!(f, "{}", parts.join("+"))
    }
}

/// Parse a hotkey string like "Ctrl+Shift+S" into a HotkeyDefinition.
/// Normalizes platform-specific names:
/// - "Command" or "Cmd" -> Ctrl on Windows, Meta on macOS
/// - "Option" or "Opt" -> Alt
/// - "Win" or "Windows" -> Meta on Windows
pub fn parse_hotkey(hotkey: &str) -> Result<HotkeyDefinition, String> {
    let normalized = hotkey.trim();
    if normalized.is_empty() {
        return Err("Empty hotkey string".to_string());
    }

    let parts: Vec<&str> = normalized.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err("Invalid hotkey format".to_string());
    }

    let mut modifiers = Vec::new();
    let mut key_str = "";

    for (i, part) in parts.iter().enumerate() {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => modifiers.push(Modifier::Ctrl),
            "shift" => modifiers.push(Modifier::Shift),
            "alt" | "option" | "opt" => modifiers.push(Modifier::Alt),
            "meta" | "command" | "cmd" | "win" | "windows" | "super" => modifiers.push(Modifier::Meta),
            _ => {
                if i == parts.len() - 1 {
                    key_str = part;
                } else {
                    return Err(format!("Unknown modifier: {}", part));
                }
            }
        }
    }

    if key_str.is_empty() {
        return Err("No key specified".to_string());
    }

    let key = parse_key(key_str)?;

    Ok(HotkeyDefinition { modifiers, key })
}

/// Parse a key string into a Key enum.
fn parse_key(key_str: &str) -> Result<Key, String> {
    let lower = key_str.to_lowercase();
    match lower.as_str() {
        "a" => Ok(Key::A),
        "b" => Ok(Key::B),
        "c" => Ok(Key::C),
        "d" => Ok(Key::D),
        "e" => Ok(Key::E),
        "f" => Ok(Key::F),
        "g" => Ok(Key::G),
        "h" => Ok(Key::H),
        "i" => Ok(Key::I),
        "j" => Ok(Key::J),
        "k" => Ok(Key::K),
        "l" => Ok(Key::L),
        "m" => Ok(Key::M),
        "n" => Ok(Key::N),
        "o" => Ok(Key::O),
        "p" => Ok(Key::P),
        "q" => Ok(Key::Q),
        "r" => Ok(Key::R),
        "s" => Ok(Key::S),
        "t" => Ok(Key::T),
        "u" => Ok(Key::U),
        "v" => Ok(Key::V),
        "w" => Ok(Key::W),
        "x" => Ok(Key::X),
        "y" => Ok(Key::Y),
        "z" => Ok(Key::Z),
        "0" => Ok(Key::Num0),
        "1" => Ok(Key::Num1),
        "2" => Ok(Key::Num2),
        "3" => Ok(Key::Num3),
        "4" => Ok(Key::Num4),
        "5" => Ok(Key::Num5),
        "6" => Ok(Key::Num6),
        "7" => Ok(Key::Num7),
        "8" => Ok(Key::Num8),
        "9" => Ok(Key::Num9),
        "escape" | "esc" => Ok(Key::Escape),
        "space" | "spacebar" => Ok(Key::Space),
        "enter" | "return" => Ok(Key::Enter),
        "tab" => Ok(Key::Tab),
        "backspace" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" | "page_up" => Ok(Key::PageUp),
        "pagedown" | "page_down" => Ok(Key::PageDown),
        "up" | "uparrow" => Ok(Key::Up),
        "down" | "downarrow" => Ok(Key::Down),
        "left" | "leftarrow" => Ok(Key::Left),
        "right" | "rightarrow" => Ok(Key::Right),
        "contextmenu" | "context_menu" | "context" | "menu" => Ok(Key::ContextMenu),
        "," | "comma" => Ok(Key::Comma),
        "." | "period" | "dot" => Ok(Key::Period),
        "/" | "slash" => Ok(Key::Slash),
        ";" | "semicolon" => Ok(Key::Semicolon),
        "'" | "quote" | "apostrophe" => Ok(Key::Quote),
        "\\" | "backslash" => Ok(Key::Backslash),
        "[" | "bracketleft" | "bracket_left" => Ok(Key::BracketLeft),
        "]" | "bracketright" | "bracket_right" => Ok(Key::BracketRight),
        "-" | "minus" | "dash" => Ok(Key::Minus),
        "=" | "equal" | "equals" => Ok(Key::Equal),
        "`" | "backtick" | "grave" | "backquote" => Ok(Key::Backtick),
        // Numpad keys
        "numpad0" => Ok(Key::Numpad0),
        "numpad1" => Ok(Key::Numpad1),
        "numpad2" => Ok(Key::Numpad2),
        "numpad3" => Ok(Key::Numpad3),
        "numpad4" => Ok(Key::Numpad4),
        "numpad5" => Ok(Key::Numpad5),
        "numpad6" => Ok(Key::Numpad6),
        "numpad7" => Ok(Key::Numpad7),
        "numpad8" => Ok(Key::Numpad8),
        "numpad9" => Ok(Key::Numpad9),
        "numpad." | "numpad_decimal" | "numpaddecimal" => Ok(Key::NumpadDecimal),
        "numpad*" | "numpad_multiply" | "numpadmultiply" => Ok(Key::NumpadMultiply),
        "numpad+" | "numpad_plus" | "numpadplus" => Ok(Key::NumpadPlus),
        "numpad-" | "numpad_minus" | "numpadminus" => Ok(Key::NumpadMinus),
        "numpad/" | "numpad_divide" | "numpaddivide" => Ok(Key::NumpadDivide),
        "numpad_enter" | "numpadenter" => Ok(Key::NumpadEnter),
        _ => {
            // Check for function keys F1-F24
            if lower.starts_with('f') {
                let num_str = &lower[1..];
                if let Ok(num) = num_str.parse::<u8>() {
                    if num >= 1 && num <= 24 {
                        return Ok(Key::FKey(num));
                    }
                }
            }
            Err(format!("Unknown key: {}", key_str))
        }
    }
}

/// Normalize a hotkey string for the current platform.
/// On Windows: Command -> Ctrl, Option -> Alt
/// On macOS: Ctrl -> Ctrl (no change), but we keep the user's preference
pub fn normalize_for_platform(hotkey: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        // On macOS, keep Command/Option as-is (they'll be mapped to Meta/Alt)
        hotkey.to_string()
    }
    #[cfg(target_os = "windows")]
    {
        // On Windows, normalize macOS-style keys to Windows equivalents
        hotkey
            .replace("Command", "Ctrl")
            .replace("Cmd", "Ctrl")
            .replace("command", "ctrl")
            .replace("cmd", "ctrl")
            .replace("Option", "Alt")
            .replace("Opt", "Alt")
            .replace("option", "alt")
            .replace("opt", "alt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let hk = parse_hotkey("S").unwrap();
        assert!(hk.modifiers.is_empty());
        assert_eq!(hk.key, Key::S);
    }

    #[test]
    fn test_parse_with_modifiers() {
        let hk = parse_hotkey("Ctrl+Shift+S").unwrap();
        assert_eq!(hk.modifiers.len(), 2);
        assert!(hk.modifiers.contains(&Modifier::Ctrl));
        assert!(hk.modifiers.contains(&Modifier::Shift));
        assert_eq!(hk.key, Key::S);
    }

    #[test]
    fn test_parse_function_key() {
        let hk = parse_hotkey("Ctrl+F9").unwrap();
        assert_eq!(hk.modifiers, vec![Modifier::Ctrl]);
        assert_eq!(hk.key, Key::FKey(9));
    }

    #[test]
    fn test_parse_macos_style() {
        let hk = parse_hotkey("Command+Shift+S").unwrap();
        assert!(hk.modifiers.contains(&Modifier::Meta));
        assert!(hk.modifiers.contains(&Modifier::Shift));
    }

    #[test]
    fn test_display() {
        let hk = HotkeyDefinition {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::S,
        };
        assert_eq!(hk.to_string(), "Ctrl+Shift+S");
    }

    #[test]
    fn test_parse_context_menu() {
        let hk = parse_hotkey("ContextMenu").unwrap();
        assert!(hk.modifiers.is_empty());
        assert_eq!(hk.key, Key::ContextMenu);
    }
}
