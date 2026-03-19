import { invoke } from '@tauri-apps/api/core';
import type { BookOutline, ChapterOutline } from '../../types';
import type { OutlineSlice, StoreSlice } from '../types';

export const createOutlineSlice: StoreSlice<OutlineSlice> = (set, get) => ({
    bookOutline: null,
    outlineProgress: null,
    outlineBatchProgress: null,
    outlineBatchStartTime: null,
    outliningChapterIds: new Set<number>(),
    outlineStreamContent: {},

    fetchBookOutline: async () => {
        const novelId = get().currentNovel?.id;
        if (!novelId) return;

        try {
            const bookOutline = await invoke<BookOutline | null>('get_book_outline', { novelId });
            set({ bookOutline });
        } catch (e) {
            get().setError(String(e));
        }
    },

    generateChapterOutlineApi: async (chapterId) => {
        const ids = new Set(get().outliningChapterIds);
        ids.add(chapterId);
        set({
            outliningChapterIds: ids,
            error: null,
            outlineStreamContent: { ...get().outlineStreamContent, [chapterId]: '' }
        });

        try {
            const outline = await invoke<ChapterOutline>('generate_chapter_outline', { chapterId });
            const nextIds = new Set(get().outliningChapterIds);
            nextIds.delete(chapterId);
            const nextContent = { ...get().outlineStreamContent };
            delete nextContent[chapterId];
            set({ outliningChapterIds: nextIds, outlineStreamContent: nextContent });

            const currentNovel = get().currentNovel;
            if (currentNovel) {
                await get().fetchChapters(currentNovel.id);
                await get().fetchBookOutline();
            }
            if (get().selectedChapter?.id === chapterId) {
                await get().selectChapter(chapterId);
            }

            return outline;
        } catch (e) {
            const nextIds = new Set(get().outliningChapterIds);
            nextIds.delete(chapterId);
            const nextContent = { ...get().outlineStreamContent };
            delete nextContent[chapterId];
            set({ outliningChapterIds: nextIds, outlineStreamContent: nextContent });
            get().setError(String(e));
            throw e;
        }
    },

    batchGenerateOutlines: async (novelId) => {
        try {
            set({ outlineBatchStartTime: Date.now() });
            await invoke('batch_generate_outlines', { novelId });
        } catch (e) {
            get().setError(String(e));
        } finally {
            if (get().currentNovel?.id === novelId) {
                await get().fetchChapters(novelId);
                await get().fetchBookOutline();
                if (get().selectedChapter?.id) {
                    await get().selectChapter(get().selectedChapter!.id!);
                }
            }
        }
    },

    batchGenerateOutlineChapters: async (novelId, chapterIds) => {
        try {
            set({ outlineBatchStartTime: Date.now() });
            await invoke('batch_generate_outline_chapters', { novelId, chapterIds });
        } catch (e) {
            get().setError(String(e));
        } finally {
            if (get().currentNovel?.id === novelId) {
                await get().fetchChapters(novelId);
                await get().fetchBookOutline();
                if (get().selectedChapter?.id) {
                    await get().selectChapter(get().selectedChapter!.id!);
                }
            }
        }
    },

    generateBookOutline: async (novelId) => {
        set({ loading: true, error: null });
        try {
            const bookOutline = await invoke<BookOutline>('generate_book_outline', { novelId });
            set({ bookOutline, loading: false });
        } catch (e) {
            set({ loading: false });
            get().setError(String(e));
            throw e;
        }
    },

    clearBookOutline: async (novelId) => {
        try {
            await invoke('clear_book_outline', { novelId });
            if (get().currentNovel?.id === novelId) {
                set({ bookOutline: null });
            }
        } catch (e) {
            get().setError(String(e));
        }
    },
});
