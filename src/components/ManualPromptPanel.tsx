import { useState, useEffect } from 'react';
import { useNovelStore } from '../store/novelStore';
import { Copy, Check, Send, AlertTriangle } from 'lucide-react';

interface Props {
    chapterId: number;
    chapterTitle: string;
    onSuccess?: () => void;
}

export default function ManualPromptPanel({ chapterId, chapterTitle, onSuccess }: Props) {
    const { generatePrompt, estimateTokens, parseManualResult, saveAnalysis, llmConfig } = useNovelStore();
    const [prompt, setPrompt] = useState('');
    const [tokenCount, setTokenCount] = useState(0);
    const [responseJson, setResponseJson] = useState('');
    const [copied, setCopied] = useState(false);
    const [parsing, setParsing] = useState(false);
    const [parseError, setParseError] = useState<string | null>(null);

    useEffect(() => {
        loadPrompt();
    }, [chapterId]);

    const loadPrompt = async () => {
        try {
            const [p, t] = await Promise.all([
                generatePrompt(chapterId),
                estimateTokens(chapterId),
            ]);
            setPrompt(p);
            setTokenCount(t);
            setResponseJson('');
            setParseError(null);
        } catch (e) {
            console.error('Failed to generate prompt:', e);
        }
    };

    const handleCopy = async () => {
        await navigator.clipboard.writeText(prompt);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    const handleParseAndSave = async () => {
        setParsing(true);
        setParseError(null);
        try {
            const analysis = await parseManualResult(responseJson);
            await saveAnalysis(chapterId, analysis);
            if (onSuccess) onSuccess();
        } catch (e) {
            setParseError(String(e));
        }
        setParsing(false);
    };

    const isOverLimit = tokenCount > llmConfig.max_context_tokens;

    return (
        <div className="space-y-4 max-w-3xl mx-auto">
            <h3 className="text-lg font-bold">手动分析：{chapterTitle}</h3>

            {/* Token info */}
            <div className={`flex items-center gap-2 text-sm ${isOverLimit ? 'text-error' : 'text-base-content/60'}`}>
                {isOverLimit && <AlertTriangle size={14} />}
                <span>
                    预估 Prompt: {tokenCount.toLocaleString()} tokens
                    / 模型上限: {llmConfig.max_context_tokens.toLocaleString()}
                </span>
            </div>

            {/* Step 1: Copy prompt */}
            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-4">
                    <div className="flex items-center justify-between mb-2">
                        <h4 className="font-semibold text-sm">① 复制 Prompt</h4>
                        <button className="btn btn-sm btn-outline gap-2" onClick={handleCopy}>
                            {copied ? <Check size={14} className="text-success" /> : <Copy size={14} />}
                            {copied ? '已复制' : '复制'}
                        </button>
                    </div>
                    <textarea
                        className="textarea textarea-bordered text-xs font-mono w-full h-40 resize-y focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                        value={prompt}
                        readOnly
                    />
                </div>
            </div>

            {/* Step 2: Paste response */}
            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-4">
                    <h4 className="font-semibold text-sm mb-2">② 粘贴 AI 返回的 JSON</h4>
                    <textarea
                        className="textarea textarea-bordered text-xs font-mono w-full h-40 resize-y focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm transition-shadow"
                        value={responseJson}
                        onChange={(e) => setResponseJson(e.target.value)}
                        placeholder='将 AI 返回的 JSON 结果粘贴到这里...'
                    />

                    {parseError && (
                        <div className="alert alert-error alert-sm mt-2">
                            <AlertTriangle size={14} />
                            <span className="text-xs">{parseError}</span>
                        </div>
                    )}

                    <button
                        className="btn btn-primary btn-sm gap-2 mt-3"
                        onClick={handleParseAndSave}
                        disabled={!responseJson.trim() || parsing}
                    >
                        {parsing ? <span className="loading loading-spinner loading-xs" /> : <Send size={14} />}
                        解析并保存
                    </button>
                </div>
            </div>
        </div>
    );
}
