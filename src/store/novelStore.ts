import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
    NovelMeta, Novel, ChapterMeta, Chapter, ChapterAnalysis,
    LlmConfig, AnalysisDimension, AnalysisMode, DimensionInfo, NovelSummary,
    ProgressEvent, StreamingEvent, EpubPreview,
} from '../types';

interface NovelStore {
    // State
    novels: NovelMeta[];
    currentNovel: Novel | null;
    chapters: ChapterMeta[];
    selectedChapter: Chapter | null;
    llmConfig: LlmConfig;
    analysisMode: AnalysisMode;
    dimensions: DimensionInfo[];
    novelSummary: NovelSummary | null;
    availableModels: string[];
    progress: ProgressEvent | null;
    batchProgress: ProgressEvent | null;
    batchStartTime: number | null;
    streamContent: Record<number, string>;
    analyzingChapterIds: Set<number>;
    loading: boolean;
    error: string | null;

    // Actions
    fetchNovels: () => Promise<void>;
    previewEpub: (path: string) => Promise<EpubPreview>;
    importEpubSelected: (path: string, selectedIndices: number[]) => Promise<string>;
    importTxtFiles: (paths: string[]) => Promise<string>;
    importSingleTxt: (path: string) => Promise<string>;
    deleteNovel: (id: string) => Promise<void>;
    deleteChapter: (chapterId: number, novelId: string) => Promise<void>;
    deleteChapters: (chapterIds: number[], novelId: string) => Promise<void>;
    clearChapterAnalysis: (chapterId: number, novelId: string) => Promise<void>;
    selectNovel: (id: string) => Promise<void>;
    fetchChapters: (novelId: string) => Promise<void>;
    selectChapter: (chapterId: number) => Promise<void>;
    generatePrompt: (chapterId: number) => Promise<string>;
    estimateTokens: (chapterId: number) => Promise<number>;
    analyzeChapterApi: (chapterId: number) => Promise<ChapterAnalysis>;
    parseManualResult: (json: string) => Promise<ChapterAnalysis>;
    saveAnalysis: (chapterId: number, analysis: ChapterAnalysis) => Promise<void>;
    fetchLlmConfig: () => Promise<void>;
    saveLlmConfig: (config: LlmConfig) => Promise<void>;
    updateDimensions: (dims: AnalysisDimension[]) => Promise<void>;
    fetchDimensions: () => Promise<void>;
    fetchSummary: () => Promise<void>;
    generateFullSummary: (novelId: string) => Promise<void>;
    getFullSummaryManualPrompt: (novelId: string) => Promise<string>;
    parseManualFullSummaryResult: (json: string, novelId?: string) => Promise<NovelSummary>;
    fetchModels: () => Promise<void>;
    setAnalysisMode: (mode: AnalysisMode) => void;
    setError: (error: string | null) => void;
    clearSelection: () => void;
    clearNovelSummary: (novelId: string) => Promise<void>;
    batchAnalyzeNovel: (novelId: string) => Promise<void>;
    batchAnalyzeChapters: (novelId: string, chapterIds: number[]) => Promise<void>;
    cancelBatch: () => Promise<void>;
    initEventListeners: () => Promise<void>;
}

