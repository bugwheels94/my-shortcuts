// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn open_icon(
    handle: tauri::AppHandle,
    invoke_message: String,
    label: String,
    webview: String,
) {
    open(&handle, invoke_message, label, webview);
}
fn open(handle: &tauri::AppHandle, invoke_message: String, label: String, webview: String) {
    if webview == "edge" || webview == "chrome" || webview == "firefox" {
        let _ = runCommand(webview, invoke_message.parse().unwrap(), handle);
    } else {
        tauri::WindowBuilder::new(
            handle,
            label, /* the unique window label */
            tauri::WindowUrl::External(invoke_message.parse().unwrap()),
        )
        .maximized(true)
        .build()
        .unwrap();
    }
}

use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::process::Command;
use tauri::api::path::app_data_dir;
fn runCommand(webview: String, url: String, handle: &tauri::AppHandle) -> Result<(), Error> {
    println!("Starting runCommand for webview: {}, url: {}", webview, url);

    let app;
    let data_directory = app_data_dir(&handle.config()).expect("Failed to get data directory");
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
        "linux" => cmd.arg("--app").arg(url),
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

use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashMap;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTraySubmenu,
};
fn main() {
    let tray_menu = SystemTrayMenu::new();

    let system_tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if event.window().label() == "main" {
                    event.window().hide().unwrap();
                    api.prevent_close();
                }
            }
            _ => {}
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                // app.get_window("main").unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    let _ = window.set_focus();
                }
                _ => {
                    let separator = ":_::_:";
                    let parts: Vec<&str> = id.split(separator).collect();

                    if !app.get_window(parts[0]).is_some() {
                        let label;
                        if parts[1] == "true" {
                            label = format!(
                                "{}{}",
                                parts[0],
                                Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
                            );
                        } else {
                            label = format!("{}", parts[0]);
                        }
                        open(app, parts[2].to_string(), label, "firefox".to_string());
                    }
                }
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![open_icon, load_json])
        .system_tray(system_tray)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
    // .run(tauri::generate_context!())
}

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct JsonRequest {
    // meta: Meta,
    content: HashMap<String, Vec<Icon>>,
}

#[derive(Deserialize, Debug)]
struct Meta {
    // gistId: String,
    // token: String,
}

#[derive(Deserialize, Debug)]
struct Icon {
    url: String,
    // icon: String,
    name: String,
    allowMultipleInstances: String,
}
#[derive(Serialize)]
struct JsonResponse {
    // Define the structure of your JSON response
    success: bool,
    message: String,
}

#[tauri::command]
fn load_json(handle: tauri::AppHandle, request: JsonRequest) -> Result<JsonResponse, String> {
    // Process the received JSON request
    // let gist_id = request.meta.gistId;
    // println!("Received JSON request: {:?}", gist_id);

    // empty_menu2.add_item(quit);
    let show = CustomMenuItem::new("show".to_string(), "Show");

    let mut new_menu = SystemTrayMenu::new().add_item(show);

    for (category, icons) in request.content {
        let mut empty_menu = SystemTrayMenu::new();

        for icon in icons {
            let separator = ":_::_:";
            let concatenated_string = format!(
                "{}-{}{}{}{}{}",
                category, icon.name, separator, icon.allowMultipleInstances, separator, icon.url
            );
            let quit = CustomMenuItem::new(concatenated_string, icon.name.clone());

            empty_menu = empty_menu.add_item(quit);
        }
        let submenu = SystemTraySubmenu::new(category, empty_menu);
        new_menu = new_menu.add_submenu(submenu);
    }

    let _aa = handle.tray_handle().set_menu(new_menu);

    let response = JsonResponse {
        success: true,
        message: "Request processed successfully".to_string(),
    };

    Ok(response)
}
