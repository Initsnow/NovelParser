import { listen } from '@tauri-apps/api/event';
import type { ProgressEvent, StreamingEvent } from '../../types';
import type { StoreSlice, EventSlice } from '../types';

let eventListenersInitialized = false;

export const createEventSlice: StoreSlice<EventSlice> = (set, get) => ({
    initEventListeners: async () => {
        // Prevent re-initialization
        if (eventListenersInitialized) return;
        eventListenersInitialized = true;

        await listen<ProgressEvent>('analysis_progress', (event) => {
            const p = event.payload;
            if (p.status === 'done' || p.status === 'error') {
                set({ progress: null });
            } else {
                set({ progress: p });
            }
        });

        await listen<ProgressEvent>('batch_progress', (event) => {
            const p = event.payload;
            if (p.status === 'batch_done' || p.status === 'batch_cancelled') {
                set({ batchProgress: null, batchStartTime: null });
            } else {
                set({ batchProgress: p });
            }
        });

        await listen<StreamingEvent>('analysis_streaming', (event) => {
            const payload = event.payload;
            const current = get().streamContent[payload.chapter_id] || '';
            set({
                streamContent: {
                    ...get().streamContent,
                    [payload.chapter_id]: current + payload.chunk
                }
            });
        });
    },
});
