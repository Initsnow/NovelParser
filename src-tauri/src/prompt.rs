use crate::models::*;

/// Generate a chapter analysis prompt based on selected dimensions.
pub fn generate_chapter_prompt(
    title: &str,
    content: &str,
    dimensions: &[AnalysisDimension],
) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是一位资深的文学评论家和小说研究者，拥有敏锐的文本洞察力。\n");
    prompt.push_str("请仔细阅读以下小说章节，进行深入、有见地的文学分析。\n");
    prompt.push_str("分析应当基于文本证据，避免泛泛而谈。每个维度都有一个 insights 字段，请在其中写出你最深刻的洞察。\n");
    prompt.push_str("请返回 JSON 格式。\n\n");

    prompt.push_str(&format!("## 章节：{}\n\n", title));
    prompt.push_str(content);
    prompt.push_str("\n\n");

    prompt.push_str("## 分析维度\n\n");
    for dim in dimensions {
        prompt.push_str(&format!("### {}\n", dim.display_name()));
        prompt.push_str(dimension_instruction(dim));
        prompt.push_str("\n\n");
    }

    prompt.push_str("## 输出 JSON 结构\n\n");
    prompt.push_str(&generate_json_schema(dimensions));

    prompt
}

/// Generate a prompt for a chapter segment (when chapter is split due to length).
pub fn generate_segment_prompt(
    title: &str,
    segment_content: &str,
    segment_index: usize,
    total_segments: usize,
    dimensions: &[AnalysisDimension],
) -> String {
    let mut prompt = String::new();

    prompt.push_str(
        "你是一位资深的文学评论家。请分析以下小说章节片段，注意这只是完整章节的一部分。\n",
    );
    prompt.push_str("分析应基于文本证据。请返回 JSON 格式。\n\n");
    prompt.push_str(&format!(
        "## 章节：{} (第 {} 段，共 {} 段)\n\n",
        title,
        segment_index + 1,
        total_segments
    ));
    prompt.push_str(segment_content);
    prompt.push_str("\n\n");

    prompt.push_str("## 分析维度\n\n");
    for dim in dimensions {
        prompt.push_str(&format!("### {}\n", dim.display_name()));
        prompt.push_str(dimension_instruction(dim));
        prompt.push_str("\n\n");
    }

    prompt.push_str("## 输出 JSON 结构\n\n");
    prompt.push_str(&generate_json_schema(dimensions));

    prompt
}

/// Generate a group summary prompt for tree-reduction.
pub fn generate_group_summary_prompt(
    chapter_summaries: &[(usize, String)],
    dimensions: &[AnalysisDimension],
) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是一位资深文学评论家。请阅读以下若干章节的分析结果，\n");
    prompt.push_str("将它们整合为一份连贯的阶段性分析报告。注意发现跨章节的演变规律和深层脉络。\n");
    prompt.push_str("请返回 JSON 格式。\n\n");

    prompt.push_str("## 各章分析\n\n");
    for (idx, summary) in chapter_summaries {
        prompt.push_str(&format!("### 第 {} 章\n{}\n\n", idx + 1, summary));
    }

    prompt.push_str("## 输出 JSON 结构\n\n");
    prompt.push_str(&generate_summary_json_schema(dimensions));

    prompt
}

/// Generate the final summary prompt from group summaries.
pub fn generate_final_summary_prompt(
    group_summaries: &[String],
    dimensions: &[AnalysisDimension],
) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是一位资深文学评论家。以下是一部小说各部分的汇总分析。\n");
    prompt.push_str(
        "请将它们合并为最终的全书深度分析报告，揭示贯穿全书的主线、发展脉络和艺术特色。\n",
    );
    prompt.push_str("请返回 JSON 格式。\n\n");

    for (i, summary) in group_summaries.iter().enumerate() {
        prompt.push_str(&format!("## 第 {} 部分汇总\n{}\n\n", i + 1, summary));
    }

    prompt.push_str("## 输出 JSON 结构\n\n");
    prompt.push_str(&generate_summary_json_schema(dimensions));

    prompt
}

