import { useEffect, useState } from 'react';
import { Plus, X, Settings, CheckCircle2 } from 'lucide-react';
import { api } from '../lib/api';
import type { Channel } from '../types';
import { formatNumber } from '../lib/utils';
import { useTelegram } from '../hooks/useTelegram';
import { ChannelSettingsModal } from '../components/ChannelSettingsModal';
import type { ChannelSuggestion, BackendError } from '../types';

export function MyChannels() {
    const [channels, setChannels] = useState<Channel[]>([]);
    const [loading, setLoading] = useState(true);
    const [showModal, setShowModal] = useState(false);
    const [username, setUsername] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState('');
    const [suggestions, setSuggestions] = useState<ChannelSuggestion[]>([]);
    const [loadingSuggestions, setLoadingSuggestions] = useState(false);

    // Channel Settings Modal State
    const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null);

    const { tg, user } = useTelegram();

    useEffect(() => {
        if (user) loadMyChannels();
    }, [user]);

    useEffect(() => {
        if (showModal && user) {
            loadSuggestions();
        }
    }, [showModal, user]);

    const loadSuggestions = async () => {
        setLoadingSuggestions(true);
        try {
            const res = await api.get('/channels/bot-admin');
            setSuggestions(res.data);
        } catch (e) {
            console.error('Failed to load suggestions:', e);
        } finally {
            setLoadingSuggestions(false);
        }
    };

    const loadMyChannels = async () => {
        try {
            const res = await api.get('/channels/my');
            setChannels(res.data);
        } catch (err: unknown) {
            const e = err as BackendError;
            setError(e.response?.data?.error || e.message || 'Error');
        } finally {
            setLoading(false);
        }
    };

    const handleAddChannel = async (e?: React.FormEvent, customUsername?: string) => {
        if (e) e.preventDefault();
        const targetUsername = customUsername || username;
        if (!targetUsername) return;

        setSubmitting(true);
        try {
            await api.post('/channels', { username: targetUsername });
            tg?.HapticFeedback.notificationOccurred('success');
            setShowModal(false);
            setUsername('');
            loadMyChannels();
        } catch (err: unknown) {
            tg?.HapticFeedback.notificationOccurred('error');
            const e = err as BackendError;
            alert(e.response?.data?.error || 'Failed to add channel. Ensure bot is admin!');
        } finally {
            setSubmitting(false);
        }
    };

    if (loading) return <div className="text-center mt-10 opacity-50">Loading...</div>;

    return (
        <div className="space-y-6 pb-24">
            <div className="flex justify-between items-center py-4">
                <h2 className="text-xl font-bold">My Channels</h2>
                <button
                    onClick={() => setShowModal(true)}
                    className="flex items-center gap-2 px-3 py-2 bg-blue-500 rounded-lg text-white text-xs font-bold active:scale-95 transition-transform"
                >
                    <Plus size={16} /> Add Channel
                </button>
            </div>

            {error && (
                <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-400 text-sm">
                    {error}
                </div>
            )}

            <div className="space-y-3 pb-8">
                {channels.map(channel => (
                    <div key={channel.id} className="glass-card border-l-4 border-l-blue-500">
                        <div className="flex justify-between items-start mb-2">
                            <span className="font-semibold">{channel.title || channel.username}</span>
                            <span className={`text-[10px] px-2 py-0.5 rounded-full uppercase font-bold tracking-wider ${channel.status === 'active' ? 'bg-green-500/20 text-green-400' : 'bg-yellow-500/20 text-yellow-400'
                                }`}>
                                {channel.status}
                            </span>
                        </div>
                        <div className="flex gap-4 text-xs text-white/50 mb-3">
                            <span>{formatNumber(channel.subscribers)} subs</span>
                            <span>{formatNumber(channel.reach || 0)} reach</span>
                        </div>
                        <button
                            onClick={() => setSelectedChannel(channel)}
                            className="w-full py-2 bg-white/5 rounded-lg text-xs font-medium hover:bg-white/10 transition-colors flex items-center justify-center gap-2"
                        >
                            <Settings size={14} /> Channel Settings
                        </button>
                    </div>
                ))}

                {channels.length === 0 && !loading && !error && (
                    <div className="text-center py-10 opacity-40 flex flex-col items-center gap-2">
                        <p>No channels listed yet</p>
                        <button onClick={() => setShowModal(true)} className="text-blue-400 text-sm">
                            List your first channel
                        </button>
                    </div>
                )}
            </div>

            {/* Add Channel Modal */}
            {showModal && (
                <div className="fixed inset-0 z-[100] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="w-full sm:max-w-sm glass rounded-t-2xl sm:rounded-2xl p-6 relative max-h-[90vh] overflow-y-auto font-sans">
                        <div className="pb-10">
                            <button
                                onClick={() => setShowModal(false)}
                                className="absolute top-4 right-4 text-white/40 hover:text-white"
                            >
                                <X size={20} />
                            </button>

                            <h3 className="text-lg font-bold mb-4">📢 Add Channel</h3>

                            <div className="space-y-4">
                                {loadingSuggestions ? (
                                    <div className="text-center py-4 opacity-30 animate-pulse text-xs">Searching for connected channels...</div>
                                ) : suggestions.length > 0 && (
                                    <div className="space-y-2">
                                        <p className="text-[10px] text-white/40 uppercase font-bold ml-1 tracking-wider">Suggested Channels</p>
                                        <div className="space-y-2">
                                            {suggestions.map(s => (
                                                <button
                                                    key={s.telegram_id}
                                                    onClick={() => handleAddChannel(undefined, s.telegram_id.toString())}
                                                    className="w-full p-3 bg-white/5 border border-white/10 rounded-xl flex items-center justify-between group active:scale-[0.98] transition-all"
                                                >
                                                    <div className="flex items-center gap-3">
                                                        <div className="w-8 h-8 rounded bg-blue-500/20 text-blue-400 flex items-center justify-center text-xs font-bold">
                                                            {s.title?.[0] || 'C'}
                                                        </div>
                                                        <div className="text-left">
                                                            <p className="text-xs font-bold text-white/90">{s.title}</p>
                                                            <p className="text-[10px] text-white/30">@{s.username || 'private'}</p>
                                                        </div>
                                                    </div>
                                                    <Plus size={14} className="text-blue-500" />
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                <div className="p-3 bg-blue-500/10 rounded-xl border border-blue-500/20 space-y-2">
                                    <h4 className="text-xs font-bold text-blue-400 flex items-center gap-2">
                                        <CheckCircle2 size={14} /> Requirements
                                    </h4>
                                    <ol className="text-[10px] text-white/70 list-decimal pl-4 space-y-1">
                                        <li>Add <strong>@ad_market_place_bot</strong> as Admin to your channel</li>
                                        <li>Ensure "Post Messages" permission is enabled</li>
                                        <li>Enter the link or username below</li>
                                    </ol>
                                </div>

                                <form onSubmit={handleAddChannel} className="space-y-4">
                                    <div className="space-y-1.5">
                                        <label className="text-xs text-white/60 ml-1">Channel Username / Link</label>
                                        <input
                                            type="text"
                                            value={username}
                                            onChange={e => setUsername(e.target.value)}
                                            placeholder="t.me/yourchannel"
                                            className="w-full h-10 px-3 bg-black/20 border border-white/10 rounded-xl text-sm focus:outline-none focus:border-blue-500/50"
                                            required
                                        />
                                    </div>

                                    <button
                                        type="submit"
                                        disabled={submitting}
                                        className="w-full h-11 bg-blue-500 hover:bg-blue-600 rounded-xl font-medium active:scale-95 transition-all disabled:opacity-50"
                                    >
                                        {submitting ? 'Verifying...' : 'Verify & Add'}
                                    </button>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {/* Channel Settings Modal */}
            {selectedChannel && (
                <ChannelSettingsModal
                    channel={selectedChannel}
                    onClose={() => setSelectedChannel(null)}
                    onSuccess={() => {
                        tg?.HapticFeedback.notificationOccurred('success');
                        loadMyChannels();
                    }}
                />
            )}
        </div>
    );
}
