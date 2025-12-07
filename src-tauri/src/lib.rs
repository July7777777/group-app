use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use tauri::Manager;
// 定义存储数据的结构 - 简化版本，只保留cache_groups和waiting_groups
#[derive(Serialize, Deserialize, Debug)]
struct AppData {
    cache_groups: Vec<String>,
    waiting_groups: Vec<String>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 保存应用数据 - 简化版本
#[tauri::command]
fn save_app_data(cache_groups: Vec<String>, waiting_groups: Vec<String>) -> Result<(), String> {
    let app_data = AppData {
        cache_groups,
        waiting_groups,
    };

    let json_data = serde_json::to_string(&app_data)
        .map_err(|e| format!("序列化数据失败: {}", e))?;

    // 使用当前目录保存数据文件
    let data_file = PathBuf::from("group_app_data.json");

    fs::write(&data_file, json_data)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

// 加载应用数据 - 简化版本
#[tauri::command]
fn load_app_data() -> Result<AppData, String> {
    let data_file = PathBuf::from("group_app_data.json");

    if !data_file.exists() {
        let default_names: Vec<String> = vec![
            "人员1".to_string(), "人员2".to_string(), "人员3".to_string(),
            "人员4".to_string(), "人员5".to_string(), "人员6".to_string(),
            "人员7".to_string(), "人员8".to_string(),
        ];
        return Ok(AppData {
            cache_groups: default_names,
            waiting_groups: vec![],
        });
    }

    let json_data = fs::read_to_string(&data_file)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let app_data: AppData = serde_json::from_str(&json_data)
        .map_err(|e| format!("解析数据失败: {}", e))?;

    Ok(app_data)
}

#[tauri::command]
async fn create_new_window(
    app: AppHandle,
    group_name: String,
    group_index: u32,
) -> Result<(), String> {
    let window_label = format!("result_window_{}", group_index);

    // 使用 get_webview_window（不是 get_window）
    if app.get_webview_window(&window_label).is_some() {
        return Ok(());
    }

    // 使用 WebviewWindowBuilder + WebviewUrl
    WebviewWindowBuilder::new(
        &app,
        window_label,
        WebviewUrl::App(format!("result?groupName={}&groupIndex={}",
            urlencoding::encode(&group_name),
            group_index
        ).into())
    )
    .title(format!("结果 - {}", group_name))
    .inner_size(900.0, 700.0)
    .resizable(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            save_app_data,
            load_app_data,
            // 👇 新增：注册新命令
            create_new_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}