/// Generate a massive manual prompt for full book summaries (if user wants to paste all chapters manually)
pub fn generate_manual_full_summary_prompt(
    chapters: &[(usize, String)],
    dimensions: &[AnalysisDimension],
) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是一位资深文学评论家。请阅读以下【所有已分析章节】的汇总数据。\n");
    prompt.push_str("你需要根据这些片段，提炼出一份贯穿整部小说的终极概览。\n");
    prompt.push_str("请严格返回 JSON 格式结果，不要包含其他说明文字。\n\n");

    for (idx, summary) in chapters {
        prompt.push_str(&format!("## 第 {} 章\n{}\n\n", idx + 1, summary));
    }

    prompt.push_str("## 输出 JSON 结构\n\n");
    prompt.push_str(&generate_summary_json_schema(dimensions));

    prompt
}

fn dimension_instruction(dim: &AnalysisDimension) -> &'static str {
    match dim {
        AnalysisDimension::Characters => {
            "梳理本章出场的所有人物。对每个人物，判断其在故事中的分量（主角/配角/龙套），\
             概括其在本章中展现出的性格面貌和关键行为。\
             重点分析人物之间的关系网络——不仅标注关系类型，还要关注本章中关系是否发生了微妙的变化或转折。"
        }
        AnalysisDimension::Plot => {
            "用自己的话概括本章的故事走向。梳理关键事件的因果链条——每个重要事件是被什么驱动的，\
             又引发了什么后果。点明本章的核心冲突是什么，以及作者在章末留下了哪些悬念。"
        }
        AnalysisDimension::Foreshadowing => {
            "寻找作者在本章埋下的伏笔和暗示——那些看似不经意但可能在后文有重要作用的细节。\
             如果本章某些情节呼应了前面章节的铺垫，也请指出。\
             标注本章的剧情转折点，以及章末是否留有引人继续阅读的钩子。"
        }
        AnalysisDimension::WritingTechnique => {
            "分析作者的叙事策略：使用的是第几人称？全知还是限知视角？\
             时间线是否有变化（倒叙、插叙、闪回）？\
             注意叙事节奏的把控——哪些地方是精细的场景描写，哪些地方是跳跃式的概述，这种节奏变化产生了什么效果？"
        }
        AnalysisDimension::Rhetoric => {
            "发掘本章中出彩的修辞手法——比喻是否新颖，拟人是否传神，排比是否有力？\
             请附上原文中最有代表性的例句。评价整体的语言风格特征，并摘录最多3句让你印象深刻的佳句。"
        }
        AnalysisDimension::Emotion => {
            "感受本章的情感纹理。整体基调是什么？\
             随着情节推进，情感是如何流动和转变的？\
             用段落或场景为单位标注情感变化，并分析作者是用了什么手法来营造这种氛围的。"
        }
        AnalysisDimension::Themes => {
            "提炼本章触及的深层主题——爱情、权力、孤独、成长、死亡、自由……\
             作者通过情节和人物传达了什么样的价值立场？是否涉及社会批判或哲学思考？"
        }
        AnalysisDimension::Worldbuilding => {
            "记录本章中新出现或进一步展开的世界设定：地点、组织、势力、社会规则、超自然法则、重要物品等。\
             注意权力结构和社会关系方面的信息。"
        }
    }
}

