import { invoke } from '@tauri-apps/api/core';
import type { ChapterMeta, Chapter } from '../../types';
import type { StoreSlice, ChapterSlice } from '../types';

export const createChapterSlice: StoreSlice<ChapterSlice> = (set, get) => ({
    chapters: [],
    selectedChapter: null,

    fetchChapters: async (novelId) => {
        try {
            const chapters = await invoke<ChapterMeta[]>('list_chapters', { novelId });
            set({ chapters });
        } catch (e) {
            get().setError(String(e));
        }
    },

    selectChapter: async (chapterId) => {
        set({ loading: true });
        try {
            const chapter = await invoke<Chapter>('get_chapter', { chapterId });
            set({ selectedChapter: chapter, loading: false });
        } catch (e) {
            set({ loading: false });
            get().setError(String(e));
        }
    },

    hydrateChapterTokenEstimates: async (chapterIds) => {
        const pendingIds = chapterIds.filter((id) => {
            const meta = get().chapters.find((ch) => ch.id === id);
            return meta && !meta.token_exact;
        });

        if (pendingIds.length === 0) return;

        try {
            const updates = await invoke<Array<{ chapter_id: number; token_count: number }>>(
                'hydrate_chapter_token_estimates',
                { chapterIds: pendingIds },
            );

            if (updates.length === 0) return;

            const updateMap = new Map(updates.map((item) => [item.chapter_id, item.token_count]));
            set({
                chapters: get().chapters.map((chapter) =>
                    updateMap.has(chapter.id)
                        ? {
                            ...chapter,
                            token_estimate: updateMap.get(chapter.id)!,
                            token_exact: true,
                        }
                        : chapter,
                ),
            });
        } catch (e) {
            get().setError(String(e));
        }
    },

    deleteChapter: async (chapterId, novelId) => {
        try {
            await invoke('delete_chapter', { chapterId });
            await get().fetchChapters(novelId);
            await get().fetchBookOutline();
            const selected = get().selectedChapter;
            if (selected && selected.id === chapterId) {
                set({ selectedChapter: null });
            }
        } catch (e) {
            get().setError(String(e));
        }
    },

    deleteChapters: async (chapterIds, novelId) => {
        try {
            await invoke('delete_chapters', { chapterIds });
            await get().fetchChapters(novelId);
            await get().fetchBookOutline();
            const selected = get().selectedChapter;
            if (selected && selected.id && chapterIds.includes(selected.id)) {
                set({ selectedChapter: null });
            }
        } catch (e) {
            get().setError(String(e));
        }
    },

    clearChapterAnalysis: async (chapterId, novelId) => {
        try {
            await invoke('clear_chapter_analysis', { chapterId });
            await get().fetchChapters(novelId);
            set({ selectedChapter: null });
        } catch (e) {
            get().setError(String(e));
        }
    },

    clearChapterOutline: async (chapterId, novelId) => {
        try {
            await invoke('clear_chapter_outline', { chapterId });
            await get().fetchChapters(novelId);
            await get().fetchBookOutline();
            set({ selectedChapter: null });
        } catch (e) {
            get().setError(String(e));
        }
    },
});
