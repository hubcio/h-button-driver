pub mod tray_menu;
use self::tray_menu::APP;

use tauri::Icon;

pub enum TrayIcon {
    Muted,
    Unmuted,
}

pub fn change_title_test1() {
    let item = APP.get().unwrap().tray_handle().get_item("hide");
    item.set_title("XDDDDDDDDD").unwrap();
}

pub fn change_title_test2() {
    let item = APP.get().unwrap().tray_handle().get_item("hide");
    item.set_title("XD").unwrap();
}

pub fn change_icon(icon: TrayIcon) {
    let item = APP.get().unwrap().tray_handle();
    match icon {
        TrayIcon::Muted => {
            // println!("set muted icon muted");
            item.set_icon(Icon::Raw(
                include_bytes!("../../icons/unmuted.png").to_vec(),
            ))
            .unwrap();
        }
        TrayIcon::Unmuted => {
            // println!("set icon.png icon muted");
            item.set_icon(Icon::Raw(include_bytes!("../../icons/icon.png").to_vec()))
                .unwrap();
        }
    }
}
