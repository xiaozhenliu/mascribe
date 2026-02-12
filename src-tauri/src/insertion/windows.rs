//! Windows text insertion via SendInput (Ctrl+V simulation).

use arboard::Clipboard;
use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub fn insert_text(text: &str) -> anyhow::Result<()> {
    println!("[insert_text] inserting: '{}'", text);

    let mut clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!("Clipboard error: {}", e))?;

    // 1. Set text to clipboard
    clipboard
        .set_text(text)
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;
    println!("[insert_text] clipboard set ({} chars)", text.len());

    // 2. Wait for clipboard to settle
    thread::sleep(Duration::from_millis(50));

    // 3. Simulate Ctrl+V using SendInput
    println!("[insert_text] simulating Ctrl+V via SendInput");
    simulate_ctrl_v()?;

    println!("[insert_text] done (text remains in clipboard)");
    Ok(())
}

fn simulate_ctrl_v() -> anyhow::Result<()> {
    unsafe {
        // Input for Ctrl down
        let ctrl_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        // Input for V down
        let v_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        // Input for V up
        let v_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        // Input for Ctrl up
        let ctrl_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        // Send Ctrl down, V down, V up, Ctrl up
        let inputs = [ctrl_down, v_down, v_up, ctrl_up];
        let sent = SendInput(
            &inputs,
            std::mem::size_of::<INPUT>() as i32,
        );

        if sent != inputs.len() as u32 {
            return Err(anyhow::anyhow!(
                "SendInput failed: only {} of {} inputs sent",
                sent,
                inputs.len()
            ));
        }
    }

    Ok(())
}
