use crate::models::*;
use tiktoken_rs::{o200k_base_singleton, CoreBPE};

const DEFAULT_TOKENIZER_MODEL: &str = "gpt-4o";

fn encoder_for_model(_model: &str) -> std::borrow::Cow<'static, CoreBPE> {
    std::borrow::Cow::Borrowed(o200k_base_singleton())
}

fn chat_overhead_for_model(model: &str) -> (usize, isize, usize) {
    let lower = model.to_ascii_lowercase();
    if lower.starts_with("gpt-3.5") || lower.contains("gpt-3.5") {
        (4, -1, 3)
    } else {
        (3, 1, 3)
    }
}

/// Estimate token count for a text string.
pub fn estimate_tokens(text: &str) -> usize {
    estimate_tokens_for_model(text, DEFAULT_TOKENIZER_MODEL)
}

pub fn estimate_tokens_for_model(text: &str, model: &str) -> usize {
    encoder_for_model(model)
        .encode_with_special_tokens(text)
        .len()
}

pub fn estimate_chat_tokens_for_model(model: &str, system_prompt: &str, user_prompt: &str) -> usize {
    let bpe = encoder_for_model(model);
    let (tokens_per_message, tokens_per_name, reply_primer_tokens) = chat_overhead_for_model(model);

    let mut total = reply_primer_tokens;
    for (role, content) in [("system", system_prompt), ("user", user_prompt)] {
        total += tokens_per_message;
        total += bpe.encode_with_special_tokens(role).len();
        total += bpe.encode_with_special_tokens(content).len();
        if tokens_per_name > 0 {
            total += tokens_per_name as usize;
        }
    }

    total
}

pub fn calculate_available_request_tokens(config: &LlmConfig, max_output: Option<u32>) -> usize {
    let output_reserve = max_output
        .or(config.chapter_max_tokens)
        .unwrap_or(8192) as usize;
    (config.max_context_tokens as usize).saturating_sub(output_reserve)
}
/// Split text content into segments that fit within a token limit.
pub fn split_content_by_tokens(content: &str, max_tokens: usize) -> Vec<String> {
    split_content_by_tokens_for_model(content, max_tokens, DEFAULT_TOKENIZER_MODEL)
}

pub fn split_content_by_tokens_for_model(
    content: &str,
    max_tokens: usize,
    model: &str,
) -> Vec<String> {
    if estimate_tokens_for_model(content, model) <= max_tokens {
        return vec![content.to_string()];
    }

    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut segments: Vec<String> = Vec::new();
    let mut current_segment = String::new();
    let mut current_tokens: usize = 0;

    for para in paragraphs {
        let para_tokens = estimate_tokens_for_model(para, model);

        if current_tokens + para_tokens > max_tokens && !current_segment.is_empty() {
            segments.push(current_segment.trim().to_string());
            current_segment = String::new();
            current_tokens = 0;
        }

        if !current_segment.is_empty() {
            current_segment.push_str("\n\n");
        }
        current_segment.push_str(para);
        current_tokens += para_tokens;
    }

    if !current_segment.trim().is_empty() {
        segments.push(current_segment.trim().to_string());
    }

    // If we still have segments that are too long, do a hard split by lines
    let mut final_segments: Vec<String> = Vec::new();
    for seg in segments {
        if estimate_tokens_for_model(&seg, model) <= max_tokens {
            final_segments.push(seg);
        } else {
            // Hard split by lines
            let lines: Vec<&str> = seg.lines().collect();
            let mut chunk = String::new();
            let mut chunk_tokens: usize = 0;
            for line in lines {
                let line_tokens = estimate_tokens_for_model(line, model);
                if chunk_tokens + line_tokens > max_tokens && !chunk.is_empty() {
                    final_segments.push(chunk.trim().to_string());
                    chunk = String::new();
                    chunk_tokens = 0;
                }
                chunk.push_str(line);
                chunk.push('\n');
                chunk_tokens += line_tokens;
            }
            if !chunk.trim().is_empty() {
                final_segments.push(chunk.trim().to_string());
            }
        }
    }

    final_segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens_chinese() {
        let text = "这是一段中文测试文本";
        let tokens = estimate_tokens_for_model(text, "gpt-4o");
        assert!(tokens > 0);
    }

    #[test]
    fn test_estimate_tokens_english() {
        let text = "This is a test sentence with some words.";
        let tokens = estimate_tokens_for_model(text, "gpt-4o");
        assert!(tokens > 0);
    }

    #[test]
    fn test_estimate_chat_tokens() {
        let tokens = estimate_chat_tokens_for_model("gpt-4o", "system", "hello world");
        assert!(tokens > 0);
    }

    #[test]
    fn test_split_short_content() {
        let content = "Short text";
        let segments = split_content_by_tokens(content, 1000);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn test_split_long_content() {
        let content = (0..100)
            .map(|i| format!("这是第{}段很长的文本内容，用来测试分段功能。", i))
            .collect::<Vec<_>>()
            .join("\n\n");
        let segments = split_content_by_tokens_for_model(&content, 100, "gpt-4o");
        assert!(segments.len() > 1);
    }
    #[test]
    fn test_performance_many_calls() {
        let text = "这是一段用来测试性能的中文文本，我们需要确保多次调用 token 估算不会耗时太久。";
        // 预热：第一次调用会触发单例初始化（解析模型数据）
        let _ = estimate_tokens(text);
        
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = estimate_tokens(text);
        }
        let duration = start.elapsed();
        // 优化后，1000 次调用在 debug 模式下也应该在几百毫秒内完成。
        // 如果没有优化（每次都 re-parse），1000 次调用可能需要 100 秒以上。
        assert!(duration.as_millis() < 1000, "Performance test failed: 1000 calls took too long ({:?})", duration);
    }
}
