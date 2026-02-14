import { Outlet } from 'react-router-dom';
import { BottomNav } from './BottomNav';
import { Header } from './Header';
import { useTelegram } from '../hooks/useTelegram';
import { useEffect } from 'react';
import { api, setAuthToken } from '../lib/api';

export function Layout() {
    const { initData, isReady } = useTelegram();

    // Sync auth token when initData is ready and call /me to persist user data
    useEffect(() => {
        if (initData) {
            setAuthToken(initData);
            // Call /me to ensure user data (username, names) is synced to backend
            api.get('/me').catch(err => console.warn('Failed to sync user data:', err));
        }
    }, [initData]);

    if (!isReady) {
        return (
            <div className="min-h-screen flex items-center justify-center bg-background text-foreground">
                Loading...
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-background text-foreground pt-16 font-sans">
            <div className="fixed inset-0 pointer-events-none z-0">
                <div className="absolute top-[-20%] left-[-10%] w-[50%] h-[50%] blur-[100px] rounded-full" style={{ backgroundColor: 'var(--accent-glow-1)' }} />
                <div className="absolute bottom-[-20%] right-[-10%] w-[50%] h-[50%] blur-[100px] rounded-full" style={{ backgroundColor: 'var(--accent-glow-2)' }} />
            </div>

            <Header />
            <main className="relative z-10 px-4 pb-40">
                <Outlet />
            </main>
            <BottomNav />
        </div>
    );
}
