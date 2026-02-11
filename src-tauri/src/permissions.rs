//! macOS permission helpers.
//!
//! Two permissions are needed at startup:
//!
//! 1. **Microphone** — cpal uses CoreAudio directly, which does NOT trigger the
//!    macOS TCC dialog. We call AVCaptureDevice.requestAccess via Objective-C.
//!
//! 2. **Accessibility** — needed for CGEvent.post() to simulate Cmd+V. We call
//!    AXIsProcessTrustedWithOptions(prompt: true) to show the system dialog.

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
