use tauri::SystemTrayMenuItem;
use tauri::{AppHandle, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu};

use once_cell::sync::OnceCell;

pub(crate) static APP: OnceCell<AppHandle> = OnceCell::new();

// fn load_icon(path: &std::path::Path) -> tray_icon::icon::Icon {
// let (icon_rgba, icon_width, icon_height) = {
//     let image = image::open(path)
//         .expect("Failed to open icon path")
//         .into_rgba8();
//     let (width, height) = image.dimensions();
//     let rgba = image.into_raw();
//     (rgba, width, height)
// };
// tray_icon::icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
//     .expect("Failed to open icon")
// }

pub fn tray_init() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    tauri::Builder::default()
        .setup(|app| {
            let app = app.handle().clone();
            APP.set(app).unwrap();
            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let item_handle = app.tray_handle().get_item(&id);
                match id.as_str() {
                    "hide" => {
                        // let window = app.get_window("main").unwrap();
                        // window.hide().unwrap();
                        // // you can also `set_selected`, `set_enabled` and `set_native_image` (macOS only).
                        item_handle.set_title("Show").unwrap();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            }
            _ => {
                println!("other event");
            }
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
