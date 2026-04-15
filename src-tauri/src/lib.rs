use tauri::{LogicalPosition, LogicalSize, Manager, Position, Size, WebviewWindow};

const TOPBAR_HEIGHT: f64 = 40.0;

fn align_topbar_window(window: &WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        let scale_factor = monitor.scale_factor();
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();

        let width = f64::from(monitor_size.width) / scale_factor;
        let x = f64::from(monitor_position.x) / scale_factor;
        let y = f64::from(monitor_position.y) / scale_factor;

        window.set_position(Position::Logical(LogicalPosition::new(x, y)))?;
        window.set_size(Size::Logical(LogicalSize::new(width, TOPBAR_HEIGHT)))?;
    }

    window.set_always_on_top(true)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_toolwindow_style(window: &WebviewWindow) {
    use std::os::raw::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };

    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let hwnd = HWND(hwnd.0 as *mut c_void);
            let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            let next_style = (current_style & !WS_EX_APPWINDOW.0) | WS_EX_TOOLWINDOW.0;

            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next_style as isize);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("missing main window configuration");

            align_topbar_window(&window)?;

            #[cfg(target_os = "windows")]
            apply_toolwindow_style(&window);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
