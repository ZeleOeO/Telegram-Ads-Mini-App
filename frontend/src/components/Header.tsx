import { Activity } from 'lucide-react';

export function Header() {
    return (
        <header className="fixed top-0 left-0 right-0 z-50 px-4 py-3 glass border-b border-white/5 flex justify-between items-center bg-background/80 backdrop-blur-md">
            <div className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400">
                    <Activity size={20} />
                </div>
                <div className="flex flex-col">
                    <span className="text-sm font-bold leading-none">TG Ads</span>
                    <span className="text-[10px] text-white/50">Marketplace</span>
                </div>
            </div>

            <button className="px-3 py-1.5 glass rounded-full text-xs font-medium text-white/80 flex items-center gap-1.5 active:scale-95 transition-transform">
                Connect Wallet
            </button>
        </header>
    );
}
