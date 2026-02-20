// ---- Analysis Dimensions ----

export type AnalysisDimension =
  | 'characters'
  | 'plot'
  | 'foreshadowing'
  | 'writing_technique'
  | 'rhetoric'
  | 'emotion'
  | 'themes'
  | 'worldbuilding';

export interface DimensionInfo {
  id: AnalysisDimension;
  name: string;
  icon: string;
  description: string;
  default: boolean;
}

// ---- Core Types ----

export interface NovelMeta {
  id: string;
  title: string;
  chapter_count: number;
  analyzed_count: number;
  created_at: string;
}

export interface Novel {
  id: string;
  title: string;
  source_type: SourceType;
  enabled_dimensions: AnalysisDimension[];
  created_at: string;
}

export type SourceType =
  | { Epub: string }
  | { TxtFiles: string[] }
  | { SingleTxt: string };

export interface ChapterMeta {
  id: number;
  index: number;
  title: string;
  has_analysis: boolean;
  token_estimate: number;
}

export interface Chapter {
  id: number | null;
  novel_id: string;
  index: number;
  title: string;
  content: string;
  analysis: ChapterAnalysis | null;
}

// ---- Analysis Types ----

export interface ChapterAnalysis {
  characters?: CharactersAnalysis;
  plot?: PlotAnalysis;
  foreshadowing?: ForeshadowingAnalysis;
  writing_technique?: WritingTechniqueAnalysis;
  rhetoric?: RhetoricAnalysis;
  emotion?: EmotionAnalysis;
  themes?: ThemesAnalysis;
  worldbuilding?: WorldbuildingAnalysis;
}

export interface CharactersAnalysis {
  characters: Character[];
  relationships: Relationship[];
  insights?: string;
}

export interface Character {
  name: string;
  role: string;
  traits: string[];
  actions: string;
}

export interface Relationship {
  from: string;
  to: string;
  relation_type: string;
  description: string;
  change?: string;
}

export interface PlotAnalysis {
  summary: string;
  key_events: KeyEvent[];
  conflicts: string[];
  suspense: string[];
  insights?: string;
}

export interface KeyEvent {
  event: string;
  cause?: string;
  effect?: string;
}

export interface ForeshadowingAnalysis {
  setups: ForeshadowItem[];
  callbacks: ForeshadowItem[];
  turning_points: string[];
  cliffhangers: string[];
  insights?: string;
}

export interface ForeshadowItem {
  content: string;
  chapter_ref?: string;
}

export interface WritingTechniqueAnalysis {
  narrative_perspective: string;
  time_sequence: string;
  pacing: string;
  structural_notes: string;
  insights?: string;
}

export interface RhetoricAnalysis {
  devices: RhetoricalDevice[];
  language_style: string;
  notable_quotes: string[];
  insights?: string;
}

export interface RhetoricalDevice {
  name: string;
  example: string;
}

export interface EmotionAnalysis {
  overall_tone: string;
  emotion_arc: EmotionPoint[];
  atmosphere_techniques: string[];
  insights?: string;
}

export interface EmotionPoint {
  segment: string;
  emotion: string;
  intensity: string;
}

export interface ThemesAnalysis {
  motifs: string[];
  values: string[];
  social_commentary?: string;
  insights?: string;
}

export interface WorldbuildingAnalysis {
  locations: WorldElement[];
  organizations: WorldElement[];
  power_systems: string[];
  items: WorldElement[];
  rules: string[];
  insights?: string;
}

export interface WorldElement {
  name: string;
  description: string;
}

// ---- Summary ----

export interface NovelSummary {
  created_at: string;
  overall_plot?: string;
  character_arcs?: CharacterArc[];
  themes?: string[];
  writing_style?: string;
  worldbuilding?: string;
}

export interface CharacterArc {
  name: string;
  arc: string;
}

// ---- LLM Config ----

export interface LlmConfig {
  base_url: string;
  api_key: string;
  model: string;
  max_context_tokens: number;
  max_output_tokens: number | null;
  temperature: number;
}

export type AnalysisMode = 'api' | 'manual';

// ---- Events ----

export interface ProgressEvent {
  novel_id: string;
  chapter_id: number | null;
  status: string;
  current: number;
  total: number;
  message: string;
}

export interface StreamingEvent {
  chapter_id: number;
  chunk: string;
  full_content: string;
}

// ---- EPUB Preview ----

export interface EpubPreviewChapter {
  index: number;
  title: string;
  char_count: number;
  suggested: boolean;
}

export interface EpubPreview {
  title: string;
  path: string;
  chapters: EpubPreviewChapter[];
}
