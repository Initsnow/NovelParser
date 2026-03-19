import { invoke } from '@tauri-apps/api/core';
import type { NovelMeta, Novel, EpubPreview } from '../../types';
import type { StoreSlice, NovelSlice } from '../types';

export const createNovelSlice: StoreSlice<NovelSlice> = (set, get) => ({
    novels: [],
    currentNovel: null,

    fetchNovels: async () => {
        try {
            const novels = await invoke<NovelMeta[]>('list_novels');
            set({ novels });
        } catch (e) {
            get().setError(String(e));
        }
    },

    previewEpub: async (path) => {
        set({ loading: true, error: null });
        try {
            const preview = await invoke<EpubPreview>('preview_epub', { path });
            set({ loading: false });
            return preview;
        } catch (e) {
            set({ loading: false });
            get().setError(String(e));
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
            set({ loading: false });
            get().setError(String(e));
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
            set({ loading: false });
            get().setError(String(e));
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
            set({ loading: false });
            get().setError(String(e));
            throw e;
        }
    },

    deleteNovel: async (id) => {
        try {
            await invoke('delete_novel', { novelId: id });
            await get().fetchNovels();
            if (get().currentNovel?.id === id) {
                set({ currentNovel: null });
                get().clearSelection();
            }
        } catch (e) {
            get().setError(String(e));
        }
    },

    selectNovel: async (id) => {
        set({ loading: true, error: null });
        try {
            const novel = await invoke<Novel>('get_novel', { novelId: id });
            set({ currentNovel: novel });
            get().clearSelection();
            await get().fetchChapters(id);
            await get().fetchBookOutline();
            set({ loading: false });
        } catch (e) {
            set({ loading: false });
            get().setError(String(e));
        }
    },
});
