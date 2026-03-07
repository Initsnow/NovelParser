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

    deleteChapter: async (chapterId, novelId) => {
        try {
            await invoke('delete_chapter', { chapterId });
            await get().fetchChapters(novelId);
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
});
