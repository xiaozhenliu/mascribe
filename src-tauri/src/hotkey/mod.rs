//! Cross-platform hotkey listener.
//!
//! Provides a unified interface for global hotkey registration on macOS and Windows.
//! Falls back to native APIs when tauri-plugin-global-shortcut doesn't support a key.

pub mod keycode;

pub use keycode::{HotkeyDefinition, Key, Modifier, parse_hotkey, normalize_for_platform};

/// Platform-specific native hotkey handle.
#[cfg(target_os = "macos")]
pub type NativeHotkeyHandle = macos::MacOSHotkeyHandle;

#[cfg(target_os = "windows")]
pub type NativeHotkeyHandle = windows::WindowsHotkeyHandle;

/// Start a native hotkey listener for the given hotkey definition.
///
/// The `on_press` callback is invoked whenever the hotkey is pressed.
/// Returns a handle that stops the listener when dropped.
pub fn start_native_listener<F>(hotkey: &HotkeyDefinition, on_press: F) -> Result<NativeHotkeyHandle, String>
where
    F: Fn() + Send + 'static,
{
    #[cfg(target_os = "macos")]
    return macos::start_native_listener(hotkey, on_press);

    #[cfg(target_os = "windows")]
    return windows::start_native_listener(hotkey, on_press);

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Native hotkey not supported on this platform".to_string())
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{HotkeyDefinition, Key, Modifier};
    use core_foundation::base::TCFType;
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopRef};
    use core_graphics::event::CGEventFlags;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    /// Wrapper to make CFRunLoopRef sendable across threads.
    struct SendableRunLoop(CFRunLoopRef);
    unsafe impl Send for SendableRunLoop {}

    /// Handle to a running macOS hotkey listener. Drop to stop.
    pub struct MacOSHotkeyHandle {
        stop_flag: Arc<AtomicBool>,
        run_loop: CFRunLoopRef,
    }

    unsafe impl Send for MacOSHotkeyHandle {}
    unsafe impl Sync for MacOSHotkeyHandle {}

    impl Drop for MacOSHotkeyHandle {
        fn drop(&mut self) {
            self.stop_flag.store(true, Ordering::SeqCst);
            unsafe {
                core_foundation::runloop::CFRunLoopStop(self.run_loop);
            }
        }
    }

    /// Convert our Key enum to macOS keycode.
    fn key_to_keycode(key: &Key) -> Option<u16> {
        use super::Key;
        match key {
            Key::A => Some(0x00),
            Key::S => Some(0x01),
            Key::D => Some(0x02),
            Key::F => Some(0x03),
            Key::H => Some(0x04),
            Key::G => Some(0x05),
            Key::Z => Some(0x06),
            Key::X => Some(0x07),
            Key::C => Some(0x08),
            Key::V => Some(0x09),
            Key::B => Some(0x0B),
            Key::Q => Some(0x0C),
            Key::W => Some(0x0D),
            Key::E => Some(0x0E),
            Key::R => Some(0x0F),
            Key::Y => Some(0x10),
            Key::T => Some(0x11),
            Key::Num1 => Some(0x12),
            Key::Num2 => Some(0x13),
            Key::Num3 => Some(0x14),
            Key::Num4 => Some(0x15),
            Key::Num6 => Some(0x16),
            Key::Num5 => Some(0x17),
            Key::Equal => Some(0x18),
            Key::Num9 => Some(0x19),
            Key::Num7 => Some(0x1A),
            Key::Minus => Some(0x1B),
            Key::Num8 => Some(0x1C),
            Key::Num0 => Some(0x1D),
            Key::BracketRight => Some(0x1E),
            Key::O => Some(0x1F),
            Key::U => Some(0x20),
            Key::BracketLeft => Some(0x21),
            Key::I => Some(0x22),
            Key::P => Some(0x23),
            Key::L => Some(0x25),
            Key::J => Some(0x26),
            Key::Quote => Some(0x27),
            Key::K => Some(0x28),
            Key::Semicolon => Some(0x29),
            Key::Backslash => Some(0x2A),
            Key::Comma => Some(0x2B),
            Key::Slash => Some(0x2C),
            Key::N => Some(0x2D),
            Key::M => Some(0x2E),
            Key::Period => Some(0x2F),
            Key::Backtick => Some(0x32),
            Key::NumpadDecimal => Some(0x41),
            Key::NumpadMultiply => Some(0x43),
            Key::NumpadPlus => Some(0x45),
            // Note: NumpadClear (0x47) not in Key enum
            Key::NumpadDivide => Some(0x4B),
            Key::NumpadEnter => Some(0x4C),
            Key::NumpadMinus => Some(0x4E),
            Key::Numpad0 => Some(0x52),
            Key::Numpad1 => Some(0x53),
            Key::Numpad2 => Some(0x54),
            Key::Numpad3 => Some(0x55),
            Key::Numpad4 => Some(0x56),
            Key::Numpad5 => Some(0x57),
            Key::Numpad6 => Some(0x58),
            Key::Numpad7 => Some(0x59),
            Key::Numpad8 => Some(0x5B),
            Key::Numpad9 => Some(0x5C),
            Key::Enter => Some(0x24),
            Key::Tab => Some(0x30),
            Key::Space => Some(0x31),
            Key::Delete => Some(0x33),
            Key::Escape => Some(0x35),
            Key::FKey(n) => match n {
                1 => Some(0x7A),
                2 => Some(0x78),
                3 => Some(0x63),
                4 => Some(0x76),
                5 => Some(0x60),
                6 => Some(0x61),
                7 => Some(0x62),
                8 => Some(0x64),
                9 => Some(0x65),
                10 => Some(0x6D),
                11 => Some(0x67),
                12 => Some(0x6F),
                13 => Some(0x69),
                14 => Some(0x6B),
                15 => Some(0x71),
                16 => Some(0x6E),
                17 => Some(0x40),
                18 => Some(0x4F),
                19 => Some(0x50),
                20 => Some(0x5A),
                _ => None,
            },
            Key::Home => Some(0x73),
            Key::PageUp => Some(0x74),
            Key::End => Some(0x77),
            Key::PageDown => Some(0x79),
            Key::Left => Some(0x7B),
            Key::Right => Some(0x7C),
            Key::Down => Some(0x7D),
            Key::Up => Some(0x7E),
            Key::ContextMenu => Some(0x6E), // macOS 上没有真正的 ContextMenu 键，使用 F16 作为替代
            _ => None,
        }
    }

    /// Convert our modifiers to CGEventFlags.
    fn modifiers_to_flags(modifiers: &[Modifier]) -> CGEventFlags {
        use super::Modifier;
        use core_graphics::event::CGEventFlags;
        let mut flags = CGEventFlags::empty();
        for m in modifiers {
            match m {
                Modifier::Ctrl => flags |= CGEventFlags::CGEventFlagControl,
                Modifier::Shift => flags |= CGEventFlags::CGEventFlagShift,
                Modifier::Alt => flags |= CGEventFlags::CGEventFlagAlternate,
                Modifier::Meta => flags |= CGEventFlags::CGEventFlagCommand,
            }
        }
        flags
    }

    pub fn start_native_listener<F>(hotkey: &HotkeyDefinition, on_press: F) -> Result<MacOSHotkeyHandle, String>
    where
        F: Fn() + Send + 'static,
    {
        use core_graphics::event::{
            CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
            EventField,
        };

        let keycode = key_to_keycode(&hotkey.key)
            .ok_or_else(|| format!("unsupported key: {}", hotkey.key))?;

        let target_flags = modifiers_to_flags(&hotkey.modifiers);

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let (tx, rx) = std::sync::mpsc::sync_channel::<SendableRunLoop>(1);

        // Clone hotkey data for the thread
        let hotkey_key = hotkey.key;

        thread::spawn(move || {
            let current_loop = CFRunLoop::get_current();
            let loop_ref = current_loop.as_concrete_TypeRef();
            unsafe {
                core_foundation::base::CFRetain(loop_ref as *const _);
            }
            let _ = tx.send(SendableRunLoop(loop_ref));

            let tap_result = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![CGEventType::KeyDown],
                move |_proxy, _etype, event| {
                    let kc = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
                    let flags = event.get_flags();

                    // Check if keycode matches and all target modifiers are present
                    if kc == keycode
                        && !stop_flag_clone.load(Ordering::SeqCst)
                        && (flags & target_flags) == target_flags
                    {
                        on_press();
                        // Swallow the event so it doesn't reach the active app
                        return None;
                    }
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
                    println!("[hotkey] CGEventTap listening for key {:?}", hotkey_key);
                    CFRunLoop::run_current();
                    println!("[hotkey] CGEventTap runloop exited");
                },
                Err(()) => {
                    println!("[hotkey] CGEventTapCreate failed — check Accessibility permission");
                }
            }
        });

        let SendableRunLoop(run_loop) = rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .map_err(|_| "timeout waiting for CGEventTap thread to start".to_string())?;

        Ok(MacOSHotkeyHandle {
            stop_flag,
            run_loop,
        })
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::{HotkeyDefinition, Key, Modifier};
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    static ACTIVE_HOOK: Lazy<Mutex<Option<HHOOK>>> = Lazy::new(|| Mutex::new(None));
    static STOP_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
    static CALLBACK: Lazy<Mutex<Option<Box<dyn Fn() + Send>>>> = Lazy::new(|| Mutex::new(None));
    static TARGET_HOTKEY: Lazy<Mutex<Option<(VIRTUAL_KEY, u16)>>> = Lazy::new(|| Mutex::new(None));

    /// Handle to a running Windows hotkey listener. Drop to stop.
    pub struct WindowsHotkeyHandle;

    impl Drop for WindowsHotkeyHandle {
        fn drop(&mut self) {
            STOP_FLAG.store(true, Ordering::SeqCst);
            if let Ok(mut hook) = ACTIVE_HOOK.lock() {
                if let Some(h) = hook.take() {
                    unsafe {
                        let _ = UnhookWindowsHookEx(h);
                    }
                }
            }
            *CALLBACK.lock().unwrap() = None;
            *TARGET_HOTKEY.lock().unwrap() = None;
            println!("[hotkey] Windows hook stopped");
        }
    }

    /// Convert our Key enum to Windows virtual key code.
    fn key_to_vk(key: &Key) -> Option<VIRTUAL_KEY> {
        match key {
            Key::A => Some(VK_A),
            Key::B => Some(VK_B),
            Key::C => Some(VK_C),
            Key::D => Some(VK_D),
            Key::E => Some(VK_E),
            Key::F => Some(VK_F),
            Key::G => Some(VK_G),
            Key::H => Some(VK_H),
            Key::I => Some(VK_I),
            Key::J => Some(VK_J),
            Key::K => Some(VK_K),
            Key::L => Some(VK_L),
            Key::M => Some(VK_M),
            Key::N => Some(VK_N),
            Key::O => Some(VK_O),
            Key::P => Some(VK_P),
            Key::Q => Some(VK_Q),
            Key::R => Some(VK_R),
            Key::S => Some(VK_S),
            Key::T => Some(VK_T),
            Key::U => Some(VK_U),
            Key::V => Some(VK_V),
            Key::W => Some(VK_W),
            Key::X => Some(VK_X),
            Key::Y => Some(VK_Y),
            Key::Z => Some(VK_Z),
            Key::Num0 => Some(VK_0),
            Key::Num1 => Some(VK_1),
            Key::Num2 => Some(VK_2),
            Key::Num3 => Some(VK_3),
            Key::Num4 => Some(VK_4),
            Key::Num5 => Some(VK_5),
            Key::Num6 => Some(VK_6),
            Key::Num7 => Some(VK_7),
            Key::Num8 => Some(VK_8),
            Key::Num9 => Some(VK_9),
            Key::FKey(n) => match n {
                1 => Some(VK_F1),
                2 => Some(VK_F2),
                3 => Some(VK_F3),
                4 => Some(VK_F4),
                5 => Some(VK_F5),
                6 => Some(VK_F6),
                7 => Some(VK_F7),
                8 => Some(VK_F8),
                9 => Some(VK_F9),
                10 => Some(VK_F10),
                11 => Some(VK_F11),
                12 => Some(VK_F12),
                13 => Some(VK_F13),
                14 => Some(VK_F14),
                15 => Some(VK_F15),
                16 => Some(VK_F16),
                17 => Some(VK_F17),
                18 => Some(VK_F18),
                19 => Some(VK_F19),
                20 => Some(VK_F20),
                21 => Some(VK_F21),
                22 => Some(VK_F22),
                23 => Some(VK_F23),
                24 => Some(VK_F24),
                _ => None,
            },
            Key::Escape => Some(VK_ESCAPE),
            Key::Space => Some(VK_SPACE),
            Key::Enter => Some(VK_RETURN),
            Key::Tab => Some(VK_TAB),
            Key::Backspace => Some(VK_BACK),
            Key::Delete => Some(VK_DELETE),
            Key::Home => Some(VK_HOME),
            Key::End => Some(VK_END),
            Key::PageUp => Some(VK_PRIOR),
            Key::PageDown => Some(VK_NEXT),
            Key::Up => Some(VK_UP),
            Key::Down => Some(VK_DOWN),
            Key::Left => Some(VK_LEFT),
            Key::Right => Some(VK_RIGHT),
            Key::Comma => Some(VK_OEM_COMMA),
            Key::Period => Some(VK_OEM_PERIOD),
            Key::Slash => Some(VK_OEM_2),
            Key::Semicolon => Some(VK_OEM_1),
            Key::Quote => Some(VK_OEM_7),
            Key::Backslash => Some(VK_OEM_5),
            Key::BracketLeft => Some(VK_OEM_4),
            Key::BracketRight => Some(VK_OEM_6),
            Key::Minus => Some(VK_OEM_MINUS),
            Key::Equal => Some(VK_OEM_PLUS),
            Key::Backtick => Some(VK_OEM_3),
            _ => None,
        }
    }

    /// Get modifier state as a bitmask.
    fn get_modifier_state() -> u16 {
        let mut state = 0u16;
        unsafe {
            if GetAsyncKeyState(VK_CONTROL.0 as i32) < 0 {
                state |= 1;
            }
            if GetAsyncKeyState(VK_SHIFT.0 as i32) < 0 {
                state |= 2;
            }
            if GetAsyncKeyState(VK_MENU.0 as i32) < 0 {
                state |= 4;
            }
            if GetAsyncKeyState(VK_LWIN.0 as i32) < 0 || GetAsyncKeyState(VK_RWIN.0 as i32) < 0 {
                state |= 8;
            }
        }
        state
    }

    /// Convert our modifiers to expected state.
    fn modifiers_to_state(modifiers: &[Modifier]) -> u16 {
        let mut state = 0u16;
        for m in modifiers {
            match m {
                Modifier::Ctrl => state |= 1,
                Modifier::Shift => state |= 2,
                Modifier::Alt => state |= 4,
                Modifier::Meta => state |= 8,
            }
        }
        state
    }

    extern "system" fn keyboard_hook_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if n_code >= 0 && w_param.0 as u32 == WM_KEYDOWN {
            let kbd_struct = unsafe { &*(l_param.0 as *const KBDLLHOOKSTRUCT) };
            let vk = VIRTUAL_KEY(kbd_struct.vkCode as u16);

            if let Ok(target) = TARGET_HOTKEY.lock() {
                if let Some((target_vk, target_mods)) = *target {
                    if vk == target_vk {
                        let current_mods = get_modifier_state();
                        // Check if all target modifiers are pressed
                        if (current_mods & target_mods) == target_mods {
                            if let Ok(callback) = CALLBACK.lock() {
                                if let Some(ref cb) = *callback {
                                    cb();
                                }
                            }
                        }
                    }
                }
            }
        }

        unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
    }

    pub fn start_native_listener<F>(hotkey: &HotkeyDefinition, on_press: F) -> Result<WindowsHotkeyHandle, String>
    where
        F: Fn() + Send + 'static,
    {
        let vk = key_to_vk(&hotkey.key)
            .ok_or_else(|| format!("unsupported key: {}", hotkey.key))?;

        let mod_state = modifiers_to_state(&hotkey.modifiers);

        // Store callback and target hotkey
        *CALLBACK.lock().unwrap() = Some(Box::new(on_press));
        *TARGET_HOTKEY.lock().unwrap() = Some((vk, mod_state));
        STOP_FLAG.store(false, Ordering::SeqCst);

        // Install hook
        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                GetModuleHandleW(None).map_err(|e| format!("GetModuleHandle failed: {:?}", e))?,
                0,
            )
        };

        if hook.is_err() {
            return Err("Failed to install keyboard hook".to_string());
        }

        let hook = hook.unwrap();
        *ACTIVE_HOOK.lock().unwrap() = Some(hook);

        println!("[hotkey] Windows hook installed for {:?}", hotkey);

        Ok(WindowsHotkeyHandle)
    }
}