fn generate_json_schema(dimensions: &[AnalysisDimension]) -> String {
    let mut parts: Vec<String> = Vec::new();

    for dim in dimensions {
        let schema = match dim {
            AnalysisDimension::Characters => {
                r#""characters": {
    "characters": [{"name": "姓名", "role": "主角/配角/龙套", "traits": ["特征1"], "actions": "行为描述"}],
    "relationships": [{"from": "人名A", "to": "人名B", "relation_type": "类型", "description": "描述", "change": "变化或null"}],
    "insights": "对本章人物塑造的整体评价和深层解读，可以自由发挥"
  }"#
            }
            AnalysisDimension::Plot => {
                r#""plot": {
    "summary": "剧情摘要",
    "key_events": [{"event": "事件描述", "cause": "原因或null", "effect": "影响或null"}],
    "conflicts": ["冲突1"],
    "suspense": ["悬念1"],
    "insights": "对本章叙事策略、情节编排的深层解读"
  }"#
            }
            AnalysisDimension::Foreshadowing => {
                r#""foreshadowing": {
    "setups": [{"content": "伏笔内容", "chapter_ref": null}],
    "callbacks": [{"content": "呼应内容", "chapter_ref": "第X章"}],
    "turning_points": ["转折点1"],
    "cliffhangers": ["悬念1"],
    "insights": "对作者伏笔技巧和叙事张力的评价"
  }"#
            }
            AnalysisDimension::WritingTechnique => {
                r#""writing_technique": {
    "narrative_perspective": "叙事视角",
    "time_sequence": "时序处理",
    "pacing": "节奏描述",
    "structural_notes": "结构特点",
    "insights": "对写作技法的整体评价，独到之处或不足"
  }"#
            }
            AnalysisDimension::Rhetoric => {
                r#""rhetoric": {
    "devices": [{"name": "手法名", "example": "原文例句"}],
    "language_style": "语言风格描述",
    "notable_quotes": ["佳句1"],
    "insights": "对本章语言艺术的整体鉴赏"
  }"#
            }
            AnalysisDimension::Emotion => {
                r#""emotion": {
    "overall_tone": "整体基调",
    "emotion_arc": [{"segment": "段落/场景", "emotion": "情绪类型", "intensity": "高/中/低"}],
    "atmosphere_techniques": ["手法1"],
    "insights": "对情感表达的深入解读"
  }"#
            }
            AnalysisDimension::Themes => {
                r#""themes": {
    "motifs": ["母题1"],
    "values": ["价值观1"],
    "social_commentary": "社会议题或null",
    "insights": "对主题深度和思想内涵的评论"
  }"#
            }
            AnalysisDimension::Worldbuilding => {
                r#""worldbuilding": {
    "locations": [{"name": "地名", "description": "描述"}],
    "organizations": [{"name": "组织名", "description": "描述"}],
    "power_systems": ["力量体系"],
    "items": [{"name": "物品名", "description": "描述"}],
    "rules": ["规则1"],
    "insights": "对世界观构建的整体评价"
  }"#
            }
        };
        parts.push(format!("  {}", schema));
    }

    format!("{{\n{}\n}}", parts.join(",\n"))
}

fn generate_summary_json_schema(dimensions: &[AnalysisDimension]) -> String {
    let mut parts: Vec<&str> = Vec::new();

    for dim in dimensions {
        match dim {
            AnalysisDimension::Characters => {
                parts.push(r#"  "character_arcs": [{"name": "角色名", "arc": "人物弧线描述"}]"#);
            }
            AnalysisDimension::Plot => {
                parts.push(r#"  "overall_plot": "全书剧情概述""#);
            }
            AnalysisDimension::Themes | AnalysisDimension::Foreshadowing => {
                parts.push(r#"  "themes": ["主题1", "主题2"]"#);
            }
            AnalysisDimension::WritingTechnique | AnalysisDimension::Rhetoric => {
                parts.push(r#"  "writing_style": "写作风格总评""#);
            }
            AnalysisDimension::Worldbuilding => {
                parts.push(r#"  "worldbuilding": "世界观总结""#);
            }
            _ => {}
        }
    }

    // Deduplicate
    parts.sort();
    parts.dedup();

    format!("{{\n{}\n}}", parts.join(",\n"))
}
