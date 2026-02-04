import { useNavigate } from 'react-router-dom';
import { Telescope, Megaphone, Radio } from 'lucide-react';
import { useTelegram } from '../hooks/useTelegram';

export function Home() {
    const navigate = useNavigate();
    const { user } = useTelegram();

    return (
        <div className="flex flex-col min-h-[80vh] pt-10 px-4">
            <div className="flex-1 space-y-8">
                {/* Hero Section */}
                <div className="space-y-2 text-center">
                    <h1 className="text-3xl font-bold bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
                        Hello, {user?.first_name || 'Guest'}
                    </h1>
                    <p className="text-white/60">
                        Welcome to the premier marketplace for Telegram ads. Connect, deal, and grow.
                    </p>
                </div>

                {/* Quick Actions Grid */}
                <div className="grid grid-cols-1 gap-4">
                    <button
                        onClick={() => navigate('/explorer')}
                        className="group relative p-6 glass-card hover:bg-white/10 transition-all text-left overflow-hidden"
                    >
                        <div className="absolute right-0 top-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
                            <Telescope size={80} />
                        </div>
                        <h3 className="text-lg font-bold flex items-center gap-2 mb-1">
                            <Telescope size={20} className="text-blue-400" />
                            Explore
                        </h3>
                        <p className="text-sm text-white/50 max-w-[80%]">
                            Find channels or campaigns to partner with.
                        </p>
                    </button>

                    <div className="grid grid-cols-2 gap-4">
                        <button
                            onClick={() => navigate('/campaigns')}
                            className="p-4 glass-card hover:bg-white/10 transition-all text-left flex flex-col justify-between h-32"
                        >
                            <Megaphone size={24} className="text-purple-400 mb-2" />
                            <div>
                                <h3 className="font-bold text-sm">My Campaigns</h3>
                                <p className="text-[10px] text-white/50">Manage ads</p>
                            </div>
                        </button>

                        <button
                            onClick={() => navigate('/channels')}
                            className="p-4 glass-card hover:bg-white/10 transition-all text-left flex flex-col justify-between h-32"
                        >
                            <Radio size={24} className="text-green-400 mb-2" />
                            <div>
                                <h3 className="font-bold text-sm">My Channels</h3>
                                <p className="text-[10px] text-white/50">Monetize</p>
                            </div>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
