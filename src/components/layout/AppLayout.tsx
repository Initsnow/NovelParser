import { Outlet, NavLink } from 'react-router-dom';
import { BookOpen, Settings, Home, Sun, Moon } from 'lucide-react';
import { useNovelStore } from '../../store/novelStore';
import { useEffect, useState } from 'react';
import { AnimatePresence } from 'framer-motion';
import LlmConfigModal from '../../components/LlmConfigModal';

export default function AppLayout() {
    const { currentNovel, error, setError, initEventListeners } = useNovelStore();
    const [showConfig, setShowConfig] = useState(false);
    const [theme, setTheme] = useState(() => localStorage.getItem('theme') || 'night');

    useEffect(() => {
        initEventListeners();
    }, [initEventListeners]);

    useEffect(() => {
        localStorage.setItem('theme', theme);
        document.documentElement.setAttribute('data-theme', theme);
    }, [theme]);

    useEffect(() => {
        if (error) {
            const timer = setTimeout(() => setError(null), 5000);
            return () => clearTimeout(timer);
        }
    }, [error, setError]);

    return (
        <div className="flex h-screen bg-base-100">
            {/* Sidebar */}
            <aside className="w-16 bg-base-200 flex flex-col items-center py-4 gap-4 border-r border-base-300">
                <NavLink
                    to="/"
                    className={({ isActive }) =>
                        `btn btn-ghost btn-square ${isActive ? 'btn-active' : ''}`
                    }
                >
                    <Home size={20} />
                </NavLink>

                {currentNovel && (
                    <NavLink
                        to={`/novel/${currentNovel.id}`}
                        className={({ isActive }) =>
                            `btn btn-ghost btn-square ${isActive ? 'btn-active' : ''}`
                        }
                    >
                        <BookOpen size={20} />
                    </NavLink>
                )}

                <div className="flex-1" />

                <button
                    className="btn btn-ghost btn-square"
                    onClick={() => setTheme(theme === 'night' ? 'emerald' : 'night')}
                    title="切换主题"
                >
                    {theme === 'night' ? <Moon size={20} /> : <Sun size={20} />}
                </button>

                <button
                    className="btn btn-ghost btn-square"
                    onClick={() => setShowConfig(true)}
                    title="LLM 设置"
                >
                    <Settings size={20} />
                </button>
            </aside>

            {/* Main content */}
            <main className="flex-1 flex flex-col overflow-hidden">
                {/* Error toast */}
                {error && (
                    <div className="toast toast-top toast-end z-[9999]">
                        <div className="alert alert-error shadow-lg">
                            <span className="text-sm">{error}</span>
                            <button className="btn btn-ghost btn-xs" onClick={() => setError(null)}>✕</button>
                        </div>
                    </div>
                )}

                <Outlet />
            </main>

            {/* LLM Config Modal */}
            <AnimatePresence>
                {showConfig && <LlmConfigModal onClose={() => setShowConfig(false)} />}
            </AnimatePresence>
        </div>
    );
}
