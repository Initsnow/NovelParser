use crate::models::*;
use crate::{llm, prompt, token_utils};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn clean_outline_json(raw: &str) -> String {
    crate::analysis::clean_json_response(raw)
}

pub fn parse_chapter_outline_json(json_str: &str) -> Result<ChapterOutline, String> {
    let cleaned = clean_outline_json(json_str);
    serde_json::from_str::<ChapterOutline>(&cleaned).map_err(|e| {
        format!(
            "提纲 JSON 解析失败: {}。原始文本前200字: {}",
            e,
            &cleaned[..cleaned.len().min(200)]
        )
    })
}

pub fn parse_book_outline_json(json_str: &str) -> Result<BookOutline, String> {
    let cleaned = clean_outline_json(json_str);
    serde_json::from_str::<BookOutline>(&cleaned).map_err(|e| {
        format!(
            "全书提纲 JSON 解析失败: {}。原始文本前200字: {}",
            e,
            &cleaned[..cleaned.len().min(200)]
        )
    })
}

pub fn hash_text(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn chapter_content_hash(title: &str, content: &str) -> String {
    hash_text(&format!("{title}\n{content}"))
}

pub fn combined_hash(parts: &[String]) -> String {
    hash_text(&parts.join("|"))
}

#[derive(Clone)]
pub struct OutlineNode {
    pub chapter_start: usize,
    pub chapter_end: usize,
    pub token_estimate: usize,
    pub content_hash: String,
    pub content: String,
}

impl OutlineNode {
    pub fn from_chapter(index: usize, outline: &ChapterOutline, content_hash: String) -> Self {
        let content = serde_json::to_string(outline).unwrap_or_default();
        Self {
            chapter_start: index,
            chapter_end: index,
            token_estimate: token_utils::estimate_tokens(&content),
            content_hash,
            content,
        }
    }

    pub fn from_book_outline(
        chapter_start: usize,
        chapter_end: usize,
        content_hash: String,
        outline: &BookOutline,
    ) -> Self {
        let content = serde_json::to_string(outline).unwrap_or_default();
        Self {
            chapter_start,
            chapter_end,
            token_estimate: token_utils::estimate_tokens(&content),
            content_hash,
            content,
        }
    }
}

pub fn make_outline_groups(nodes: &[OutlineNode], target_tokens: usize) -> Vec<Vec<OutlineNode>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    let mut total = 0usize;

    for node in nodes {
        if !current.is_empty() && total + node.token_estimate > target_tokens {
            groups.push(current);
            current = Vec::new();
            total = 0;
        }

        total += node.token_estimate.max(1);
        current.push(node.clone());
    }

    if !current.is_empty() {
        groups.push(current);
    }

    groups
}

pub async fn generate_chapter_outline(
    app: &tauri::AppHandle,
    chapter: &Chapter,
    config: &LlmConfig,
    chapter_id: i64,
) -> Result<(String, ChapterOutline), String> {
    let content_hash = chapter_content_hash(&chapter.title, &chapter.content);
    let prompt_text = prompt::generate_chapter_outline_prompt(&chapter.title, &chapter.content);
    let response = llm::call_api_stream(
        config,
        &prompt_text,
        app,
        "outline_streaming",
        chapter_id,
        config.chapter_max_tokens,
    )
    .await?;
    let mut outline = parse_chapter_outline_json(&response)?;
    outline.created_at = chrono::Utc::now().to_rfc3339();

    Ok((content_hash, outline))
}
