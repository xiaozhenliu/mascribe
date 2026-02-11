use arboard::Clipboard;
use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::thread;
use std::time::Duration;

const KEY_V: CGKeyCode = 9;

pub fn insert_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!("Clipboard error: {}", e))?;

    // 1. Save current clipboard
    let saved = clipboard.get_text().ok();

    // 2. Set recognition text
    clipboard
        .set_text(text)
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;

    // 3. Wait for clipboard to settle
    thread::sleep(Duration::from_millis(50));

    // 4. Simulate Cmd+V
    simulate_cmd_v()?;

    // 5. Wait for paste to complete
    thread::sleep(Duration::from_millis(100));

    // 6. Restore original clipboard
    if let Some(original) = saved {
        let _ = clipboard.set_text(&original);
    }

    Ok(())
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
