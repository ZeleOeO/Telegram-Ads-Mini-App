import { useState, useEffect } from 'react';
import { X, DollarSign, BarChart2, ShieldCheck } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';
import type { Channel, AdFormat, BackendError } from '../types';

interface ChannelSettingsModalProps {
    channel: Channel;
    onClose: () => void;
    onSuccess: () => void;
}

export function ChannelSettingsModal({ channel, onClose, onSuccess }: ChannelSettingsModalProps) {
    const [activeTab, setActiveTab] = useState<'pricing' | 'stats' | 'team'>('pricing');
    const { tg } = useTelegram();

    // Stats state
    const [language, setLanguage] = useState(channel.language || 'EN');
    const [category, setCategory] = useState(channel.category || 'other');

    // Pricing state
    const [format, setFormat] = useState('Post');
    const [price, setPrice] = useState('');
    const [formats, setFormats] = useState<AdFormat[]>([]);

    // Team state
    const [managerInput, setManagerInput] = useState('');

    const [submitting, setSubmitting] = useState(false);

    useEffect(() => {
        loadFormats();
    }, [channel.id]);

    const loadFormats = async () => {
        try {
            const res = await api.get(`/channels/${channel.id}/ad-formats`);
            setFormats(res.data);
        } catch (e) {
            console.error('Failed to load formats');
        }
    };

    const handleUpdateStats = async (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        try {
            await api.put(`/channels/${channel.id}`, {
                language,
                category
            });
            onSuccess();
        } catch (e) {
            alert('Failed to update stats');
        } finally {
            setSubmitting(false);
        }
    };

    const handleAddPricing = async (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        try {
            await api.post(`/channels/${channel.id}/ad-formats`, {
                format_name: format,
                price_ton: parseFloat(price),
                format_description: `Standard ${format.toLowerCase()} ad placement`,
            });
            setPrice('');
            loadFormats();
            onSuccess();
        } catch (e) {
            alert('Failed to add pricing format');
        } finally {
            setSubmitting(false);
        }
    };

    const handleDeleteChannel = async () => {
        if (!confirm('Are you sure you want to delete this channel?')) return;
        setSubmitting(true);
        try {
            await api.delete(`/channels/${channel.id}`);
            tg?.HapticFeedback.notificationOccurred('success');
            onSuccess();
        } catch (e) {
            alert('Failed to delete channel');
            setSubmitting(false);
        }
    };

    const handleDeleteFormat = async (formatId: number) => {
        if (!confirm('Delete this pricing format?')) return;
        try {
            await api.delete(`/channels/${channel.id}/ad-formats/${formatId}`);
            tg?.HapticFeedback.notificationOccurred('success');
            loadFormats();
        } catch (e) {
            alert('Failed to delete format');
        }
    };

    const handleInviteManager = async () => {
        if (!managerInput) return;
        setSubmitting(true);
        try {
            await api.post(`/channels/${channel.id}/pr-managers`, {
                username_or_id: managerInput
            });
            setManagerInput('');
            alert('PR Manager invited! They will receive a notification.');
        } catch (err: unknown) {
            const e = err as BackendError;
            alert(e.response?.data?.error || 'Failed to invite manager');
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="fixed inset-0 z-[110] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full sm:max-w-md glass rounded-t-2xl sm:rounded-2xl relative max-h-[90vh] overflow-y-auto font-sans">
                <div className="p-4 border-b border-white/10 flex justify-between items-center sticky top-0 bg-[#0b0e14]/80 backdrop-blur-md z-10">
                    <h3 className="text-sm font-bold uppercase tracking-wider text-muted">Channel Settings</h3>
                    <button onClick={onClose} className="text-muted hover:text-white"><X size={20} /></button>
                </div>

                {/* Tabs */}
                <div className="flex p-2 gap-1">
                    <button
                        onClick={() => setActiveTab('pricing')}
                        className={`flex-1 py-2 text-xs font-bold rounded-lg transition-all ${activeTab === 'pricing' ? 'bg-white/10 text-white' : 'text-muted'}`}
                    >
                        <DollarSign size={14} className="inline mr-1" /> Pricing
                    </button>
                    <button
                        onClick={() => setActiveTab('stats')}
                        className={`flex-1 py-2 text-xs font-bold rounded-lg transition-all ${activeTab === 'stats' ? 'bg-white/10 text-white' : 'text-muted'}`}
                    >
                        <BarChart2 size={14} className="inline mr-1" /> General
                    </button>
                    <button
                        onClick={() => setActiveTab('team')}
                        className={`flex-1 py-2 text-xs font-bold rounded-lg transition-all ${activeTab === 'team' ? 'bg-white/10 text-white' : 'text-muted'}`}
                    >
                        <ShieldCheck size={14} className="inline mr-1" /> Team
                    </button>
                </div>

                <div className="p-6 pt-2 pb-24">
                    {activeTab === 'pricing' && (
                        <div className="space-y-6">
                            <div className="p-1 bg-white/5 rounded-2xl border border-white/5 overflow-hidden">
                                <div className="p-4 space-y-4">
                                    <div className="flex justify-between items-center">
                                        <h4 className="text-[10px] font-bold text-blue-400 uppercase tracking-widest">Add New Format</h4>
                                        <DollarSign size={14} className="text-blue-400/50" />
                                    </div>

                                    <div className="space-y-1.5">
                                        <label className="text-[10px] text-muted uppercase font-bold ml-1">Format Type</label>
                                        <select value={format} onChange={e => setFormat(e.target.value)} className="w-full h-11 glass rounded-xl px-3 text-xs focus:outline-none focus:border-blue-500/50 appearance-none">
                                            <option value="Post">Standard Post</option>
                                            <option value="Story">Story</option>
                                            <option value="Repost">Repost</option>
                                            <option value="Round">Round Video</option>
                                        </select>
                                        <p className="text-[10px] text-muted ml-1">All ads are permanent</p>
                                    </div>

                                    <div className="space-y-1.5">
                                        <label className="text-[10px] text-muted uppercase font-bold ml-1">Price (TON)</label>
                                        <div className="relative">
                                            <input
                                                type="number"
                                                step="0.1"
                                                value={price}
                                                onChange={e => setPrice(e.target.value)}
                                                placeholder="0.0"
                                                className="w-full h-12 glass border border-white/10 rounded-xl pl-4 pr-12 text-sm text-white focus:outline-none focus:border-blue-500/50 transition-all"
                                                required
                                            />
                                            <div className="absolute right-4 top-1/2 -translate-y-1/2 text-[10px] font-bold text-blue-400 uppercase">TON</div>
                                        </div>
                                    </div>

                                    <button
                                        onClick={handleAddPricing}
                                        disabled={submitting || !price}
                                        className="w-full h-12 bg-blue-500 hover:bg-blue-400 disabled:opacity-30 disabled:hover:bg-blue-500 text-black rounded-xl text-xs font-bold active:scale-[0.98] transition-all shadow-lg shadow-blue-500/20"
                                    >
                                        {submitting ? 'Adding...' : 'Verify & Add Format'}
                                    </button>
                                </div>
                            </div>

                            <div className="space-y-3">
                                <h4 className="text-[10px] font-bold text-muted uppercase tracking-widest ml-1">Active Ad Formats</h4>
                                <div className="space-y-2">
                                    {formats.map(f => (
                                        <div key={f.id} className="group p-4 bg-white/5 rounded-2xl border border-white/5 flex items-center justify-between hover:border-white/10 transition-all">
                                            <div className="space-y-1">
                                                <div className="text-xs font-bold">{f.format_name}</div>
                                                <div className="text-[10px] text-muted uppercase font-medium">Standard Placement</div>
                                            </div>
                                            <div className="flex items-center gap-3">
                                                <div className="text-sm font-bold text-blue-400 bg-blue-400/10 px-3 py-1.5 rounded-lg border border-blue-400/20">
                                                    {Number(f.price_ton || 0).toFixed(3)} TON
                                                </div>
                                                <button
                                                    onClick={() => handleDeleteFormat(f.id)}
                                                    className="w-8 h-8 flex items-center justify-center bg-red-500/10 text-red-400 rounded-lg hover:bg-red-500/20 active:scale-95 transition-all opacity-0 group-hover:opacity-100"
                                                >
                                                    <X size={14} />
                                                </button>
                                            </div>
                                        </div>
                                    ))}
                                    {formats.length === 0 && (
                                        <div className="text-center py-10 bg-white/5 rounded-2xl border border-dashed border-white/10">
                                            <p className="text-xs text-muted italic">No formats listed yet</p>
                                        </div>
                                    )}
                                </div>
                            </div>
                        </div>
                    )}

                    {activeTab === 'stats' && (
                        <div className="space-y-6">
                            <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-2xl space-y-2">
                                <h4 className="text-xs font-bold text-blue-400 uppercase tracking-widest flex items-center gap-2">
                                    <ShieldCheck size={14} /> Official Analytics
                                </h4>
                                <p className="text-[10px] text-muted leading-relaxed">
                                    Reach and premium subscriber data are fetched automatically from Telegram records to ensure transparency and trust for advertisers.
                                </p>
                            </div>

                            <form onSubmit={handleUpdateStats} className="space-y-5">
                                <div className="space-y-1.5">
                                    <label className="text-[10px] text-muted uppercase font-bold ml-1">Channel Category</label>
                                    <select
                                        value={category}
                                        onChange={e => setCategory(e.target.value)}
                                        className="w-full h-12 bg-white/5 border border-white/10 rounded-xl px-4 text-sm text-white focus:border-blue-400 outline-none appearance-none"
                                    >
                                        <option value="crypto">Crypto</option>
                                        <option value="forex">Forex</option>
                                        <option value="tech">Technology</option>
                                        <option value="news">News</option>
                                        <option value="health">Health & Fitness</option>
                                        <option value="sports">Sports</option>
                                        <option value="business">Business & Finance</option>
                                        <option value="lifestyle">Lifestyle</option>
                                        <option value="entertainment">Entertainment</option>
                                        <option value="education">Education</option>
                                        <option value="other">Other</option>
                                    </select>
                                </div>
                                <div className="space-y-1.5">
                                    <label className="text-[10px] text-muted uppercase font-bold ml-1">Primary Language</label>
                                    <select
                                        value={language}
                                        onChange={e => setLanguage(e.target.value)}
                                        className="w-full h-12 bg-white/5 border border-white/10 rounded-xl px-4 text-sm text-white focus:border-blue-400 outline-none appearance-none"
                                    >
                                        <option value="EN">English</option>
                                        <option value="RU">Russian</option>
                                        <option value="UZ">Uzbek</option>
                                        <option value="ES">Spanish</option>
                                        <option value="DE">German</option>
                                    </select>
                                </div>
                                <button type="submit" disabled={submitting} className="w-full h-12 bg-white/10 hover:bg-white/20 text-white rounded-xl text-xs font-bold active:scale-[0.98] transition-all border border-white/10">
                                    {submitting ? 'Saving...' : 'Update Settings'}
                                </button>
                            </form>

                            <div className="grid grid-cols-2 gap-3">
                                <div className="p-4 bg-white/5 rounded-2xl border border-white/5">
                                    <div className="text-[10px] text-muted uppercase font-bold mb-1">Current Reach</div>
                                    <div className="text-lg font-bold">{Number(channel.reach || 0).toLocaleString()}</div>
                                </div>
                                <div className="p-4 bg-white/5 rounded-2xl border border-white/5">
                                    <div className="text-[10px] text-muted uppercase font-bold mb-1">Premium %</div>
                                    <div className="text-lg font-bold text-blue-400">{Number(channel.premium_percentage || 0).toFixed(1)}%</div>
                                </div>
                            </div>

                            <div className="pt-4 border-t border-white/5">
                                <button
                                    onClick={handleDeleteChannel}
                                    disabled={submitting}
                                    className="w-full h-12 bg-red-500/10 border border-red-500/20 text-red-400 hover:bg-red-500/20 rounded-xl text-xs font-bold active:scale-[0.98] transition-all flex items-center justify-center gap-2"
                                >
                                    <X size={14} /> Delete Channel
                                </button>
                                <p className="text-[10px] text-center text-muted mt-2">
                                    Warning: Deleting a channel will remove all pricing settings. Active deals will remain.
                                </p>
                            </div>
                        </div>
                    )}

                    {activeTab === 'team' && (
                        <div className="space-y-6">
                            <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-2xl space-y-2">
                                <h4 className="text-xs font-bold text-blue-400 uppercase tracking-widest">PR Managers</h4>
                                <p className="text-[10px] text-muted leading-relaxed">
                                    Managers can view ad requests, analytics, and manage bookings for this channel. They must have started our bot at least once.
                                </p>
                            </div>

                            <div className="space-y-3">
                                <div className="space-y-1.5">
                                    <label className="text-[10px] text-muted uppercase font-bold ml-1">Invite by Username or ID</label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            value={managerInput}
                                            onChange={e => setManagerInput(e.target.value)}
                                            placeholder="@username or 123456"
                                            className="flex-1 h-11 bg-white/5 border border-white/10 rounded-xl px-4 text-sm focus:outline-none focus:border-blue-400"
                                        />
                                        <button
                                            onClick={handleInviteManager}
                                            disabled={submitting || !managerInput}
                                            className="h-11 px-4 bg-white/10 rounded-xl text-xs font-bold hover:bg-white/20 active:scale-95 transition-all disabled:opacity-30"
                                        >
                                            Invite
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
