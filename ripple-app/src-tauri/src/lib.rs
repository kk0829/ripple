pub mod discovery;
pub mod logging;
pub mod messages;
pub mod websocket;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();

    tauri::Builder::default()
        .setup(|app| {
            discovery::register(app.handle());

            tauri::async_runtime::spawn(async move {
                if let Err(e) = websocket::start_server().await {
                    tracing::error!("WebSocket server failed: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            messages::send_message,
            messages::get_device_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
