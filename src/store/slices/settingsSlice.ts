import { invoke } from '@tauri-apps/api/core';
import type { LlmConfig, DimensionInfo } from '../../types';
import type { StoreSlice, SettingsSlice } from '../types';

export const createSettingsSlice: StoreSlice<SettingsSlice> = (set, get) => ({
    llmConfig: {
        base_url: 'https://api.openai.com/v1',
        api_key: '',
        model: 'gpt-4o',
        max_context_tokens: 128000,
        chapter_max_tokens: 8192,
        summary_max_tokens: 16384,
        temperature: 0.3,
        max_concurrent_tasks: 3,
        context_injection_mode: 'None',
    },
    dimensions: [],
    availableModels: [],

    fetchLlmConfig: async () => {
        try {
            const llmConfig = await invoke<LlmConfig>('get_llm_config');
            set({ llmConfig });
        } catch (e) {
            console.error(e);
        }
    },

    saveLlmConfig: async (config) => {
        try {
            await invoke('save_llm_config', { config });
            set({ llmConfig: config });
        } catch (e) {
            get().setError(String(e));
            throw e;
        }
    },

    updateDimensions: async (dims) => {
        const novel = get().currentNovel;
        if (!novel) return;
        try {
            await invoke('update_novel_dimensions', { novelId: novel.id, dimensions: dims });
            set({ currentNovel: { ...novel, enabled_dimensions: dims } });
        } catch (e) {
            get().setError(String(e));
            throw e;
        }
    },

    fetchDimensions: async () => {
        try {
            const dimensions = await invoke<DimensionInfo[]>('get_all_dimensions');
            set({ dimensions });
        } catch (e) {
            console.error(e);
        }
    },

    fetchModels: async () => {
        try {
            const models = await invoke<string[]>('list_models');
            set({ availableModels: models });
        } catch (e) {
            get().setError(String(e));
            throw e;
        }
    },
});
