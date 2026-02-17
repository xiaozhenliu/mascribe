//! macOS text insertion via CGEvent (Cmd+V simulation).

use arboard::Clipboard;
use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use objc2_app_kit::{
    NSApplicationActivationOptions, NSRunningApplication, NSWorkspace,
};
use std::sync::Mutex;
use std::process::Command;
use std::thread;
use std::time::Duration;

const KEY_V: CGKeyCode = 9;
static LAST_TARGET_PID: Mutex<Option<i32>> = Mutex::new(None);

pub fn remember_frontmost_target_app() {
    let workspace = NSWorkspace::sharedWorkspace();
    let Some(frontmost) = workspace.frontmostApplication() else {
        println!("[insert_text] no frontmost app available");
        return;
    };

    let frontmost_pid = frontmost.processIdentifier();
    let current_pid = NSRunningApplication::currentApplication().processIdentifier();
    if frontmost_pid == current_pid {
        println!("[insert_text] frontmost app is MaScribe itself, skip remembering target");
        return;
    }

    let mut slot = LAST_TARGET_PID.lock().unwrap();
    *slot = Some(frontmost_pid);
    println!("[insert_text] remembered paste target pid={}", frontmost_pid);
}

pub fn reactivate_remembered_target_app() {
    let target_pid = {
        let slot = LAST_TARGET_PID.lock().unwrap();
        *slot
    };

    let Some(pid) = target_pid else {
        println!("[insert_text] no remembered target app pid");
        return;
    };

    let Some(target) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
        println!("[insert_text] remembered target pid={} is no longer running", pid);
        return;
    };

    let workspace = NSWorkspace::sharedWorkspace();
    let frontmost_pid = workspace
        .frontmostApplication()
        .map(|app| app.processIdentifier())
        .unwrap_or_default();
    if frontmost_pid == pid {
        println!("[insert_text] target pid={} already frontmost", pid);
        return;
    }

    // Use gentle activation (no force-all-windows) and retry once.
    let ok1 = target.activateWithOptions(NSApplicationActivationOptions::empty());
    if !ok1 {
        thread::sleep(Duration::from_millis(40));
        let ok2 = target.activateWithOptions(NSApplicationActivationOptions::empty());
        println!(
            "[insert_text] re-activate target pid={} result={} retry={}",
            pid, ok1, ok2
        );
    } else {
        println!("[insert_text] re-activate target pid={} result={}", pid, ok1);
    }
}

pub fn insert_text(text: &str) -> anyhow::Result<()> {
    println!("[insert_text] inserting: '{}'", text);

    // Always verify permission right before paste. If missing, actively prompt first.
    let mut ax_trusted = is_accessibility_trusted(false);
    if !ax_trusted {
        println!("[insert_text] accessibility missing, requesting permission prompt");
        ax_trusted = is_accessibility_trusted(true);
    }
    println!("[insert_text] accessibility trusted: {}", ax_trusted);

    let mut clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!("Clipboard error: {}", e))?;

    // 1. Set recognition text to clipboard (keep it for re-paste)
    clipboard
        .set_text(text)
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;
    println!("[insert_text] clipboard set ({} chars)", text.len());

    // 2. Wait for clipboard to settle
    thread::sleep(Duration::from_millis(50));

    // 3. Bring the previously focused app back to front, then paste.
    reactivate_remembered_target_app();
    thread::sleep(Duration::from_millis(80));

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
/// prompt=true: ask macOS to open the Accessibility authorization flow.
fn is_accessibility_trusted(prompt: bool) -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: core_foundation::base::CFTypeRef) -> bool;
    }

    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = if prompt {
        CFBoolean::true_value()
    } else {
        CFBoolean::false_value()
    };
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
