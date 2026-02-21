import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import { useNovelStore } from '../store/novelStore';
import type { LlmConfig } from '../types';
import { X, Save, RefreshCw } from 'lucide-react';
import { motion } from 'framer-motion';

const MODEL_PRESETS: { name: string; tokens: number }[] = [
    { name: 'Gemini 3.1 Pro', tokens: 1000000 },
    { name: 'GPT-5.2', tokens: 400000 },
    { name: 'DeepSeek-V3.2', tokens: 131000 },
    { name: 'Kimi K2.5', tokens: 262000 },
];

interface Props {
    onClose: () => void;
}

export default function LlmConfigModal({ onClose }: Props) {
    const { llmConfig, fetchLlmConfig, saveLlmConfig, availableModels, fetchModels } = useNovelStore();
    const [config, setConfig] = useState<LlmConfig>(llmConfig);
    const [saving, setSaving] = useState(false);
    const [fetchingModels, setFetchingModels] = useState(false);

    useEffect(() => {
        fetchLlmConfig();
    }, []);

    useEffect(() => {
        setConfig(llmConfig);
    }, [llmConfig]);

    // We no longer need the native dialog ref since we are building a custom Tailwind modal.
    // Handle Escape key to close
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };
        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, [onClose]);

    // Auto-fetch models when modal opens if base_url and api_key are set
    useEffect(() => {
        if (config.base_url && config.api_key && availableModels.length === 0) {
            handleFetchModels();
        }
    }, []);

    const handleSave = async () => {
        setSaving(true);
        try {
            await saveLlmConfig(config);
            onClose();
        } catch (e) {
            console.error('Failed to save config:', e);
        }
        setSaving(false);
    };

    const handleFetchModels = async () => {
        // Save current config first so the backend uses the latest base_url/api_key
        setFetchingModels(true);
        try {
            await saveLlmConfig(config);
            await fetchModels();
        } catch (e) {
            console.error('Failed to fetch models:', e);
        }
        setFetchingModels(false);
    };

    return createPortal(
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            {/* Backdrop */}
            <motion.div
                className="absolute inset-0 bg-base-300/60 backdrop-blur-sm shadow-xl"
                onClick={onClose}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.2 }}
            />

            {/* Modal Content */}
            <motion.div
                className="relative z-10 w-full max-w-lg bg-base-200 rounded-2xl shadow-2xl border border-base-content/10 flex flex-col max-h-[90vh]"
                initial={{ opacity: 0, scale: 0.95, y: 10 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95, y: 10 }}
                transition={{ type: "spring", bounce: 0, duration: 0.3 }}
            >
                <div className="flex items-center justify-between p-4 border-b border-base-300 shrink-0">
                    <h3 className="font-bold text-lg">LLM 配置</h3>
                    <button className="btn btn-ghost btn-sm btn-square" onClick={onClose} title="关闭 (Esc)">
                        <X size={16} />
                    </button>
                </div>

                <div className="p-4 overflow-y-auto space-y-4">
                    {/* Base URL */}
                    <div className="form-control">
                        <label className="label"><span className="label-text">API Base URL</span></label>
                        <input
                            type="text"
                            className="input input-bordered input-sm w-full focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                            value={config.base_url}
                            onChange={(e) => setConfig({ ...config, base_url: e.target.value })}
                            placeholder="https://api.openai.com/v1"
                        />
                    </div>

                    {/* API Key */}
                    <div className="form-control">
                        <label className="label"><span className="label-text">API Key</span></label>
                        <input
                            type="password"
                            className="input input-bordered input-sm w-full focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                            value={config.api_key}
                            onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
                            placeholder="sk-..."
                        />
                    </div>

                    {/* Model */}
                    <div className="form-control">
                        <label className="label"><span className="label-text">模型名称</span></label>
                        <div className="flex gap-2">
                            {availableModels.length > 0 ? (
                                <select
                                    className="select select-bordered select-sm flex-1 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                                    value={config.model}
                                    onChange={(e) => setConfig({ ...config, model: e.target.value })}
                                >
                                    {!availableModels.includes(config.model) && config.model && (
                                        <option value={config.model}>{config.model}</option>
                                    )}
                                    {availableModels.map((m) => (
                                        <option key={m} value={m}>{m}</option>
                                    ))}
                                </select>
                            ) : (
                                <input
                                    type="text"
                                    className="input input-bordered input-sm flex-1 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                                    value={config.model}
                                    onChange={(e) => setConfig({ ...config, model: e.target.value })}
                                    placeholder="gpt-4o"
                                />
                            )}
                            <button
                                className="btn btn-sm btn-outline btn-square focus:outline-none focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                                onClick={handleFetchModels}
                                disabled={fetchingModels || !config.base_url}
                                title="刷新模型列表"
                            >
                                {fetchingModels
                                    ? <span className="loading loading-spinner loading-xs" />
                                    : <RefreshCw size={14} />}
                            </button>
                        </div>
                        {availableModels.length > 0 && (
                            <label className="label">
                                <span className="label-text-alt text-base-content/40">
                                    已获取 {availableModels.length} 个模型
                                </span>
                            </label>
                        )}
                    </div>

                    {/* Max Context Tokens */}
                    <div className="form-control">
                        <label className="label"><span className="label-text">最大上下文 Token 数</span></label>
                        <div className="flex gap-2">
                            <input
                                type="number"
                                className="input input-bordered input-sm flex-1 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                                value={config.max_context_tokens}
                                onChange={(e) => setConfig({ ...config, max_context_tokens: parseInt(e.target.value) || 0 })}
                            />
                            <div className="dropdown dropdown-end">
                                <label tabIndex={0} className="btn btn-sm btn-outline focus:outline-none focus:ring-1 focus:ring-primary shadow-sm transition-shadow">预设</label>
                                <ul tabIndex={0} className="dropdown-content menu p-2 shadow bg-base-300 rounded-box w-52 z-10">
                                    {MODEL_PRESETS.map((preset) => (
                                        <li key={preset.name}>
                                            <a onClick={() => setConfig({ ...config, max_context_tokens: preset.tokens })}>
                                                {preset.name} — {preset.tokens.toLocaleString()}
                                            </a>
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        </div>
                    </div>

                    {/* Max Output Tokens */}
                    <div className="grid grid-cols-2 gap-4">
                        <div className="form-control">
                            <label className="label"><span className="label-text">章节输出上限</span></label>
                            <input
                                type="number"
                                className="input input-bordered input-sm w-full focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                                value={config.chapter_max_tokens || 8192}
                                onChange={(e) => setConfig({ ...config, chapter_max_tokens: parseInt(e.target.value) || null })}
                            />
                        </div>
                        <div className="form-control">
                            <label className="label"><span className="label-text">全书汇总输出上限</span></label>
                            <input
                                type="number"
                                className="input input-bordered input-sm w-full focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                                value={config.summary_max_tokens || 16384}
                                onChange={(e) => setConfig({ ...config, summary_max_tokens: parseInt(e.target.value) || null })}
                            />
                        </div>
                    </div>

                    {/* Temperature */}
                    <div className="form-control mt-2">
                        <label className="label flex items-center justify-between pb-1">
                            <span className="label-text font-medium">Temperature</span>
                            <span className="font-mono text-xs bg-base-300 px-2 py-0.5 rounded-md">{config.temperature.toFixed(1)}</span>
                        </label>
                        <div className="py-2">
                            <input
                                type="range"
                                min="0"
                                max="1"
                                step="0.1"
                                className="range range-primary"
                                value={config.temperature}
                                onChange={(e) => setConfig({ ...config, temperature: parseFloat(e.target.value) })}
                            />
                        </div>
                    </div>

                    {/* Max Concurrent Tasks */}
                    <div className="form-control">
                        <label className="label"><span className="label-text">最大并发数 (批量分析)</span></label>
                        <input
                            type="number"
                            min="1"
                            max="20"
                            className="input input-bordered input-sm w-full focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                            value={config.max_concurrent_tasks || 3}
                            onChange={(e) => setConfig({ ...config, max_concurrent_tasks: Math.max(1, parseInt(e.target.value) || 3) })}
                        />
                    </div>

                    {/* Context Injection */}
                    <div className="form-control mt-4">
                        <label className="label pb-0">
                            <div className="relative group inline-block">
                                <span className="label-text font-medium cursor-help border-b border-dashed border-base-content/50">上下文注入</span>
                                <div className="absolute left-0 bottom-full mb-2 hidden group-hover:block z-[100] w-64 sm:w-[280px] p-3 bg-base-200 text-base-content text-xs rounded-lg shadow-xl border border-base-300">
                                    <h3 className="font-bold mb-1">提示</h3>
                                    <p className="whitespace-normal break-words leading-relaxed">
                                        开启上下文可极大增强 AI 对剧情连贯性的理解，但会增加 Token 消耗。<br /><br />
                                        选择“全部已有章节”时，将会把之前所有章节的剧情摘要拼接附上，篇幅较长的小说可能会占用较长上下文。
                                    </p>
                                </div>
                            </div>
                        </label>
                        <select
                            className="select select-bordered select-sm w-full mt-2 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                            value={config.context_injection_mode || 'None'}
                            onChange={(e) => setConfig({ ...config, context_injection_mode: e.target.value as any })}
                        >
                            <option value="None">无</option>
                            <option value="PreviousChapter">上一章节</option>
                            <option value="AllPrevious">全部已有章节</option>
                        </select>
                    </div>
                </div>

                <div className="flex justify-end gap-2 p-4 border-t border-base-300 shrink-0 bg-base-200 rounded-b-2xl">
                    <button className="btn btn-ghost btn-sm" onClick={onClose}>取消</button>
                    <button className="btn btn-primary btn-sm gap-2" onClick={handleSave} disabled={saving}>
                        {saving ? <span className="loading loading-spinner loading-xs" /> : <Save size={14} />}
                        保存
                    </button>
                </div>
            </motion.div>
        </div>,
        document.body
    );
}
