use crate::models::LlmConfig;
use crate::token_utils::estimate_tokens;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use tauri::Emitter;

/// List available models from an OpenAI-compatible API.
pub async fn list_models(config: &LlmConfig) -> Result<Vec<String>, String> {
    let mut url = config.base_url.trim_end_matches('/').to_string();
    if !url.ends_with("/models") {
        url = format!("{}/models", url);
    }

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .send()
        .await
        .map_err(|e| format!("请求模型列表失败: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("接口返回错误状态码: {} - {}", status, text));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;
    let mut model_ids = Vec::new();

    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for item in data {
            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                model_ids.push(id.to_string());
            }
        }
    } else {
        return Err("返回的数据格式不正确，缺少 data 数组".to_string());
    }

    model_ids.sort();
    Ok(model_ids)
}

/// Call an OpenAI-compatible API with the given prompt.
pub async fn call_api(config: &LlmConfig, prompt: &str) -> Result<String, String> {
    // Token check
    let prompt_tokens = estimate_tokens(prompt);
    let max_tokens = config.max_context_tokens as usize;
    if prompt_tokens > max_tokens {
        return Err(format!(
            "Prompt 预估 {} tokens，超过模型上限 {} tokens。请减少分析维度或使用更大上下文的模型。",
            prompt_tokens, max_tokens
        ));
    }

    // Configure client
    let openai_config = OpenAIConfig::new()
        .with_api_base(&config.base_url)
        .with_api_key(&config.api_key);

    let client = Client::with_config(openai_config);

    // Build request
    let mut builder = CreateChatCompletionRequestArgs::default();
    builder
        .model(&config.model)
        .temperature(config.temperature)
        .messages(vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage::from(
                    "你是一位专业的文学分析助手。请严格按照用户要求返回 JSON 格式，不要添加任何额外文本。"
                )
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage::from(prompt)
            ),
        ]);

    if let Some(max_output) = config.max_output_tokens {
        builder.max_tokens(max_output);
    } else {
        // Fallback for max_output_tokens
        builder.max_tokens(8192 as u32);
    }

    let request = builder
        .build()
        .map_err(|e| format!("构建请求失败: {}", e))?;

    // Call API
    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| format!("API 调用失败: {}", e))?;

    // Extract content
    let content = response
        .choices
        .first()
        .and_then(|c| c.message.content.as_ref())
        .ok_or_else(|| "API 返回为空".to_string())?
        .clone();

    Ok(content)
}

/// Call API with streaming, emitting partial content via Tauri events.
pub async fn call_api_stream(
    config: &LlmConfig,
    prompt: &str,
    app: &tauri::AppHandle,
    chapter_id: i64,
) -> Result<String, String> {
    let prompt_tokens = estimate_tokens(prompt);
    let max_tokens = config.max_context_tokens as usize;
    if prompt_tokens > max_tokens {
        return Err(format!(
            "Prompt 预估 {} tokens，超过模型上限 {} tokens。",
            prompt_tokens, max_tokens
        ));
    }

    let openai_config = OpenAIConfig::new()
        .with_api_base(&config.base_url)
        .with_api_key(&config.api_key);
    let client = Client::with_config(openai_config);

    let mut builder = CreateChatCompletionRequestArgs::default();
    builder
        .model(&config.model)
        .temperature(config.temperature)
        .messages(vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage::from(
                    "你是一位专业的文学分析助手。请严格按照用户要求返回 JSON 格式，不要添加任何额外文本。"
                )
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage::from(prompt)
            ),
        ]);

    if let Some(max_output) = config.max_output_tokens {
        builder.max_tokens(max_output);
    } else {
        builder.max_tokens(8192u32);
    }

    let request = builder
        .build()
        .map_err(|e| format!("构建请求失败: {}", e))?;

    let mut stream = client
        .chat()
        .create_stream(request)
        .await
        .map_err(|e| format!("API 流式调用失败: {}", e))?;

    let mut full_content = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                for choice in &response.choices {
                    if let Some(ref content) = choice.delta.content {
                        full_content.push_str(content);
                        let _ = app.emit(
                            "analysis_streaming",
                            serde_json::json!({
                                "chapter_id": chapter_id,
                                "chunk": content,
                                "full_content": full_content,
                            }),
                        );
                    }
                }
            }
            Err(e) => {
                return Err(format!("流式响应出错: {}", e));
            }
        }
    }

    if full_content.is_empty() {
        return Err("API 返回为空".to_string());
    }

    Ok(full_content)
}
