import { invoke } from '@tauri-apps/api/core';
import type { ChapterAnalysis, AnalysisMode } from '../../types';
import type { StoreSlice, AnalysisSlice } from '../types';

export const createAnalysisSlice: StoreSlice<AnalysisSlice> = (set, get) => ({
    analysisMode: 'manual',
    streamContent: {},
    analyzingChapterIds: new Set<number>(),
    progress: null,
    batchProgress: null,
    batchStartTime: null,

    setAnalysisMode: (mode: AnalysisMode) => set({ analysisMode: mode }),

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

        const dims = get().currentNovel?.enabled_dimensions || [];

        try {
            const analysis = await invoke<ChapterAnalysis>('analyze_chapter_api', {
                chapterId,
                dimensions: dims
            });
            const after = new Set(get().analyzingChapterIds);
            after.delete(chapterId);
            const afterContent = { ...get().streamContent };
            delete afterContent[chapterId];

            set({ analyzingChapterIds: after, streamContent: afterContent });

            if (get().currentNovel) {
                await get().fetchChapters(get().currentNovel!.id);
            }
            if (get().selectedChapter?.id === chapterId) {
                await get().selectChapter(chapterId);
            }
            return analysis;
        } catch (e) {
            const after = new Set(get().analyzingChapterIds);
            after.delete(chapterId);
            const afterContent = { ...get().streamContent };
            delete afterContent[chapterId];

            set({ analyzingChapterIds: after, streamContent: afterContent });
            get().setError(String(e));
            throw e;
        }
    },

    parseManualResult: async (json) => {
        return invoke<ChapterAnalysis>('parse_manual_result', { jsonStr: json });
    },

    saveAnalysis: async (chapterId, analysis) => {
        try {
            await invoke('save_analysis', { chapterId, analysisData: analysis });
            if (get().currentNovel) {
                await get().fetchChapters(get().currentNovel!.id);
            }
            if (get().selectedChapter?.id === chapterId) {
                await get().selectChapter(chapterId);
            }
        } catch (e) {
            get().setError(String(e));
            throw e;
        }
    },

    batchAnalyzeNovel: async (novelId) => {
        try {
            set({ batchStartTime: Date.now() });
            await invoke('batch_analyze_novel', { novelId });
        } catch (e) {
            get().setError(String(e));
        } finally {
            if (get().currentNovel?.id === novelId) {
                await get().fetchChapters(novelId);
            }
        }
    },

    batchAnalyzeChapters: async (novelId, chapterIds) => {
        try {
            set({ batchStartTime: Date.now() });
            await invoke('batch_analyze_chapters', { novelId, chapterIds });
        } catch (e) {
            get().setError(String(e));
        } finally {
            if (get().currentNovel?.id === novelId) {
                await get().fetchChapters(novelId);
            }
        }
    },

    cancelBatch: async () => {
        try {
            await invoke('cancel_batch');
        } catch (e) {
            get().setError(String(e));
        }
    },
});
