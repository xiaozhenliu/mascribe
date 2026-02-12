//! Cross-platform permission helpers.
//!
//! On macOS, three permissions are needed at startup:
//! 1. **Microphone** — cpal uses CoreAudio directly, which does NOT trigger the
//!    macOS TCC dialog. We call AVCaptureDevice.requestAccess via Objective-C.
//! 2. **Accessibility** — needed for CGEvent.post() to simulate Cmd+V. We call
//!    AXIsProcessTrustedWithOptions(prompt: true) to show the system dialog.
//! 3. **Screen Recording** — needed for capturing the active window. We use
//!    CGPreflightScreenCaptureAccess to check and trigger the permission dialog.
//!
//! On Windows, these permissions are handled differently:
//! - Microphone: Windows prompts automatically on first use
//! - Accessibility/Screen Recording: No explicit permission system
//!   (SendInput and GDI work without special permissions)

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::Bool;
#[cfg(target_os = "macos")]
use objc2::{class, msg_send};
#[cfg(target_os = "macos")]
use objc2_foundation::NSString;

/// AVAuthorizationStatus values from Apple docs.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum AVAuthorizationStatus {
    NotDetermined = 0,
    Restricted = 1,
    Denied = 2,
    Authorized = 3,
}

/// Check the current microphone authorization status.
#[cfg(target_os = "macos")]
pub fn microphone_authorization_status() -> AVAuthorizationStatus {
    unsafe {
        let media_type: Retained<NSString> = NSString::from_str("soun");
        let status: i32 = msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: &*media_type
        ];
        match status {
            0 => AVAuthorizationStatus::NotDetermined,
            1 => AVAuthorizationStatus::Restricted,
            2 => AVAuthorizationStatus::Denied,
            3 => AVAuthorizationStatus::Authorized,
            _ => AVAuthorizationStatus::NotDetermined,
        }
    }
}

/// Request microphone permission from the user.
/// This triggers the macOS TCC dialog if permission has not yet been determined.
/// Blocks until the user makes a choice.
#[cfg(target_os = "macos")]
pub fn request_microphone_permission() -> bool {
    use block2::RcBlock;
    use std::sync::{Arc, Condvar, Mutex};

    let status = microphone_authorization_status();
    println!("[permissions] Current microphone status: {:?}", status);

    match status {
        AVAuthorizationStatus::Authorized => return true,
        AVAuthorizationStatus::Denied | AVAuthorizationStatus::Restricted => return false,
        AVAuthorizationStatus::NotDetermined => {
            // Need to ask the user
        }
    }

    println!("[permissions] Requesting microphone permission...");

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();
    let granted_result = Arc::new(Mutex::new(false));
    let granted_result2 = granted_result.clone();

    let block = RcBlock::new(move |granted: Bool| {
        let is_granted = granted.as_bool();
        *granted_result2.lock().unwrap() = is_granted;
        let (lock, cvar) = &*pair2;
        *lock.lock().unwrap() = true;
        cvar.notify_one();
    });

    unsafe {
        let media_type: Retained<NSString> = NSString::from_str("soun");
        let _: () = msg_send![
            class!(AVCaptureDevice),
            requestAccessForMediaType: &*media_type,
            completionHandler: &*block
        ];
    }

    // Wait for user response (blocks until they click Allow/Deny)
    let (lock, cvar) = &*pair;
    let mut done = lock.lock().unwrap();
    while !*done {
        done = cvar.wait(done).unwrap();
    }

    let granted = *granted_result.lock().unwrap();
    println!(
        "[permissions] Microphone permission {}",
        if granted { "GRANTED" } else { "DENIED" }
    );
    granted
}

/// Prompt the user to grant Accessibility permission if not yet authorized.
///
/// Unlike Microphone, Accessibility uses `AXIsProcessTrustedWithOptions`.
/// With `AXTrustedCheckOptionPrompt: true`, macOS shows a dialog directing
/// the user to System Settings → Privacy & Security → Accessibility.
///
/// Returns true if already trusted, false if not (user needs to toggle manually).
#[cfg(target_os = "macos")]
pub fn request_accessibility_permission() -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: core_foundation::base::CFTypeRef) -> bool;
    }

    // prompt: true → macOS shows the "App wants to control this computer" dialog
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

    let trusted = unsafe { AXIsProcessTrustedWithOptions(options.as_CFTypeRef()) };
    println!("[permissions] Accessibility trusted: {}", trusted);
    trusted
}

/// Check if screen recording permission is granted.
/// On macOS 10.15+, uses CGPreflightScreenCaptureAccess.
#[cfg(target_os = "macos")]
pub fn check_screen_recording_permission() -> bool {
    unsafe {
        // Try CGPreflightScreenCaptureAccess if available (macOS 10.15+)
        // This is the proper way to check without triggering a prompt
        extern "C" {
            fn CGPreflightScreenCaptureAccess() -> bool;
        }

        let has_permission = CGPreflightScreenCaptureAccess();
        println!("[permissions] Screen recording permission: {}", has_permission);
        has_permission
    }
}

/// Request screen recording permission by attempting a test capture.
/// This will trigger the system permission dialog if not yet granted.
#[allow(dead_code)]
#[cfg(target_os = "macos")]
pub fn request_screen_recording_permission() -> bool {
    use core_graphics::display::CGDisplay;

    // First check if we already have permission
    if check_screen_recording_permission() {
        return true;
    }

    println!("[permissions] Requesting screen recording permission...");

    // Attempt a capture to trigger the permission dialog
    let _ = CGDisplay::screenshot(
        CGDisplay::main().bounds(),
        0,
        0,
        0,
    );

    // Check again after the attempt
    let granted = check_screen_recording_permission();
    println!(
        "[permissions] Screen recording permission {}",
        if granted { "GRANTED" } else { "DENIED or PENDING" }
    );
    granted
}

// ============================================================================
// Windows Stub Implementations
// ============================================================================

/// Windows: Microphone permission is handled automatically by the system
/// on first use. No explicit request needed.
#[cfg(target_os = "windows")]
pub fn request_microphone_permission() -> bool {
    println!("[permissions] Windows microphone permission: auto-granted on first use");
    true
}

/// Windows: No accessibility permission system like macOS.
/// SendInput works without special permissions.
#[cfg(target_os = "windows")]
pub fn request_accessibility_permission() -> bool {
    println!("[permissions] Windows accessibility: no permission required");
    true
}

/// Windows: No screen recording permission system like macOS.
/// GDI screenshot APIs work without special permissions.
#[cfg(target_os = "windows")]
pub fn check_screen_recording_permission() -> bool {
    println!("[permissions] Windows screen recording: no permission required");
    true
}

/// Windows: Stub for screen recording permission request.
#[cfg(target_os = "windows")]
pub fn request_screen_recording_permission() -> bool {
    println!("[permissions] Windows screen recording: no permission required");
    true
}
