import type { StateCreator } from 'zustand';
import type {
    NovelMeta, Novel, ChapterMeta, Chapter, ChapterAnalysis,
    LlmConfig, AnalysisDimension, AnalysisMode, DimensionInfo, NovelSummary, ChapterOutline, BookOutline,
    ProgressEvent, EpubPreview,
} from '../types';

export interface SharedState {
    loading: boolean;
    error: string | null;
    setError: (error: string | null) => void;
    clearSelection: () => void;
}

export interface NovelSlice {
    novels: NovelMeta[];
    currentNovel: Novel | null;
    fetchNovels: () => Promise<void>;
    previewEpub: (path: string) => Promise<EpubPreview>;
    importEpubSelected: (path: string, selectedIndices: number[]) => Promise<string>;
    importTxtFiles: (paths: string[]) => Promise<string>;
    importSingleTxt: (path: string) => Promise<string>;
    deleteNovel: (id: string) => Promise<void>;
    selectNovel: (id: string) => Promise<void>;
}

export interface ChapterSlice {
    chapters: ChapterMeta[];
    selectedChapter: Chapter | null;
    fetchChapters: (novelId: string) => Promise<void>;
    selectChapter: (chapterId: number) => Promise<void>;
    hydrateChapterTokenEstimates: (chapterIds: number[]) => Promise<void>;
    deleteChapter: (chapterId: number, novelId: string) => Promise<void>;
    deleteChapters: (chapterIds: number[], novelId: string) => Promise<void>;
    clearChapterAnalysis: (chapterId: number, novelId: string) => Promise<void>;
    clearChapterOutline: (chapterId: number, novelId: string) => Promise<void>;
}

export interface AnalysisSlice {
    analysisMode: AnalysisMode;
    streamContent: Record<number, string>;
    analyzingChapterIds: Set<number>;
    progress: ProgressEvent | null;
    batchProgress: ProgressEvent | null;
    batchStartTime: number | null;
    setAnalysisMode: (mode: AnalysisMode) => void;
    generatePrompt: (chapterId: number) => Promise<string>;
    estimateTokens: (chapterId: number) => Promise<number>;
    analyzeChapterApi: (chapterId: number) => Promise<ChapterAnalysis>;
    parseManualResult: (json: string) => Promise<ChapterAnalysis>;
    saveAnalysis: (chapterId: number, analysis: ChapterAnalysis) => Promise<void>;
    batchAnalyzeNovel: (novelId: string) => Promise<void>;
    batchAnalyzeChapters: (novelId: string, chapterIds: number[]) => Promise<void>;
    cancelBatch: () => Promise<void>;
}

export interface SummarySlice {
    novelSummary: NovelSummary | null;
    fetchSummary: () => Promise<void>;
    generateFullSummary: (novelId: string) => Promise<void>;
    getFullSummaryManualPrompt: (novelId: string) => Promise<string>;
    parseManualFullSummaryResult: (json: string, novelId?: string) => Promise<NovelSummary>;
    clearNovelSummary: (novelId: string) => Promise<void>;
}

export interface OutlineSlice {
    bookOutline: BookOutline | null;
    outlineProgress: ProgressEvent | null;
    outlineBatchProgress: ProgressEvent | null;
    outlineBatchStartTime: number | null;
    outliningChapterIds: Set<number>;
    outlineStreamContent: Record<number, string>;
    fetchBookOutline: () => Promise<void>;
    generateChapterOutlineApi: (chapterId: number) => Promise<ChapterOutline>;
    batchGenerateOutlines: (novelId: string) => Promise<void>;
    batchGenerateOutlineChapters: (novelId: string, chapterIds: number[]) => Promise<void>;
    generateBookOutline: (novelId: string) => Promise<void>;
    clearBookOutline: (novelId: string) => Promise<void>;
}

export interface SettingsSlice {
    llmConfig: LlmConfig;
    dimensions: DimensionInfo[];
    availableModels: string[];
    fetchLlmConfig: () => Promise<void>;
    saveLlmConfig: (config: LlmConfig) => Promise<void>;
    fetchModels: () => Promise<void>;
    updateDimensions: (dims: AnalysisDimension[]) => Promise<void>;
    fetchDimensions: () => Promise<void>;
}

export interface EventSlice {
    initEventListeners: () => Promise<void>;
}

export type StoreState = SharedState & NovelSlice & ChapterSlice & AnalysisSlice & SummarySlice & OutlineSlice & SettingsSlice & EventSlice;
export type StoreSlice<T> = StateCreator<StoreState, [], [], T>;
