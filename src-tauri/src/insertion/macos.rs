//! macOS text insertion via CGEvent (Cmd+V simulation).

use arboard::Clipboard;
use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::process::Command;
use std::thread;
use std::time::Duration;

const KEY_V: CGKeyCode = 9;

pub fn insert_text(text: &str) -> anyhow::Result<()> {
    println!("[insert_text] inserting: '{}'", text);

    let ax_trusted = is_accessibility_trusted();
    println!("[insert_text] accessibility trusted: {}", ax_trusted);

    let mut clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!("Clipboard error: {}", e))?;

    // 1. Set recognition text to clipboard (keep it for re-paste)
    clipboard
        .set_text(text)
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;
    println!("[insert_text] clipboard set ({} chars)", text.len());

    // 2. Wait for clipboard to settle
    thread::sleep(Duration::from_millis(50));

    // 3. Simulate Cmd+V — try CGEvent first, fall back to AppleScript
    if ax_trusted {
        println!("[insert_text] simulating Cmd+V via CGEvent");
        simulate_cmd_v()?;
    } else {
        println!("[insert_text] no accessibility permission, using AppleScript fallback");
        simulate_cmd_v_applescript()?;
    }

    println!("[insert_text] done (text remains in clipboard)");
    Ok(())
}

/// Check if the app has Accessibility (Trusted) permission on macOS.
/// prompt=false: silent check, no system popup.
fn is_accessibility_trusted() -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: core_foundation::base::CFTypeRef) -> bool;
    }

    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::false_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

    unsafe { AXIsProcessTrustedWithOptions(options.as_CFTypeRef()) }
}

fn simulate_cmd_v() -> anyhow::Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow::anyhow!("Failed to create CGEventSource"))?;

    let key_down = CGEvent::new_keyboard_event(source.clone(), KEY_V, true)
        .map_err(|_| anyhow::anyhow!("Failed to create key down event"))?;
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_down.post(core_graphics::event::CGEventTapLocation::HID);

    let key_up = CGEvent::new_keyboard_event(source, KEY_V, false)
        .map_err(|_| anyhow::anyhow!("Failed to create key up event"))?;
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.post(core_graphics::event::CGEventTapLocation::HID);

    Ok(())
}

/// Fallback: use osascript to simulate Cmd+V via System Events.
/// osascript/System Events has its own Accessibility grant which is usually
/// already authorized on dev machines and persists across app rebuilds.
fn simulate_cmd_v_applescript() -> anyhow::Result<()> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run osascript: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("[insert_text] AppleScript stderr: {}", stderr);
        return Err(anyhow::anyhow!("AppleScript Cmd+V failed: {}", stderr));
    }
    println!("[insert_text] AppleScript Cmd+V sent OK");
    Ok(())
}
