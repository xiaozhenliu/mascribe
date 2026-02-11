use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{ActivationPolicy, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItemBuilder::new("Settings...")
        .id("settings")
        .build(app)?;

    let quit_item = MenuItemBuilder::new("Quit Voice Input")
        .id("quit")
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&settings_item, &quit_item])
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Voice Input")
        .icon_as_template(true) // Adapts to macOS light/dark menu bar
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
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
                    .title("Voice Input Settings")
                    .inner_size(520.0, 480.0)
                    .resizable(false)
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
