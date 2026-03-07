import { create } from 'zustand';
import type { StoreState } from './types';

import { createNovelSlice } from './slices/novelSlice';
import { createChapterSlice } from './slices/chapterSlice';
import { createAnalysisSlice } from './slices/analysisSlice';
import { createSummarySlice } from './slices/summarySlice';
import { createSettingsSlice } from './slices/settingsSlice';
import { createEventSlice } from './slices/eventSlice';

export const useNovelStore = create<StoreState>()((...a) => ({
    // Shared generic state
    loading: false,
    error: null,
    setError: (error) => a[0]({ error }),
    clearSelection: () => a[0]({ selectedChapter: null, novelSummary: null }),

    // Combine all slices
    ...createNovelSlice(...a),
    ...createChapterSlice(...a),
    ...createAnalysisSlice(...a),
    ...createSummarySlice(...a),
    ...createSettingsSlice(...a),
    ...createEventSlice(...a),
}));

export * from './types';
