use serde::{Deserialize, Serialize};

// ---- EPUB Preview ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubPreview {
    pub title: String,
    pub path: String,
    pub chapters: Vec<crate::epub_parser::EpubPreviewChapter>,
}

// ---- Core Structures ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Novel {
    pub id: String,
    pub title: String,
    pub source_type: SourceType,
    pub enabled_dimensions: Vec<AnalysisDimension>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelMeta {
    pub id: String,
    pub title: String,
    pub chapter_count: usize,
    pub analyzed_count: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Epub(String),
    TxtFiles(Vec<String>),
    SingleTxt(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: Option<i64>,
    pub novel_id: String,
    pub index: usize,
    pub title: String,
    pub content: String,
    pub analysis: Option<ChapterAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterMeta {
    pub id: i64,
    pub index: usize,
    pub title: String,
    pub has_analysis: bool,
    pub token_estimate: usize,
}

// ---- Events ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub novel_id: String,
    pub chapter_id: Option<i64>,
    pub status: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

// ---- Analysis Dimensions ----

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisDimension {
    Characters,
    Plot,
    Foreshadowing,
    WritingTechnique,
    Rhetoric,
    Emotion,
    Themes,
    Worldbuilding,
}

impl AnalysisDimension {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Characters,
            Self::Plot,
            Self::Foreshadowing,
            Self::WritingTechnique,
            Self::Rhetoric,
            Self::Emotion,
            Self::Themes,
            Self::Worldbuilding,
        ]
    }

    pub fn default_set() -> Vec<Self> {
        vec![
            Self::Characters,
            Self::Plot,
            Self::Foreshadowing,
            Self::WritingTechnique,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Characters => "人物图谱",
            Self::Plot => "剧情脉络",
            Self::Foreshadowing => "伏笔与转折",
            Self::WritingTechnique => "写作技法",
            Self::Rhetoric => "修辞与语言",
            Self::Emotion => "情感与氛围",
            Self::Themes => "主题与思想",
            Self::Worldbuilding => "世界观设定",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Characters => "👤",
            Self::Plot => "📖",
            Self::Foreshadowing => "🔮",
            Self::WritingTechnique => "✍️",
            Self::Rhetoric => "🎨",
            Self::Emotion => "💠",
            Self::Themes => "🏛️",
            Self::Worldbuilding => "🌍",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Characters => "出场人物、性格特征、人物间关系及变化",
            Self::Plot => "本章摘要、关键事件序列、因果链、冲突与悬念",
            Self::Foreshadowing => "伏笔铺设与呼应、剧情转折点、悬念设置/解除",
            Self::WritingTechnique => "叙事视角、时序处理、节奏控制、结构特点",
            Self::Rhetoric => "修辞手法及例句、语言风格、经典佳句摘录",
            Self::Emotion => "情感基调、情感变化曲线、氛围营造手法",
            Self::Themes => "涉及的主题/母题、价值观表达、社会/哲学议题",
            Self::Worldbuilding => "地点/组织/势力/规则/物品、权力体系、社会结构",
        }
    }
}

// ---- Chapter Analysis (all fields Optional) ----

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChapterAnalysis {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characters: Option<CharactersAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plot: Option<PlotAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreshadowing: Option<ForeshadowingAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writing_technique: Option<WritingTechniqueAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rhetoric: Option<RhetoricAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion: Option<EmotionAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub themes: Option<ThemesAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worldbuilding: Option<WorldbuildingAnalysis>,
}

// ---- Dimension-specific structures ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharactersAnalysis {
    pub characters: Vec<Character>,
    pub relationships: Vec<Relationship>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub actions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub relation_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotAnalysis {
    pub summary: String,
    #[serde(default)]
    pub key_events: Vec<KeyEvent>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub suspense: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeshadowingAnalysis {
    #[serde(default)]
    pub setups: Vec<ForeshadowItem>,
    #[serde(default)]
    pub callbacks: Vec<ForeshadowItem>,
    #[serde(default)]
    pub turning_points: Vec<String>,
    #[serde(default)]
    pub cliffhangers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeshadowItem {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingTechniqueAnalysis {
    pub narrative_perspective: String,
    #[serde(default)]
    pub time_sequence: String,
    #[serde(default)]
    pub pacing: String,
    #[serde(default)]
    pub structural_notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhetoricAnalysis {
    #[serde(default)]
    pub devices: Vec<RhetoricalDevice>,
    #[serde(default)]
    pub language_style: String,
    #[serde(default)]
    pub notable_quotes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhetoricalDevice {
    pub name: String,
    #[serde(default)]
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionAnalysis {
    pub overall_tone: String,
    #[serde(default)]
    pub emotion_arc: Vec<EmotionPoint>,
    #[serde(default)]
    pub atmosphere_techniques: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionPoint {
    pub segment: String,
    pub emotion: String,
    #[serde(default)]
    pub intensity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemesAnalysis {
    #[serde(default)]
    pub motifs: Vec<String>,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_commentary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldbuildingAnalysis {
    #[serde(default)]
    pub locations: Vec<WorldElement>,
    #[serde(default)]
    pub organizations: Vec<WorldElement>,
    #[serde(default)]
    pub power_systems: Vec<String>,
    #[serde(default)]
    pub items: Vec<WorldElement>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insights: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldElement {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

// ---- Novel Summary ----

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NovelSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_plot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_arcs: Option<Vec<CharacterArc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub themes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writing_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worldbuilding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterArc {
    pub name: String,
    pub arc: String,
}

// ---- LLM Config ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub max_context_tokens: i32,
    #[serde(default = "default_max_output_tokens")]
    pub max_output_tokens: Option<u32>,
    pub temperature: f32,
}

fn default_max_output_tokens() -> Option<u32> {
    Some(8192)
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o".to_string(),
            max_context_tokens: 1000000,
            max_output_tokens: Some(8192),
            temperature: 0.7,
        }
    }
}
