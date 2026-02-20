import { createPortal } from 'react-dom';

interface Props {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    kind?: 'warning' | 'error' | 'info';
    onConfirm: () => void;
    onCancel: () => void;
}

export default function ConfirmDialog({
    title, message, confirmText = '确定', cancelText = '取消',
    kind = 'warning', onConfirm, onCancel,
}: Props) {
    const btnClass = kind === 'error' ? 'btn-error' : kind === 'warning' ? 'btn-warning' : 'btn-primary';

    return createPortal(
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
            <div className="absolute inset-0 bg-black/40 backdrop-blur-sm" onClick={onCancel} />
            <div className="relative z-10 bg-base-200 rounded-2xl shadow-2xl border border-base-content/10 w-full max-w-sm p-6 flex flex-col gap-4 animate-[fadeIn_0.15s_ease-out]">
                <h3 className="font-bold text-lg">{title}</h3>
                <p className="text-sm text-base-content/70">{message}</p>
                <div className="flex justify-end gap-2 mt-2">
                    <button className="btn btn-ghost btn-sm" onClick={onCancel}>{cancelText}</button>
                    <button className={`btn btn-sm ${btnClass}`} onClick={onConfirm}>{confirmText}</button>
                </div>
            </div>
        </div>,
        document.body
    );
}
