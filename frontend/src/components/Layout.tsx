import { Outlet } from 'react-router-dom';
import { BottomNav } from './BottomNav';
import { Header } from './Header';
import { useTelegram } from '../hooks/useTelegram';
import { useEffect } from 'react';
import { setAuthToken } from '../lib/api';

export function Layout() {
    const { initData, isReady } = useTelegram();

    // Sync auth token when initData is ready
    useEffect(() => {
        if (initData) {
            setAuthToken(initData);
        }
    }, [initData]);

    if (!isReady) {
        return (
            <div className="min-h-screen flex items-center justify-center bg-background text-white/50">
                Loading...
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-background text-white pt-16 font-sans">
            <div className="fixed inset-0 pointer-events-none z-0">
                <div className="absolute top-[-20%] left-[-10%] w-[50%] h-[50%] bg-blue-500/10 blur-[100px] rounded-full" />
                <div className="absolute bottom-[-20%] right-[-10%] w-[50%] h-[50%] bg-purple-500/10 blur-[100px] rounded-full" />
            </div>

            <Header />
            <main className="relative z-10 px-4 pb-40">
                <Outlet />
            </main>
            <BottomNav />
        </div>
    );
}
