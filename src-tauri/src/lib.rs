mod analysis;
mod commands;
mod epub_parser;
mod export;
mod llm;
mod models;
mod outline;
mod prompt;
mod storage;
mod token_utils;
mod txt_parser;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use storage::Database;
use tauri::Manager;

pub(crate) struct AppState {
    pub db: Mutex<Database>,
    pub batch_cancel: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let db = Database::new(&app_data_dir)
                .map_err(|e| format!("数据库初始化失败: {}", e))
                .expect("Failed to initialize database");
            app.manage(AppState {
                db: Mutex::new(db),
                batch_cancel: AtomicBool::new(false),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::novel::list_novels,
            commands::novel::preview_epub,
            commands::novel::import_epub_selected,
            commands::novel::import_txt_files,
            commands::novel::import_single_txt,
            commands::novel::delete_novel,
            commands::novel::get_novel,
            commands::chapter::list_chapters,
            commands::chapter::get_chapter,
            commands::chapter::get_chapter_content,
            commands::chapter::hydrate_chapter_token_estimates,
            commands::chapter::delete_chapter,
            commands::chapter::delete_chapters,
            commands::chapter::clear_chapter_analysis,
            commands::chapter::clear_chapter_outline,
            commands::analysis::generate_prompt,
            commands::analysis::estimate_prompt_tokens,
            commands::analysis::estimate_text_tokens,
            commands::analysis::parse_manual_result,
            commands::analysis::save_analysis,
            commands::analysis::analyze_chapter_api,
            commands::batch::batch_analyze_novel,
            commands::batch::batch_analyze_chapters,
            commands::batch::cancel_batch,
            commands::settings::get_llm_config,
            commands::settings::save_llm_config,
            commands::settings::list_models,
            commands::settings::update_novel_dimensions,
            commands::settings::get_all_dimensions,
            commands::summary::get_full_summary_manual_prompt,
            commands::summary::get_novel_summary,
            commands::summary::save_novel_summary,
            commands::summary::clear_novel_summary,
            commands::summary::generate_full_summary,
            commands::summary::parse_manual_summary_result,
            commands::outline::generate_chapter_outline,
            commands::outline::estimate_outline_prompt_tokens,
            commands::outline::get_chapter_outline,
            commands::outline::batch_generate_outlines,
            commands::outline::batch_generate_outline_chapters,
            commands::outline::get_book_outline,
            commands::outline::clear_book_outline,
            commands::outline::generate_book_outline,
            commands::export::export_novel_report,
            commands::export::export_book_outline,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