export const useNovelStore = create<NovelStore>((set, get) => ({
    novels: [],
    currentNovel: null,
    chapters: [],
    selectedChapter: null,
    llmConfig: {
        base_url: 'https://api.openai.com/v1',
        api_key: '',
        model: 'gpt-4o',
        max_context_tokens: 128000,
        chapter_max_tokens: 8192,
        summary_max_tokens: 16384,
        temperature: 0.3,
        max_concurrent_tasks: 3,
    },
    analysisMode: 'manual',
    dimensions: [],
    novelSummary: null,
    availableModels: [],
    progress: null,
    batchProgress: null,
    batchStartTime: null,
    streamContent: {},
    analyzingChapterIds: new Set<number>(),
    loading: false,
    error: null,

    fetchNovels: async () => {
        try {
            const novels = await invoke<NovelMeta[]>('list_novels');
            set({ novels });
        } catch (e) {
            set({ error: String(e) });
        }
    },

    previewEpub: async (path) => {
        set({ loading: true, error: null });
        try {
            const preview = await invoke<EpubPreview>('preview_epub', { path });
            set({ loading: false });
            return preview;
        } catch (e) {
            set({ loading: false, error: String(e) });
            throw e;
        }
    },

    importEpubSelected: async (path, selectedIndices) => {
        set({ loading: true, error: null });
        try {
            const id = await invoke<string>('import_epub_selected', { path, selectedIndices });
            await get().fetchNovels();
            set({ loading: false });
            return id;
        } catch (e) {
            set({ loading: false, error: String(e) });
            throw e;
        }
    },

    importTxtFiles: async (paths) => {
        set({ loading: true, error: null });
        try {
            const id = await invoke<string>('import_txt_files', { paths });
            await get().fetchNovels();
            set({ loading: false });
            return id;
        } catch (e) {
            set({ loading: false, error: String(e) });
            throw e;
        }
    },

    importSingleTxt: async (path) => {
        set({ loading: true, error: null });
        try {
            const id = await invoke<string>('import_single_txt', { path });
            await get().fetchNovels();
            set({ loading: false });
            return id;
        } catch (e) {
            set({ loading: false, error: String(e) });
            throw e;
        }
    },

    deleteNovel: async (id) => {
        try {
            await invoke('delete_novel', { novelId: id });
            await get().fetchNovels();
            if (get().currentNovel?.id === id) {
                set({ currentNovel: null, chapters: [], selectedChapter: null });
            }
        } catch (e) {
            set({ error: String(e) });
        }
    },

    deleteChapter: async (chapterId, novelId) => {
        try {
            await invoke('delete_chapter', { chapterId });
            await get().fetchChapters(novelId);
            const selected = get().selectedChapter;
            if (selected && selected.id === chapterId) {
                set({ selectedChapter: null });
            }
        } catch (e) {
            set({ error: String(e) });
        }
    },

    deleteChapters: async (chapterIds, novelId) => {
        try {
            await invoke('delete_chapters', { chapterIds });
            await get().fetchChapters(novelId);
            const selected = get().selectedChapter;
            if (selected && selected.id && chapterIds.includes(selected.id)) {
                set({ selectedChapter: null });
            }
        } catch (e) {
            set({ error: String(e) });
        }
    },

    clearChapterAnalysis: async (chapterId, novelId) => {
        try {
            await invoke('clear_chapter_analysis', { chapterId });
            await get().fetchChapters(novelId);
            set({ selectedChapter: null });
        } catch (e) {
            set({ error: String(e) });
        }
    },

    selectNovel: async (id) => {
        set({ loading: true, error: null });
        try {
            const novel = await invoke<Novel>('get_novel', { novelId: id });
            set({ currentNovel: novel, selectedChapter: null, novelSummary: null });
            await get().fetchChapters(id);
            set({ loading: false });
        } catch (e) {
            set({ loading: false, error: String(e) });
        }
    },

    fetchChapters: async (novelId) => {
        try {
            const chapters = await invoke<ChapterMeta[]>('list_chapters', { novelId });
            set({ chapters });
        } catch (e) {
            set({ error: String(e) });
        }
    },

    selectChapter: async (chapterId) => {
        set({ loading: true });
        try {
            const chapter = await invoke<Chapter>('get_chapter', { chapterId });
            set({ selectedChapter: chapter, loading: false });
        } catch (e) {
            set({ loading: false, error: String(e) });
        }
    },

    generatePrompt: async (chapterId) => {
        const dims = get().currentNovel?.enabled_dimensions || [];
        return invoke<string>('generate_prompt', { chapterId, dimensions: dims });
    },

    estimateTokens: async (chapterId) => {
        const dims = get().currentNovel?.enabled_dimensions || [];
        return invoke<number>('estimate_prompt_tokens', { chapterId, dimensions: dims });
    },

    analyzeChapterApi: async (chapterId) => {
        const ids = new Set(get().analyzingChapterIds);
        ids.add(chapterId);
        set({
            analyzingChapterIds: ids,
            error: null,
            streamContent: { ...get().streamContent, [chapterId]: '' }
        });
        try {
            const dims = get().currentNovel?.enabled_dimensions || [];
            const analysis = await invoke<ChapterAnalysis>('analyze_chapter_api', {
                chapterId, dimensions: dims,
            });
            // Refresh chapter and list
            await get().selectChapter(chapterId);
            if (get().currentNovel) {
                await get().fetchChapters(get().currentNovel!.id);
            }
            const after = new Set(get().analyzingChapterIds);
            after.delete(chapterId);
            const afterContent = { ...get().streamContent };
            delete afterContent[chapterId];

            set({ analyzingChapterIds: after, streamContent: afterContent });
            return analysis;
        } catch (e) {
            const after = new Set(get().analyzingChapterIds);
            after.delete(chapterId);

            const afterContent = { ...get().streamContent };
            delete afterContent[chapterId];

            set({ analyzingChapterIds: after, streamContent: afterContent, error: String(e) });
            throw e;
        }
    },

    parseManualResult: async (json) => {
        return invoke<ChapterAnalysis>('parse_manual_result', { jsonStr: json });
    },

    saveAnalysis: async (chapterId, analysis) => {
        await invoke('save_analysis', { chapterId, analysisData: analysis });
        await get().selectChapter(chapterId);
        if (get().currentNovel) {
            await get().fetchChapters(get().currentNovel!.id);
        }
    },

    fetchLlmConfig: async () => {
        try {
            const config = await invoke<LlmConfig>('get_llm_config');
            set({ llmConfig: config });
        } catch (e) {
            set({ error: String(e) });
        }
    },

    saveLlmConfig: async (config) => {
        await invoke('save_llm_config', { config });
        set({ llmConfig: config });
    },

    updateDimensions: async (dims) => {
        const novel = get().currentNovel;
        if (novel) {
            await invoke('update_novel_dimensions', { novelId: novel.id, dimensions: dims });
            set({ currentNovel: { ...novel, enabled_dimensions: dims } });
        }
    },

    fetchDimensions: async () => {
        try {
            const dims = await invoke<DimensionInfo[]>('get_all_dimensions');
            set({ dimensions: dims });
        } catch (e) {
            set({ error: String(e) });
        }
    },

    fetchSummary: async () => {
        const novel = get().currentNovel;
        if (novel) {
            try {
                const summary = await invoke<NovelSummary | null>('get_novel_summary', { novelId: novel.id });
                set({ novelSummary: summary });
            } catch (e) {
                set({ error: String(e) });
            }
        }
    },

    generateFullSummary: async (novelId) => {
        set({ loading: true, error: null });
        try {
            const summary = await invoke<NovelSummary>('generate_full_summary', { novelId });
            set({ novelSummary: summary, loading: false });
        } catch (e) {
            set({ loading: false, error: String(e) });
            throw e;
        }
    },

    getFullSummaryManualPrompt: async (novelId) => {
        return invoke<string>('get_full_summary_manual_prompt', { novelId });
    },

    parseManualFullSummaryResult: async (json, novelId) => {
        const text = json.trim();
        const start = text.indexOf('{');
        const end = text.lastIndexOf('}');
        if (start === -1 || end === -1) {
            throw new Error("无效的 JSON 格式");
        }
        const cleanJson = text.substring(start, end + 1);
        try {
            const parsed = JSON.parse(cleanJson) as NovelSummary;
            parsed.created_at = new Date().toISOString();
            set({ novelSummary: parsed });

            if (novelId) {
                await invoke('save_novel_summary', { novelId, summary: parsed });
            }

            return parsed;
        } catch (e) {
            throw new Error("JSON 解析失败: " + String(e));
        }
    },

    clearNovelSummary: async (novelId) => {
        try {
            await invoke('clear_novel_summary', { novelId });
            set({ novelSummary: null });
        } catch (e) {
            console.error('Failed to clear novel summary:', e);
            throw e;
        }
    },

    setAnalysisMode: (mode) => set({ analysisMode: mode }),

    fetchModels: async () => {
        try {
            const models = await invoke<string[]>('list_models');
            set({ availableModels: models });
        } catch (e) {
            set({ error: String(e) });
        }
    },

    setError: (error) => set({ error }),
    clearSelection: () => set({ selectedChapter: null }),

    batchAnalyzeNovel: async (novelId) => {
        set({ loading: true, error: null });
        try {
            await invoke('batch_analyze_novel', { novelId });
            await get().fetchChapters(novelId);
            set({ loading: false });
        } catch (e) {
            set({ loading: false, error: String(e) });
            throw e;
        }
    },

    cancelBatch: async () => {
        await invoke('cancel_batch');
    },

    batchAnalyzeChapters: async (novelId, chapterIds) => {
        set({ loading: true, error: null });
        try {
            await invoke('batch_analyze_chapters', { novelId, chapterIds });
            await get().fetchChapters(novelId);
            set({ loading: false });
        } catch (e) {
            set({ loading: false, error: String(e) });
            throw e;
        }
    },

    initEventListeners: async () => {
        await listen<ProgressEvent>('analysis_progress', (event) => {
            set({ progress: event.payload });
            if (event.payload.status === 'done' || event.payload.status === 'error') {
                setTimeout(() => set({ progress: null }), 3000);
            }
        });

        await listen<ProgressEvent>('batch_progress', (event) => {
            const payload = event.payload;
            const currentStatus = payload.status;

            set((state) => {
                let newStartTime = state.batchStartTime;
                if (currentStatus === 'batch_analyzing' && !state.batchStartTime) {
                    newStartTime = Date.now();
                } else if (currentStatus === 'batch_done' || currentStatus === 'batch_cancelled') {
                    newStartTime = null;
                }

                // Track active chapters during batch
                const newAnalyzing = new Set(state.analyzingChapterIds);
                if (currentStatus === 'batch_analyzing' && payload.chapter_id) {
                    newAnalyzing.add(payload.chapter_id);
                } else if ((currentStatus === 'chapter_done' || currentStatus === 'error') && payload.chapter_id) {
                    newAnalyzing.delete(payload.chapter_id);
                } else if (currentStatus === 'batch_cancelled' || currentStatus === 'batch_done') {
                    // Optional: clear all if needed, but they usually clear via chapter_done
                }

                return {
                    batchProgress: payload,
                    batchStartTime: newStartTime,
                    analyzingChapterIds: newAnalyzing
                };
            });

            // Refresh chapter list when a chapter finishes analyzing
            if (payload.status === 'chapter_done') {
                if (get().currentNovel?.id === payload.novel_id) {
                    get().fetchChapters(payload.novel_id);
                }
            }

            if (payload.status === 'batch_done' || payload.status === 'batch_cancelled' || payload.status === 'error') {
                setTimeout(() => set({ batchProgress: null }), 3000);
                if (get().currentNovel?.id === payload.novel_id) {
                    get().fetchChapters(payload.novel_id);
                }
                set({ loading: false });
            }
        });

        await listen<StreamingEvent>('analysis_streaming', (event) => {
            const payload = event.payload;
            set({
                streamContent: {
                    ...get().streamContent,
                    [payload.chapter_id]: payload.full_content
                }
            });
        });
    },
}));
