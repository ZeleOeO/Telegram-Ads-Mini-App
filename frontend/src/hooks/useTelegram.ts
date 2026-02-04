import { useEffect, useState } from 'react';
import WebApp from '@twa-dev/sdk';

export interface TelegramUser {
    id: number;
    first_name: string;
    last_name?: string;
    username?: string;
    language_code?: string;
    is_premium?: boolean;
}

export function useTelegram() {
    const [user, setUser] = useState<TelegramUser | null>(null);
    const [initData, setInitData] = useState<string>('');
    const [isReady, setIsReady] = useState(false);

    useEffect(() => {
        if (isReady) return;
        // 1. Try SDK First
        let data = WebApp.initData;
        const userData = WebApp.initDataUnsafe?.user;

        // 2. Fallback: Parse Hash manually if SDK failed
        if (!data && window.location.hash) {
            try {
                const hash = window.location.hash.substring(1);
                const params = new URLSearchParams(hash);
                if (params.has('tgWebAppData')) {
                    data = params.get('tgWebAppData') || '';
                    // Try to parse user from the hash string if possible, 
                    // but usually it's encoded. For MVP we just need the raw string for auth.
                    console.log('UseTelegram: using fallback hash data');
                }
            } catch (_error) {
                console.error('UseTelegram: Hash parse failed', _error);
            }
        }

        // Initialize
        if (WebApp) {
            WebApp.ready();
            WebApp.expand();
            try {
                WebApp.setHeaderColor('#0b0e14');
                WebApp.setBackgroundColor('#0b0e14');
            } catch (_error) {
                // Ignore theme errors
            }
        }

        if (data) {
            setInitData(data);
        }
        if (userData) {
            setUser(userData as TelegramUser);
        }
        setIsReady(true);

    }, []);

    const onClose = () => {
        WebApp.close();
    };

    return {
        onClose,
        tg: WebApp,
        user,
        initData,
        isReady
    };
}
