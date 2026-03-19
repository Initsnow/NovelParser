use crate::models::*;
use crate::outline as outline_mod;
use crate::storage::Database;
use crate::AppState;
use futures::StreamExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, Manager};

async fn do_generate_chapter_outline(
    http_client: &reqwest::Client,
    app: &tauri::AppHandle,
    db_mutex: &Mutex<Database>,
    chapter_id: i64,
) -> Result<ChapterOutline, String> {
    let (chapter, config, content_hash, cached_outline) = {
        let db = db_mutex.lock().map_err(|e| e.to_string())?;
        let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
        let config = db.load_llm_config().map_err(|e| e.to_string())?;
        let content_hash = outline_mod::chapter_content_hash(&chapter.title, &chapter.content);
        let cached_outline = match db
            .load_chapter_outline_hash(chapter_id)
            .map_err(|e| e.to_string())?
        {
            Some(existing_hash) if existing_hash == content_hash => db
                .load_chapter_outline(chapter_id)
                .map_err(|e| e.to_string())?,
            _ => None,
        };
        (chapter, config, content_hash, cached_outline)
    };

    if let Some(outline) = cached_outline {
        let _ = app.emit(
            "outline_progress",
            ProgressEvent {
                novel_id: chapter.novel_id,
                chapter_id: Some(chapter_id),
                status: "cached".to_string(),
                current: 1,
                total: 1,
                message: "命中章节提纲缓存".to_string(),
            },
        );
        return Ok(outline);
    }

    let _ = app.emit(
        "outline_progress",
        ProgressEvent {
            novel_id: chapter.novel_id.clone(),
            chapter_id: Some(chapter_id),
            status: "outlining".to_string(),
            current: 0,
            total: 1,
            message: "正在提取章节提纲...".to_string(),
        },
    );

    let (_, mut outline) =
        outline_mod::generate_chapter_outline(http_client, app, &chapter, &config, chapter_id).await?;
    outline.created_at = chrono::Utc::now().to_rfc3339();

    {
        let db = db_mutex.lock().map_err(|e| e.to_string())?;
        db.save_chapter_outline(
            chapter_id,
            &chapter.novel_id,
            chapter.index,
            &content_hash,
            &outline,
        )
        .map_err(|e| e.to_string())?;
        db.clear_book_outline(&chapter.novel_id)
            .map_err(|e| e.to_string())?;
    }

    let _ = app.emit(
        "outline_progress",
        ProgressEvent {
            novel_id: chapter.novel_id,
            chapter_id: Some(chapter_id),
            status: "done".to_string(),
            current: 1,
            total: 1,
            message: "章节提纲已生成".to_string(),
        },
    );

    Ok(outline)
}

fn make_group_prompt_items(group: &[outline_mod::OutlineNode]) -> Vec<(usize, usize, String)> {
    group.iter()
        .map(|node| (node.chapter_start, node.chapter_end, node.content.clone()))
        .collect()
}

fn promote_group_to_node(
    novel_id: &str,
    layer: i32,
    group_index: i32,
    group: &[outline_mod::OutlineNode],
    outline: &BookOutline,
) -> (OutlineCacheEntry, outline_mod::OutlineNode) {
    let chapter_start = group.first().map(|n| n.chapter_start).unwrap_or_default();
    let chapter_end = group.last().map(|n| n.chapter_end).unwrap_or_default();
    let content_hash = outline_mod::combined_hash(
        &group
            .iter()
            .map(|node| node.content_hash.clone())
            .collect::<Vec<_>>(),
    );
    let entry = OutlineCacheEntry {
        layer,
        group_index,
        chapter_start,
        chapter_end,
        content_hash: content_hash.clone(),
        outline: outline.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let node = outline_mod::OutlineNode::from_book_outline(
        chapter_start,
        chapter_end,
        content_hash.clone(),
        outline,
    );

    let _ = novel_id;
    (entry, node)
}

#[tauri::command]
pub async fn generate_chapter_outline(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    chapter_id: i64,
) -> Result<ChapterOutline, String> {
    do_generate_chapter_outline(&state.http_client, &app, &state.db, chapter_id).await
}

#[tauri::command]
pub fn estimate_outline_prompt_tokens(
    state: State<'_, AppState>,
    chapter_id: i64,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
    let config = db.load_llm_config().unwrap_or_default();
    let prompt_text =
        crate::prompt::generate_chapter_outline_prompt(&chapter.title, &chapter.content);
    Ok(crate::token_utils::estimate_tokens_for_model(
        &prompt_text,
        &config.model,
    ))
}

#[tauri::command]
pub fn get_chapter_outline(
    state: State<'_, AppState>,
    chapter_id: i64,
) -> Result<Option<ChapterOutline>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_chapter_outline(chapter_id).map_err(|e| e.to_string())
}

