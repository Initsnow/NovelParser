use crate::models::*;

/// Estimate token count for a text string.
/// Uses a conservative heuristic: Chinese chars ≈ 1.5 tokens each, ASCII words ≈ 1 token.
pub fn estimate_tokens(text: &str) -> usize {
    let mut tokens: f64 = 0.0;
    for ch in text.chars() {
        if ch.is_ascii() {
            if ch.is_ascii_whitespace() || ch.is_ascii_punctuation() {
                tokens += 0.25;
            } else {
                tokens += 0.3; // ~4 ASCII chars per token on average
            }
        } else {
            tokens += 1.5; // CJK chars are ~1.5 tokens conservatively
        }
    }
    tokens.ceil() as usize
}

/// Calculate available content tokens given LLM config and prompt template overhead.
pub fn calculate_available_tokens(config: &LlmConfig, prompt_template_tokens: usize) -> usize {
    let output_reserve = config.max_output_tokens.unwrap_or(4096) as usize;
    let total = config.max_context_tokens as usize;
    total
        .saturating_sub(prompt_template_tokens)
        .saturating_sub(output_reserve)
}

/// Split chapter content into segments that fit within the token budget.
/// Splits on paragraph boundaries (\n\n) to maintain readability.
pub fn split_content_by_tokens(content: &str, max_tokens: usize) -> Vec<String> {
    if estimate_tokens(content) <= max_tokens {
        return vec![content.to_string()];
    }

    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut segments: Vec<String> = Vec::new();
    let mut current_segment = String::new();
    let mut current_tokens: usize = 0;

    for para in paragraphs {
        let para_tokens = estimate_tokens(para);

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
        if estimate_tokens(&seg) <= max_tokens {
            final_segments.push(seg);
        } else {
            // Hard split by lines
            let lines: Vec<&str> = seg.lines().collect();
            let mut chunk = String::new();
            let mut chunk_tokens: usize = 0;
            for line in lines {
                let line_tokens = estimate_tokens(line);
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
        let tokens = estimate_tokens(text);
        // 10 Chinese chars * 1.5 = 15
        assert!(tokens >= 10 && tokens <= 20);
    }

    #[test]
    fn test_estimate_tokens_english() {
        let text = "This is a test sentence with some words.";
        let tokens = estimate_tokens(text);
        assert!(tokens > 5 && tokens < 20);
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
        let segments = split_content_by_tokens(&content, 100);
        assert!(segments.len() > 1);
    }
}
