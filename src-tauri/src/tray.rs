use tauri::image::Image;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{ActivationPolicy, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

pub fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Check current autostart state
    let autostart_manager = app.autolaunch();
    let is_autostart = autostart_manager.is_enabled().unwrap_or(false);

    let launch_at_login = CheckMenuItemBuilder::new("Launch at Login")
        .id("launch-at-login")
        .checked(is_autostart)
        .build(app)?;

    let settings_item = MenuItemBuilder::new("Settings...")
        .id("settings")
        .build(app)?;

    let quit_item = MenuItemBuilder::new("Quit MaScribe")
        .id("quit")
        .build(app)?;

    let separator = PredefinedMenuItem::separator(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&launch_at_login, &separator, &settings_item, &separator, &quit_item])
        .build()?;

    // Use dedicated tray icon (black outline, transparent background)
    // so macOS template mode renders it correctly in light/dark menu bars.
    let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
        .unwrap_or_else(|_| app.default_window_icon().unwrap().clone());

    TrayIconBuilder::new()
        .icon(tray_icon)
        .tooltip("MaScribe")
        .icon_as_template(true) // Adapts to macOS light/dark menu bar
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "launch-at-login" => {
                let manager = app.autolaunch();
                let currently_enabled = manager.is_enabled().unwrap_or(false);
                if currently_enabled {
                    let _ = manager.disable();
                    println!("[tray] Autostart disabled");
                } else {
                    let _ = manager.enable();
                    println!("[tray] Autostart enabled");
                }
            }
            "settings" => {
                println!("[tray] Settings selected");

                // Temporarily become a Regular app so macOS activates us properly.
                // Without this, the settings window flashes and disappears because
                // Accessory apps don't get foreground activation.
                let _ = app.set_activation_policy(ActivationPolicy::Regular);

                // If settings window already exists, just focus it
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    // Create settings window on demand
                    match WebviewWindowBuilder::new(
                        app,
                        "settings",
                        WebviewUrl::App("settings.html".into()),
                    )
                    .title("MaScribe Settings")
                    .inner_size(520.0, 580.0)
                    .min_inner_size(520.0, 400.0)
                    .resizable(true)
                    .center()
                    .build()
                    {
                        Ok(window) => {
                            println!("[tray] Settings window created");
                            // When settings window is closed, revert to Accessory
                            let app_handle = app.clone();
                            window.on_window_event(move |event| {
                                if let tauri::WindowEvent::Destroyed = event {
                                    println!("[tray] Settings window closed, reverting to Accessory");
                                    let _ = app_handle.set_activation_policy(ActivationPolicy::Accessory);
                                }
                            });
                        }
                        Err(e) => {
                            println!("[tray] Failed to create settings window: {}", e);
                            // Revert on failure
                            let _ = app.set_activation_policy(ActivationPolicy::Accessory);
                        }
                    }
                }
            }
            "quit" => {
                println!("[tray] Quit selected");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
