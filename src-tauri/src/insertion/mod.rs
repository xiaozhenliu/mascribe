//! Cross-platform text insertion via clipboard paste simulation.
//!
//! On macOS: Uses CGEvent to simulate Cmd+V (requires Accessibility permission)
//! On Windows: Uses SendInput to simulate Ctrl+V (no special permissions needed)

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

/// Insert text into the active application by simulating paste (Ctrl+V / Cmd+V).
///
/// This function:
/// 1. Saves the current clipboard content
/// 2. Sets the text to the clipboard
/// 3. Simulates the paste keyboard shortcut
/// 4. Restores the original clipboard content
pub fn insert_text(text: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    return macos::insert_text(text);

    #[cfg(target_os = "windows")]
    return windows::insert_text(text);

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err(anyhow::anyhow!("Text insertion not supported on this platform"))
}