async fn run_batch_outline_generation(
    http_client: reqwest::Client,
    app: tauri::AppHandle,
    state: &State<'_, AppState>,
    novel_id: String,
    metas: Vec<ChapterMeta>,
    concurrency: usize,
) -> Result<(), String> {
    let total = metas.len();
    if total == 0 {
        return Ok(());
    }

    state.batch_cancel.store(false, Ordering::Relaxed);
    let completed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));

    let app_for_tasks = app.clone();
    let novel_id_for_tasks = novel_id.clone();
    let completed_for_tasks = completed.clone();
    let failed_for_tasks = failed.clone();

    let mut tasks = futures::stream::iter(metas.into_iter().map(move |meta| {
        let http_client = http_client.clone();
        let app = app_for_tasks.clone();
        let novel_id = novel_id_for_tasks.clone();
        let db = &state.db;
        let cancel_flag = &state.batch_cancel;
        let completed = completed_for_tasks.clone();
        let failed = failed_for_tasks.clone();

        async move {
            if cancel_flag.load(Ordering::Relaxed) {
                return Ok::<bool, String>(true);
            }

            let current = completed.load(Ordering::Relaxed);
            let _ = app.emit(
                "outline_batch_progress",
                ProgressEvent {
                    novel_id: novel_id.clone(),
                    chapter_id: Some(meta.id),
                    status: "batch_outlining".to_string(),
                    current,
                    total,
                    message: format!("正在处理: {} ({}/{})", meta.title, current, total),
                },
            );

            match do_generate_chapter_outline(&http_client, &app, db, meta.id).await {
                Ok(_) => {
                    let next = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    let _ = app.emit(
                        "outline_batch_progress",
                        ProgressEvent {
                            novel_id,
                            chapter_id: Some(meta.id),
                            status: "chapter_done".to_string(),
                            current: next,
                            total,
                            message: format!("已完成: {} ({}/{})", meta.title, next, total),
                        },
                    );
                }
                Err(err) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    let current = completed.load(Ordering::Relaxed);
                    let _ = app.emit(
                        "outline_batch_progress",
                        ProgressEvent {
                            novel_id,
                            chapter_id: Some(meta.id),
                            status: "error".to_string(),
                            current,
                            total,
                            message: format!("提纲生成失败 {}: {}", meta.title, err),
                        },
                    );
                }
            }

            Ok(false)
        }
    }))
    .buffer_unordered(concurrency);

    while let Some(result) = tasks.next().await {
        if let Ok(true) = result {
            state.batch_cancel.store(false, Ordering::Relaxed);
            let current = completed.load(Ordering::Relaxed);
            let _ = app.emit(
                "outline_batch_progress",
                ProgressEvent {
                    novel_id: novel_id.clone(),
                    chapter_id: None,
                    status: "batch_cancelled".to_string(),
                    current,
                    total,
                    message: format!("提纲批处理已取消 ({}/{})", current, total),
                },
            );
            return Ok(());
        }
    }

    let success_count = completed.load(Ordering::Relaxed);
    let fail_count = failed.load(Ordering::Relaxed);
    let done_message = if fail_count > 0 {
        format!("章节提纲生成完成：成功 {}，失败 {}", success_count, fail_count)
    } else {
        "章节提纲生成完成".to_string()
    };

    let _ = app.emit(
        "outline_batch_progress",
        ProgressEvent {
            novel_id,
            chapter_id: None,
            status: "batch_done".to_string(),
            current: total,
            total,
            message: done_message,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn batch_generate_outlines(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<(), String> {
    let (metas, concurrency) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let metas = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?;
        let config = db.load_llm_config().map_err(|e| e.to_string())?;
        (metas, config.max_concurrent_tasks.max(1) as usize)
    };

    run_batch_outline_generation(state.http_client.clone(), app, &state, novel_id, metas, concurrency).await
}

#[tauri::command]
pub async fn batch_generate_outline_chapters(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
    chapter_ids: Vec<i64>,
) -> Result<(), String> {
    let (metas, concurrency) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let all = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?;
        let metas = all
            .into_iter()
            .filter(|meta| chapter_ids.contains(&meta.id))
            .collect::<Vec<_>>();
        let config = db.load_llm_config().map_err(|e| e.to_string())?;
        (metas, config.max_concurrent_tasks.max(1) as usize)
    };

    run_batch_outline_generation(state.http_client.clone(), app, &state, novel_id, metas, concurrency).await
}

#[tauri::command]
pub fn get_book_outline(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<Option<BookOutline>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_book_outline(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_book_outline(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_book_outline(&novel_id).map_err(|e| e.to_string())?;
    db.clear_outline_cache(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_book_outline(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<BookOutline, String> {
    let chapter_outlines = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.list_chapter_outlines(&novel_id).map_err(|e| e.to_string())?
    };

    if chapter_outlines.is_empty() {
        return Err("当前没有已生成的章节提纲".to_string());
    }

    let root_hash = outline_mod::combined_hash(
        &chapter_outlines
            .iter()
            .map(|(_, _, _, hash, _)| hash.clone())
            .collect::<Vec<_>>(),
    );

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(existing_hash) = db
            .load_book_outline_hash(&novel_id)
            .map_err(|e| e.to_string())?
        {
            if existing_hash == root_hash {
                if let Some(outline) = db.load_book_outline(&novel_id).map_err(|e| e.to_string())? {
                    return Ok(outline);
                }
            }
        }
    }

    let mut nodes = chapter_outlines
        .into_iter()
        .map(|(_, index, _, hash, outline)| outline_mod::OutlineNode::from_chapter(index, &outline, hash))
        .collect::<Vec<_>>();

    let (max_context_tokens, max_concurrent_tasks, summary_max_tokens) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let config = db.load_llm_config().map_err(|e| e.to_string())?;
        (config.max_context_tokens, config.max_concurrent_tasks, config.summary_max_tokens)
    };

    let total_initial_tokens: usize = nodes.iter().map(|n| n.token_estimate).sum();
    // 如果总 Token 数小于模型上下文的 80%，直接进行单次归并（Direct Mode）
    let context_limit = (max_context_tokens as f64 * 0.8) as usize;
    
    let is_direct_mode = total_initial_tokens < context_limit && nodes.len() > 1;

    let _ = app.emit(
        "outline_progress",
        ProgressEvent {
            novel_id: novel_id.clone(),
            chapter_id: None,
            status: "summarizing".to_string(),
            current: 0,
            total: 1,
            message: if is_direct_mode { "模型上下文足够，正在进行直接归并..." } else { "正在准备分层归并章节提纲..." }.to_string(),
        },
    );

    let mut layer = 1i32;
    let mut final_outline: Option<BookOutline> = None;
    let concurrency = max_concurrent_tasks.max(1) as usize;

    while final_outline.is_none() {
        let (groups, _target_tokens) = if is_direct_mode || nodes.len() <= 1 {
            (vec![nodes.clone()], 0)
        } else {
            let target = if layer == 1 { 6000 } else { 8000 };
            (outline_mod::make_outline_groups(&nodes, target), target)
        };

        let total_groups = groups.len();
        let mut next_nodes = Vec::with_capacity(total_groups);

        {
            let current_layer = layer;
            let novel_id_clone = novel_id.clone();
            let app_clone = app.clone();
            let http_client = state.http_client.clone();
            
            let mut stream = futures::stream::iter(groups.into_iter().enumerate().map(move |(group_index, group)| {
                let app = app_clone.clone();
                let novel_id = novel_id_clone.clone();
                let http_client = http_client.clone();
                let layer = current_layer;
                let total_groups = total_groups;
                let sm_max_tokens = summary_max_tokens;

                async move {
                    let _ = app.emit(
                        "outline_progress",
                        ProgressEvent {
                            novel_id: novel_id.clone(),
                            chapter_id: None,
                            status: "summarizing".to_string(),
                            current: group_index + 1,
                            total: total_groups,
                            message: format!("正在归并第 {} 层 ({}/{})", layer, group_index + 1, total_groups),
                        },
                    );

                    let group_hash = outline_mod::combined_hash(
                        &group
                            .iter()
                            .map(|node| node.content_hash.clone())
                            .collect::<Vec<_>>(),
                    );

                    // 首先获取配置和检查缓存，不持有锁跨 await
                    let (config, cached) = {
                        let st = app.state::<AppState>();
                        let db = st.db.lock().map_err(|e| e.to_string())?;
                        let config = db.load_llm_config().map_err(|e| e.to_string())?;
                        let cached = db.load_outline_cache(&novel_id, layer, group_index as i32)
                            .map_err(|e| e.to_string())?;
                        (config, cached)
                    };

                    let group_outline = if let Some(cache) = cached.filter(|c| c.content_hash == group_hash) {
                        cache.outline
                    } else {
                        let prompt_text = crate::prompt::generate_outline_group_prompt(&make_group_prompt_items(&group), layer as usize);
                        let response = crate::llm::call_api(&http_client, &config, &prompt_text, sm_max_tokens).await?;
                        let mut outline = outline_mod::parse_book_outline_json(&response)?;
                        outline.created_at = chrono::Utc::now().to_rfc3339();
                        
                        // 更新缓存
                        let (entry, _) = promote_group_to_node(&novel_id, layer, group_index as i32, &group, &outline);
                        let st = app.state::<AppState>();
                        let db = st.db.lock().map_err(|e| e.to_string())?;
                        db.save_outline_cache(&novel_id, &entry).map_err(|e| e.to_string())?;
                        
                        outline
                    };

                    let (_, updated_next_node) = promote_group_to_node(&novel_id, layer, group_index as i32, &group, &group_outline);

                    Ok::<(outline_mod::OutlineNode, BookOutline), String>((updated_next_node, group_outline))
                }
            })).buffered(concurrency);
            
            while let Some(res) = stream.next().await {
                let (next_node, _) = res?;
                next_nodes.push(next_node);
            }
        } 

        if next_nodes.len() == 1 {
            let mut outline = outline_mod::parse_book_outline_json(&next_nodes[0].content)?;
            outline.created_at = chrono::Utc::now().to_rfc3339();
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.save_book_outline(&novel_id, &root_hash, &outline)
                .map_err(|e| e.to_string())?;
            final_outline = Some(outline);
        } else {
            if next_nodes.len() >= nodes.len() {
                 return Err(format!("全书提纲归并停滞：第 {} 层归并后节点数（{}）未能减少。这通常是因为内容过长，请尝试在设置中增加上下文限制或更换更强大的模型。", layer, next_nodes.len()));
            }
            nodes = next_nodes;
            layer += 1;
        }
    }

    let outline = final_outline.expect("book outline must exist");
    let _ = app.emit(
        "outline_progress",
        ProgressEvent {
            novel_id,
            chapter_id: None,
            status: "done".to_string(),
            current: 1,
            total: 1,
            message: "全书提纲已生成".to_string(),
        },
    );

    Ok(outline)
}
