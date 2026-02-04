import { useEffect, useState } from 'react';
import { Plus, X, Image as ImageIcon, Trash2, Users } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';
import { CampaignApplicationsModal } from '../components/CampaignApplicationsModal';
import type { Campaign, BackendError } from '../types';

export function Campaigns() {
    const [campaigns, setCampaigns] = useState<Campaign[]>([]);
    const [loading, setLoading] = useState(false);
    const [showModal, setShowModal] = useState(false);
    const [selectedCampaignId, setSelectedCampaignId] = useState<number | null>(null);
    const [selectedCampaignTitle, setSelectedCampaignTitle] = useState('');

    // Form State
    const [title, setTitle] = useState('');
    const [brief, setBrief] = useState('');
    const [budget, setBudget] = useState('');
    const [minSubs, setMinSubs] = useState('');
    const [mediaUrls, setMediaUrls] = useState<string[]>(['']);
    const [submitting, setSubmitting] = useState(false);

    const { tg } = useTelegram();

    useEffect(() => {
        loadMyCampaigns();
    }, []);

    const loadMyCampaigns = async () => {
        setLoading(true);
        try {
            const res = await api.get('/campaigns/my');
            setCampaigns(res.data);
        } catch (e) {
            console.error('Failed to load campaigns', e);
        } finally {
            setLoading(false);
        }
    };

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        try {
            const validUrls = mediaUrls.filter((u: string) => u.trim().length > 0);
            await api.post('/campaigns', {
                title,
                brief,
                budget_ton: parseFloat(budget),
                target_subscribers_min: minSubs ? parseInt(minSubs) : null,
                media_urls: validUrls.length > 0 ? validUrls : null
            });
            tg?.HapticFeedback.notificationOccurred('success');
            setShowModal(false);
            loadMyCampaigns();
            // Reset form
            setTitle('');
            setBrief('');
            setBudget('');
            setMinSubs('');
            setMediaUrls(['']);
        } catch (err: unknown) {
            tg?.HapticFeedback.notificationOccurred('error');
            const e = err as BackendError;
            alert(e.response?.data?.error || 'Failed to create campaign');
        } finally {
            setSubmitting(false);
        }
    };

    const updateMediaUrl = (index: number, value: string) => {
        const newUrls = [...mediaUrls];
        newUrls[index] = value;
        setMediaUrls(newUrls);
    };

    const addMediaField = () => setMediaUrls([...mediaUrls, '']);
    const removeMediaField = (index: number) => {
        if (mediaUrls.length > 1) {
            setMediaUrls(mediaUrls.filter((_: string, i: number) => i !== index));
        }
    };

    return (
        <div className="space-y-6 pb-24">
            <div className="flex justify-between items-center py-4">
                <div>
                    <h2 className="text-xl font-bold">My Campaigns</h2>
                    <p className="text-xs text-white/50">Manage your active ad requests</p>
                </div>
                <button
                    onClick={() => setShowModal(true)}
                    className="flex items-center gap-2 px-4 py-2 bg-blue-500 rounded-xl text-white font-bold text-xs active:scale-95 transition-transform shadow-lg shadow-blue-500/20"
                >
                    <Plus size={16} /> New Campaign
                </button>
            </div>

            {loading ? (
                <div className="text-center mt-10 opacity-50">Loading...</div>
            ) : (
                <div className="space-y-4 pb-8">
                    {campaigns.map((c: Campaign) => (
                        <div key={c.id} className="glass-card">
                            <div className="flex justify-between items-start mb-2">
                                <span className="font-semibold text-lg">{c.title}</span>
                                <span className="px-2 py-1 bg-green-500/10 text-green-400 rounded-lg font-mono text-xs">
                                    {Number(c.budget_ton).toFixed(3)} TON
                                </span>
                            </div>
                            <p className="text-sm text-white/60 line-clamp-3 mb-3">
                                {c.brief}
                            </p>

                            {/* Media Preview */}
                            {c.media_urls && (
                                <div className="flex gap-2 overflow-x-auto pb-2 mb-2">
                                    {(c.media_urls as unknown as string).split(',').slice(0, 3).map((url, i) => (
                                        <img key={i} src={url.trim()} className="w-12 h-12 rounded-lg object-cover bg-white/5" alt="" />
                                    ))}
                                </div>
                            )}

                            <div className="flex justify-between items-center pt-3 border-t border-white/5">
                                <button
                                    onClick={() => {
                                        setSelectedCampaignId(c.id);
                                        setSelectedCampaignTitle(c.title);
                                    }}
                                    className="flex items-center gap-2 px-3 py-1.5 bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg text-[10px] font-bold transition-all"
                                >
                                    <Users size={14} className="text-blue-400" /> View Applications
                                </button>
                                <span className="px-2 py-0.5 bg-yellow-500/10 text-yellow-400 rounded text-[10px] uppercase font-bold tracking-wider">
                                    Active
                                </span>
                            </div>
                        </div>
                    ))}

                    {campaigns.length === 0 && (
                        <div className="text-center py-12 opacity-40 flex flex-col items-center gap-3">
                            <Megaphone size={40} className="opacity-50" />
                            <p>You haven't created any campaigns yet.</p>
                            <button onClick={() => setShowModal(true)} className="text-blue-400 text-sm font-bold">
                                Create your first one
                            </button>
                        </div>
                    )}
                </div>
            )}

            {showModal && (
                <div className="fixed inset-0 z-[100] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="w-full sm:max-w-md glass rounded-t-2xl sm:rounded-2xl p-6 relative max-h-[90vh] overflow-y-auto">
                        <div className="pb-20">
                            <button
                                onClick={() => setShowModal(false)}
                                className="absolute top-4 right-4 text-white/40 hover:text-white"
                            >
                                <X size={20} />
                            </button>

                            <h3 className="text-lg font-bold mb-6">📣 Create Campaign</h3>

                            <form onSubmit={handleCreate} className="space-y-4">
                                <input
                                    type="text"
                                    value={title}
                                    onChange={e => setTitle(e.target.value)}
                                    placeholder="Campaign Title"
                                    className="w-full h-12 px-4 bg-black/20 border border-white/10 rounded-xl text-sm focus:outline-none focus:border-blue-500/50"
                                    required
                                />

                                <textarea
                                    value={brief}
                                    onChange={e => setBrief(e.target.value)}
                                    placeholder="Place ad here..."
                                    className="w-full h-24 px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-sm focus:outline-none focus:border-blue-500/50 resize-none"
                                    required
                                />

                                <div className="grid grid-cols-2 gap-3">
                                    <div className="space-y-1">
                                        <label className="text-[10px] uppercase text-white/40 ml-1">Budget (TON)</label>
                                        <input
                                            type="number"
                                            step="0.1"
                                            value={budget}
                                            onChange={e => setBudget(e.target.value)}
                                            placeholder="100"
                                            className="w-full h-10 px-3 bg-black/20 border border-white/10 rounded-xl text-sm"
                                            required
                                        />
                                    </div>
                                    <div className="space-y-1">
                                        <label className="text-[10px] uppercase text-white/40 ml-1">Min Subs</label>
                                        <input
                                            type="number"
                                            value={minSubs}
                                            onChange={e => setMinSubs(e.target.value)}
                                            placeholder="1000"
                                            className="w-full h-10 px-3 bg-black/20 border border-white/10 rounded-xl text-sm"
                                        />
                                    </div>
                                </div>

                                <div className="space-y-2 pt-2">
                                    <label className="text-xs text-white/60 ml-1 flex items-center gap-2">
                                        <ImageIcon size={14} /> Creative Assets (URLs)
                                    </label>
                                    {mediaUrls.map((url: string, idx: number) => (
                                        <div key={idx} className="flex gap-2">
                                            <input
                                                type="url"
                                                value={url}
                                                onChange={e => updateMediaUrl(idx, e.target.value)}
                                                placeholder="https://imgur.com/..."
                                                className="flex-1 h-10 px-3 bg-black/20 border border-white/10 rounded-xl text-xs font-mono"
                                            />
                                            {mediaUrls.length > 1 && (
                                                <button
                                                    type="button"
                                                    onClick={() => removeMediaField(idx)}
                                                    className="w-10 h-10 flex items-center justify-center bg-red-500/10 text-red-400 rounded-xl hover:bg-red-500/20"
                                                >
                                                    <Trash2 size={16} />
                                                </button>
                                            )}
                                        </div>
                                    ))}
                                    <button
                                        type="button"
                                        onClick={addMediaField}
                                        className="text-xs text-blue-400 hover:text-blue-300 ml-1"
                                    >
                                        + Add another URL
                                    </button>
                                </div>

                                <button
                                    type="submit"
                                    disabled={submitting}
                                    className="w-full h-12 mt-2 bg-blue-500 hover:bg-blue-600 rounded-xl font-bold active:scale-95 transition-all disabled:opacity-50"
                                >
                                    {submitting ? 'Creating...' : 'Launch Campaign'}
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            )}

            {selectedCampaignId && (
                <CampaignApplicationsModal
                    campaignId={selectedCampaignId}
                    campaignTitle={selectedCampaignTitle}
                    onClose={() => setSelectedCampaignId(null)}
                />
            )}
        </div>
    );
}

function Megaphone({ size, className }: { size: number, className?: string }) {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width={size}
            height={size}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className={className}
        >
            <path d="m3 11 18-5v12L3 14v-3z" />
            <path d="M11.6 16.8a3 3 0 1 1-5.8-1.6" />
        </svg>
    );
}
