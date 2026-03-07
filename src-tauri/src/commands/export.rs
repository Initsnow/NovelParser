use crate::export as export_mod;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_novel_report(
    state: State<'_, AppState>,
    novel_id: String,
    dir_path: String,
) -> Result<(), String> {
    // Collect all data while holding the lock
    let (novel, summary, metas_and_chapters) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
        let summary = db
            .load_novel_summary(&novel_id)
            .map_err(|e| e.to_string())?;
        let metas = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?;
        let chapters_data: Vec<_> = metas
            .into_iter()
            .filter_map(|meta| db.load_chapter(meta.id).ok())
            .filter(|ch| ch.analysis.is_some())
            .collect();
        (novel, summary, chapters_data)
    };

    // Do file I/O outside the lock, in spawn_blocking to avoid blocking async runtime
    tokio::task::spawn_blocking(move || {
        let folder_name = format!("《{}》分析报告", novel.title);
        let target_dir = std::path::Path::new(&dir_path).join(folder_name);

        if !target_dir.exists() {
            std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        }

        if let Some(s) = &summary {
            let global_md = export_mod::generate_global_summary_md(&novel, Some(s));
            let sum_path = target_dir.join("全书分析.md");
            std::fs::write(&sum_path, global_md).map_err(|e| e.to_string())?;
        }

        for ch in metas_and_chapters {
            let md = export_mod::generate_chapter_md(&ch);
            let safe_title = ch
                .title
                .replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'][..], "_");
            let file_name = format!("第{:03}章_{}.md", ch.index + 1, safe_title);
            let ch_path = target_dir.join(file_name);
            std::fs::write(&ch_path, md).map_err(|e| e.to_string())?;
        }

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("导出任务失败: {}", e))?
}
