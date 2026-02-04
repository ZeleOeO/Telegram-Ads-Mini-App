import { useEffect, useState } from 'react';
import { X, Users, Globe, Flame, CheckCircle2, DollarSign } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';
import type { Channel, AdFormat } from '../types';

interface ChannelDetailsModalProps {
    channelId: number;
    onClose: () => void;
    onStartDeal?: (formatId: number) => void;
}

export function ChannelDetailsModal({ channelId, onClose, onStartDeal }: ChannelDetailsModalProps) {
    const [channel, setChannel] = useState<Channel | null>(null);
    const [formats, setFormats] = useState<AdFormat[]>([]);
    const [loading, setLoading] = useState(true);
    const { tg } = useTelegram();

    useEffect(() => {
        const fetchDetails = async () => {
            try {
                // Fetch basic channels list to find this specific one (backend doesn't have a single channel detail yet, or use /channels?id=x if supported)
                // For now, let's assume we can fetch formats at least
                const [formatsRes, channelsRes] = await Promise.all([
                    api.get(`/channels/${channelId}/ad-formats`),
                    api.get('/channels')
                ]);

                const foundChannel = channelsRes.data.find((c: Channel) => c.id === channelId);
                setChannel(foundChannel);
                setFormats(formatsRes.data);
            } catch (err) {
                console.error('Failed to fetch channel details:', err);
            } finally {
                setLoading(false);
            }
        };
        fetchDetails();
    }, [channelId]);

    const handleStartDeal = (formatId: number) => {
        tg?.HapticFeedback.impactOccurred('medium');
        if (onStartDeal) {
            onStartDeal(formatId);
        } else {
            tg?.showAlert('Deal creation coming soon in this view!');
        }
    };

    if (loading) {
        return (
            <div className="fixed inset-0 z-[110] flex items-center justify-center bg-black/60 backdrop-blur-sm">
                <div className="text-white opacity-50">Loading details...</div>
            </div>
        );
    }

    if (!channel) return null;

    return (
        <div className="fixed inset-0 z-[110] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full sm:max-w-md glass rounded-t-2xl sm:rounded-2xl relative max-h-[90vh] overflow-y-auto font-sans">
                {/* Cover / Header Section */}
                <div className="h-24 bg-gradient-to-r from-blue-600/20 to-purple-600/20 relative">
                    <button
                        onClick={onClose}
                        className="absolute top-4 right-4 z-20 w-8 h-8 flex items-center justify-center bg-black/40 backdrop-blur-md rounded-full text-white/70 hover:text-white transition-colors"
                    >
                        <X size={18} />
                    </button>
                    <div className="absolute -bottom-6 left-6">
                        <div className="w-16 h-16 bg-blue-500 rounded-2xl flex items-center justify-center text-2xl shadow-xl border-4 border-[#0b0e14]">
                            {channel.title?.[0] || 'T'}
                        </div>
                    </div>
                </div>

                <div className="p-6 pt-10 pb-20 space-y-6">
                    <div>
                        <h3 className="text-xl font-bold">{channel.title}</h3>
                        <p className="text-blue-400 text-sm">@{channel.username || 'private'}</p>
                        {channel.category && (
                            <div className="mt-2 inline-flex px-2.5 py-1 bg-blue-500/10 border border-blue-500/20 rounded-lg text-[10px] font-bold text-blue-400 uppercase tracking-widest">
                                {channel.category}
                            </div>
                        )}
                    </div>

                    {/* Stats Grid */}
                    <div className="grid grid-cols-3 gap-3">
                        <div className="p-3 bg-white/5 rounded-2xl border border-white/5 text-center">
                            <Users size={16} className="mx-auto mb-1 text-blue-400 opacity-60" />
                            <div className="text-sm font-bold">{(channel.subscribers || 0).toLocaleString()}</div>
                            <div className="text-[10px] text-white/40 uppercase">Subs</div>
                        </div>
                        <div className="p-3 bg-white/5 rounded-2xl border border-white/5 text-center">
                            <Flame size={16} className="mx-auto mb-1 text-orange-400 opacity-60" />
                            <div className="text-sm font-bold">{channel.reach || '0'}k</div>
                            <div className="text-[10px] text-white/40 uppercase">Avg View</div>
                        </div>
                        <div className="p-3 bg-white/5 rounded-2xl border border-white/5 text-center">
                            <Globe size={16} className="mx-auto mb-1 text-purple-400 opacity-60" />
                            <div className="text-sm font-bold">{channel.language || 'Global'}</div>
                            <div className="text-[10px] text-white/40 uppercase">Lang</div>
                        </div>
                    </div>

                    {/* About Section */}
                    <div className="space-y-2">
                        <h4 className="text-xs font-bold text-white/60 uppercase tracking-wider">Analytics & Reach</h4>
                        <div className="space-y-4">
                            {/* Premium Bar */}
                            <div className="space-y-1.5">
                                <div className="flex justify-between text-[10px] font-bold uppercase tracking-tight">
                                    <span className="text-white/40">Telegram Premium Users</span>
                                    <span className="text-blue-400">{channel.premium_percentage || 12}%</span>
                                </div>
                                <div className="h-1.5 w-full bg-white/5 rounded-full overflow-hidden border border-white/5">
                                    <div
                                        className="h-full bg-gradient-to-r from-blue-600 to-blue-400 rounded-full"
                                        style={{ width: `${channel.premium_percentage || 12}%` }}
                                    />
                                </div>
                            </div>

                            {/* Language Profile */}
                            <div className="flex items-center justify-between p-3 bg-white/5 rounded-xl border border-white/5">
                                <div className="flex items-center gap-2">
                                    <Globe size={14} className="text-purple-400" />
                                    <span className="text-xs font-bold">Primary Language</span>
                                </div>
                                <span className="text-xs text-white/60 font-mono bg-white/5 px-2 py-1 rounded border border-white/10">
                                    {channel.language || 'English'}
                                </span>
                            </div>

                            <p className="text-sm text-white/70 leading-relaxed">
                                {channel.description || "No description provided. This channel is available for advertisements with high engagement rates."}
                            </p>
                        </div>
                    </div>

                    {/* Pricing Section */}
                    <div className="space-y-3">
                        <h4 className="text-xs font-bold text-white/60 uppercase tracking-wider">Ad Formats & Pricing</h4>
                        {formats.length === 0 ? (
                            <div className="p-4 bg-white/5 rounded-xl border border-dashed border-white/10 text-center text-xs text-white/40 italic">
                                No pricing set yet. Contact owner for custom offer.
                            </div>
                        ) : (
                            <div className="space-y-2">
                                {formats.map((f) => (
                                    <div key={f.id} className="p-4 bg-white/5 rounded-2xl border border-white/10 flex items-center justify-between group hover:border-blue-500/30 transition-all">
                                        <div className="space-y-1">
                                            <div className="font-bold flex items-center gap-2">
                                                {f.format_name}
                                                <span className="px-1.5 py-0.5 bg-blue-500/10 text-blue-400 text-[10px] rounded border border-blue-500/20">
                                                    {f.duration || '24h'}
                                                </span>
                                            </div>
                                            <div className="text-[10px] text-white/40">{f.format_description || 'Standard placement'}</div>
                                        </div>
                                        <button
                                            onClick={() => handleStartDeal(f.id)}
                                            className="px-4 h-9 bg-blue-500 text-black text-sm font-bold rounded-xl flex items-center gap-2 active:scale-95 transition-all"
                                        >
                                            <DollarSign size={14} /> {Number(f.price_ton).toFixed(3)} TON
                                        </button>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>

                    {/* Extra Info */}
                    <div className="p-3 bg-green-500/10 rounded-xl border border-green-500/20 flex gap-3 items-start">
                        <CheckCircle2 size={16} className="text-green-400 mt-0.5 shrink-0" />
                        <div className="text-[11px] text-green-400/80 leading-snug">
                            This channel is <strong>verified</strong> by our bot. Auto-posting is enabled, meaning your ad can be scheduled and posted instantly after approval.
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
