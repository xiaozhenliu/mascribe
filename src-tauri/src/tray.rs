use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;

pub fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_item = MenuItemBuilder::new("Quit Voice Input")
        .id("quit")
        .build(app)?;

    let menu = MenuBuilder::new(app).items(&[&quit_item]).build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Voice Input")
        .icon_as_template(true) // Adapts to macOS light/dark menu bar
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => {
                println!("[tray] Quit selected");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
