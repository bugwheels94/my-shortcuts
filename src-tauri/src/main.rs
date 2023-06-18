// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        // .setup(|app| {
        //     // listen to the `event-name` (emitted on any window)
        //     let id = app.listen_global("event-name", |event| {
        //       println!("got event-name with payload {:?}", event.payload());
        //     });
        //     // unlisten to the event using the `id` returned on the `listen_global` function
        //     // a `once_global` API is also exposed on the `App` struct
        //     // app.unlisten(id);
        //     // emit the `event-name` event to all webview windows on the frontend
        //     // app.emit_all("event-name", Payload { message: "Tauri is awesome!".into() }).unwrap();
        //     Ok(())
        //   })
        .invoke_handler(tauri::generate_handler![greet, open_docs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn open_docs(handle: tauri::AppHandle, invoke_message: String) {
    let docs_window = tauri::WindowBuilder::new(
        &handle,
        "external", /* the unique window label */
        tauri::WindowUrl::External(invoke_message.parse().unwrap()),
    )
    .build()
    .unwrap();
}
