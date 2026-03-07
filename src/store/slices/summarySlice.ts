import { invoke } from '@tauri-apps/api/core';
import type { NovelSummary } from '../../types';
import type { StoreSlice, SummarySlice } from '../types';

export const createSummarySlice: StoreSlice<SummarySlice> = (set, get) => ({
    novelSummary: null,

    fetchSummary: async () => {
        const novelId = get().currentNovel?.id;
        if (!novelId) return;
        try {
            const novelSummary = await invoke<NovelSummary | null>('get_novel_summary', { novelId });
            set({ novelSummary });
        } catch (e) {
            get().setError(String(e));
        }
    },

    generateFullSummary: async (novelId) => {
        set({ loading: true, error: null });
        try {
            const summary = await invoke<NovelSummary>('generate_full_summary', { novelId });
            set({ novelSummary: summary, loading: false });
        } catch (e) {
            set({ loading: false });
            get().setError(String(e));
            throw e;
        }
    },

    getFullSummaryManualPrompt: async (novelId) => {
        return invoke<string>('get_full_summary_manual_prompt', { novelId });
    },

    parseManualFullSummaryResult: async (json, novelId) => {
        set({ loading: true, error: null });
        try {
            const summary = await invoke<NovelSummary>('parse_manual_result', { jsonStr: json });
            summary.created_at = new Date().toISOString();

            if (novelId) {
                await invoke('save_novel_summary', { novelId, summary });
                if (get().currentNovel?.id === novelId) {
                    set({ novelSummary: summary });
                }
            }

            set({ loading: false });
            return summary;
        } catch (e) {
            set({ loading: false });
            get().setError(String(e));
            throw e;
        }
    },

    clearNovelSummary: async (novelId) => {
        try {
            await invoke('clear_novel_summary', { novelId });
            if (get().currentNovel?.id === novelId) {
                set({ novelSummary: null });
            }
        } catch (e) {
            get().setError(String(e));
        }
    },
});
