#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::image::Image;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::{
    menu::{Menu, MenuItem},
    AppHandle, WebviewWindowBuilder,
};

use image::ImageReader;

fn decode_icon() -> Image<'static> {
    // Load and decode the PNG image
    let img = ImageReader::new(std::io::Cursor::new(include_bytes!("../icons/icon.png")))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();

    let (width, height) = img.dimensions();
    let raw = img.into_raw();

    Image::new_owned(raw, width, height)
}

#[derive(Deserialize, Debug)]
struct Icon {
    url: String,
    name: String,
    allowMultipleInstances: String,
}

#[derive(Deserialize, Debug)]
struct JsonRequest {
    content: HashMap<String, Vec<Icon>>,
    webview: String,
}

#[derive(Serialize)]
struct JsonResponse {
    success: bool,
    message: String,
}

#[tauri::command]
async fn open_icon(app: AppHandle, invoke_message: String, label: String, webview: String) {
    open(&app, invoke_message, label, webview).await;
}

async fn open(app: &AppHandle, invoke_message: String, label: String, webview: String) {
    println!("Opening-- {} in {} mode", invoke_message, webview);

    if webview == "edge" || webview == "chrome" || webview == "firefox" {
        // Open via external browser in app mode if supported
        let _ = runCommand(webview, invoke_message.parse().unwrap(), app);
    } else {
        // Open inside a Tauri external window
        let _ = WebviewWindowBuilder::new(
            app,
            label,
            tauri::WebviewUrl::External(invoke_message.parse().unwrap()),
        )
        .maximized(true)
        .visible(true)
        .build();
    }
}

use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::process::Command;
fn runCommand(webview: String, url: String, handle: &tauri::AppHandle) -> Result<(), Error> {
    println!("Starting runCommand for webview: {}, url: {}", webview, url);

    let app;
    let data_directory = handle
        .path()
        .app_data_dir()
        .expect("Failed to get data directory");
    let file_path = data_directory.join("my-shortcut-log.txt");

    println!("Log file will be written to: {:?}", file_path);

    let mut file = File::create(&file_path).expect("Failed to create file");

    app = match webview.as_str() {
        "edge" => match std::env::consts::OS {
            "linux" => "microsoft-edge-stable",
            "macos" => "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            "windows" => "cmd",
            _ => return Err(Error::new(std::io::ErrorKind::Other, "Unsupported OS")),
        },
        "chrome" => match std::env::consts::OS {
            "linux" => "google-chrome",
            "macos" => "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "windows" => "cmd",
            _ => return Err(Error::new(std::io::ErrorKind::Other, "Unsupported OS")),
        },
        "firefox" => match std::env::consts::OS {
            "linux" => "firefox",
            "macos" => "/Applications/Firefox.app/Contents/MacOS/firefox",
            "windows" => "cmd",
            _ => return Err(Error::new(std::io::ErrorKind::Other, "Unsupported OS")),
        },
        _ => "",
    };

    println!("Resolved browser command: {}", app);

    let mut cmd = Command::new(app);
    let s = "--app=".to_string() + &url;

    match std::env::consts::OS {
        "linux" => match webview.as_str() {
            "edge" => cmd.arg(&s),
            "chrome" => cmd.arg(&s),
            "firefox" => cmd.arg(url),
            _ => &mut cmd,
        },
        "macos" => match webview.as_str() {
            "edge" => cmd.arg(&s),
            "chrome" => cmd.arg(&s),
            "firefox" => cmd.arg(url),
            _ => &mut cmd,
        },
        "windows" => match webview.as_str() {
            "edge" => cmd
                .arg("/C")
                .arg("start")
                .arg("/B")
                .arg("msedge.exe")
                .arg(&s),
            "chrome" => cmd
                .arg("/C")
                .arg("start")
                .arg("/B")
                .arg("chrome.exe")
                .arg(&s),
            _ => &mut cmd,
        },
        _ => return Err(Error::new(std::io::ErrorKind::Other, "Unsupported OS")),
    };

    println!("Executing command: {:?}", cmd);

    let output = cmd.output()?;

    println!("Command executed. Success: {}", output.status.success());

    if output.status.success() {
        file.write_all(&output.stdout)?;
    } else {
        file.write_all(&output.stderr)?;
    }

    println!("Command output written to log.");

    Ok(())
}

#[tauri::command]
fn load_json(app: AppHandle, request: JsonRequest) -> Result<JsonResponse, String> {
    use tauri::menu::{Menu, MenuItem, Submenu};

    let mut submenus = vec![];

    for (category, icons) in request.content {
        // 1. Create items for the current category
        let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<_>>> = Vec::new();

        for icon in icons {
            let separator = ":_::_:";
            let id = format!(
                "{}-{}{}{}{}{}{}{}",
                category,
                icon.name,
                separator,
                icon.allowMultipleInstances,
                separator,
                icon.url,
                separator,
                request.webview
            );

            if let Ok(item) = MenuItem::with_id(&app, id, &icon.name, true, None::<&str>) {
                items.push(Box::new(item));
            }
        }

        // 2. Create submenu for this category
        // let submenu = Menu::with_items(&app, &items.iter().map(|i| &**i).collect::<Vec<_>>())
        //     .map_err(|e| format!("submenu creation failed: {e}"))?;
        let submenu = Submenu::with_items(
            &app,
            &category,
            true,
            &items.iter().map(|i| &**i).collect::<Vec<_>>(),
        )
        .map_err(|e| format!("subsubmenu failed: {e}"))?;

        submenus.push(Box::new(submenu) as Box<dyn tauri::menu::IsMenuItem<_>>);
    }

    // 3. Optionally add static menu items like "Show"
    if let Ok(show) = MenuItem::with_id(&app, "show", "Show", true, None::<&str>) {
        submenus.insert(0, Box::new(show));
    }

    // 4. Final tray menu
    let menu = Menu::with_items(&app, &submenus.iter().map(|i| &**i).collect::<Vec<_>>())
        .map_err(|e| format!("menu creation failed: {e}"))?;

    // 5. Update tray
    if let Some(tray) = app.tray_by_id("menu") {
        tray.set_menu(Some(menu))
            .map_err(|e| format!("tray set_menu failed: {e}"))?;
    }

    Ok(JsonResponse {
        success: true,
        message: "Icons loaded successfully".to_string(),
    })
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_icon, load_json])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            TrayIconBuilder::with_id("menu")
                .icon(decode_icon())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();

                    match id {
                        "quit" => {
                            println!("quit menu item was clicked");
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {
                            println!("handling menu item: {:?}", id);
                            let separator = ":_::_:";
                            let parts: Vec<&str> = id.split(separator).collect();
                            println!("handling1 menu item: {:?}", parts.len());

                            if parts.len() == 4 {
                                let label = if parts[1] == "true" {
                                    format!(
                                        "{}{}",
                                        parts[0],
                                        Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
                                    )
                                } else {
                                    parts[0].to_string()
                                };

                                let invoke_message = parts[2].to_string();
                                let webview = parts[3].to_string();
                                println!("Opening2 {} in {} mode", label, webview);

                                if app.get_webview_window(&label).is_none() {
                                    let app_handle = app.clone();
                                    println!("Opening1 {} in {} mode", invoke_message, webview);

                                    tauri::async_runtime::spawn(async move {
                                        println!("Opening {} in {} mode", invoke_message, webview);
                                        open(&app_handle, invoke_message, label, webview).await;
                                    });
                                }
                            }
                        }
                    }
                })
                .build(app)?; // <-- No changes here

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri app")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
