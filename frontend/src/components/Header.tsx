import { Menu, User, Bell } from 'lucide-react';
import { useState, useEffect } from 'react';
import { useTelegram } from '../hooks/useTelegram';

interface UserData {
    username?: string;
    first_name?: string;
    photo_url?: string;
}

export function Header() {
    const { user: tgUser } = useTelegram();
    const [userData, setUserData] = useState<UserData | null>(null);

    useEffect(() => {
        // Fetch additional user data if needed, or just use TG user
        if (tgUser) {
            setUserData({
                first_name: tgUser.first_name,
                username: tgUser.username,
                // photo_url would come from backend if we stored it, or TG if accessible
            });
        }
    }, [tgUser]);

    return (
        <header className="fixed top-0 left-0 right-0 z-50 px-6 py-4 flex justify-between items-center bg-background/80 backdrop-blur-md">
            {/* User Profile Section */}
            <div className="flex items-center gap-3">
                <div className="relative">
                    <div className="w-10 h-10 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white ring-2 ring-white/10 shadow-lg shadow-blue-500/20">
                        {userData?.photo_url ? (
                            <img src={userData.photo_url} alt="Profile" className="w-full h-full rounded-full object-cover" />
                        ) : (
                            <User size={20} />
                        )}
                    </div>
                    <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-green-500 rounded-full border-2 border-background" />
                </div>

                <div className="flex flex-col">
                    <span className="text-sm font-bold text-white leading-none">
                        {userData?.first_name || 'Guest'}
                    </span>
                    <span className="text-[10px] text-white/50 font-medium">
                        {userData?.username ? `@${userData.username}` : 'Welcome back'}
                    </span>
                </div>
            </div>

            {/* Actions */}
            <div className="flex items-center gap-3">
                <button className="w-10 h-10 rounded-xl glass flex items-center justify-center text-white/70 hover:text-white hover:bg-white/10 transition-colors relative">
                    <Bell size={20} />
                    <span className="absolute top-2.5 right-2.5 w-2 h-2 bg-red-500 rounded-full border border-background" />
                </button>
                <button className="w-10 h-10 rounded-xl glass flex items-center justify-center text-white/70 hover:text-white hover:bg-white/10 transition-colors">
                    <Menu size={20} />
                </button>
            </div>
        </header>
    );
}
