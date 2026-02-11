// Native hotkey listener via CGEventTap.
//
// Used as fallback when tauri-plugin-global-shortcut doesn't support
// a particular key (e.g. ContextMenu). Runs on a dedicated background
// thread with its own CFRunLoop.

use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopRef};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Wrapper to make CFRunLoopRef sendable across threads.
/// Safety: CFRunLoop is thread-safe (retained reference, only used for CFRunLoopStop).
struct SendableRunLoop(CFRunLoopRef);
unsafe impl Send for SendableRunLoop {}

/// Handle to a running CGEventTap listener. Drop to stop.
pub struct NativeHotkeyHandle {
    /// Signal the background thread to exit
    stop_flag: Arc<AtomicBool>,
    /// The CFRunLoop of the background thread (for waking it up)
    run_loop: CFRunLoopRef,
}

// CFRunLoopRef is a raw pointer; safe to send across threads
unsafe impl Send for NativeHotkeyHandle {}
unsafe impl Sync for NativeHotkeyHandle {}

impl Drop for NativeHotkeyHandle {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        unsafe {
            core_foundation::runloop::CFRunLoopStop(self.run_loop);
        }
    }
}

/// Map a key name string to its macOS virtual keycode.
/// Only maps keys that global-hotkey doesn't support.
fn key_name_to_keycode(name: &str) -> Option<u16> {
    match name {
        "ContextMenu" => Some(0x6E), // 110
        _ => None,
    }
}

/// Start a native CGEventTap listener for a specific keycode.
///
/// The `on_press` callback is invoked on the background thread whenever
/// the target key is pressed. Returns a handle that stops the listener
/// when dropped.
pub fn start_native_listener<F>(key_name: &str, on_press: F) -> Result<NativeHotkeyHandle, String>
where
    F: Fn() + Send + 'static,
{
    let keycode = key_name_to_keycode(key_name)
        .ok_or_else(|| format!("unsupported native key: {}", key_name))?;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    // Channel to receive the CFRunLoopRef from the background thread
    let (tx, rx) = std::sync::mpsc::sync_channel::<SendableRunLoop>(1);

    thread::spawn(move || {
        let current_loop = CFRunLoop::get_current();
        // Send the run loop ref back to the main thread
        let loop_ref = current_loop.as_concrete_TypeRef();
        // Retain so it stays valid after this scope
        unsafe {
            core_foundation::base::CFRetain(loop_ref as *const _);
        }
        let _ = tx.send(SendableRunLoop(loop_ref));

        let tap_result = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown],
            move |_proxy, _etype, event| {
                let kc =
                    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
                if kc == keycode && !stop_flag_clone.load(Ordering::SeqCst) {
                    on_press();
                }
                // Return the event unchanged (ListenOnly — we don't consume it)
                Some(event.clone())
            },
        );

        match tap_result {
            Ok(tap) => unsafe {
                let loop_source = tap
                    .mach_port
                    .create_runloop_source(0)
                    .expect("[hotkey] failed to create runloop source");
                current_loop.add_source(&loop_source, kCFRunLoopCommonModes);
                tap.enable();
                println!(
                    "[hotkey] CGEventTap listening for keycode {} (key: native)",
                    keycode
                );
                // This blocks until CFRunLoopStop() is called from the handle
                CFRunLoop::run_current();
                println!("[hotkey] CGEventTap runloop exited");
            },
            Err(()) => {
                println!("[hotkey] CGEventTapCreate failed — check Accessibility permission");
            }
        }
    });

    // Wait for the background thread to send us the run loop ref
    let SendableRunLoop(run_loop) = rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .map_err(|_| "timeout waiting for CGEventTap thread to start".to_string())?;

    Ok(NativeHotkeyHandle {
        stop_flag,
        run_loop,
    })
}
