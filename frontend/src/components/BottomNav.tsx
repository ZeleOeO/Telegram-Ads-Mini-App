import { Link, useLocation } from 'react-router-dom';
import { Compass, MessageSquare, Megaphone, LayoutList, Home } from 'lucide-react';
import { cn } from '../lib/utils';
import { useTelegram } from '../hooks/useTelegram';

export function BottomNav() {
    const location = useLocation();
    const { tg } = useTelegram();

    const tabs = [
        { id: 'home', label: 'Home', icon: Home, path: '/' },
        { id: 'explorer', label: 'Explore', icon: Compass, path: '/explorer' },
        { id: 'campaigns', label: 'Campaigns', icon: Megaphone, path: '/campaigns' },
        { id: 'channels', label: 'Channels', icon: LayoutList, path: '/channels' },
        { id: 'deals', label: 'Deals', icon: MessageSquare, path: '/deals' },
    ];

    const haptic = () => {
        if (tg?.HapticFeedback) {
            tg.HapticFeedback.impactOccurred('light');
        }
    };

    return (
        <nav className="fixed bottom-4 left-4 right-4 h-[64px] glass rounded-2xl flex justify-around items-center z-30 shadow-lg border border-white/10">
            {tabs.map((tab) => {
                const isActive = location.pathname === tab.path;
                const Icon = tab.icon;

                return (
                    <Link
                        key={tab.id}
                        to={tab.path}
                        onClick={haptic}
                        className={cn(
                            "flex flex-col items-center justify-center w-full h-full transition-colors duration-200",
                            isActive ? "text-blue-400" : "text-white/40 hover:text-white/60"
                        )}
                    >
                        <Icon size={24} strokeWidth={isActive ? 2.5 : 2} />
                        <span className="text-[10px] mt-1 font-medium">{tab.label}</span>
                    </Link>
                );
            })}
        </nav>
    );
}
