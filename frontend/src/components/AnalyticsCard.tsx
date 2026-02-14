import type { LucideIcon } from 'lucide-react';
import { cn } from '../lib/utils';

interface AnalyticsCardProps {
    title: string;
    value: string | number;
    trend?: string;
    trendColor?: 'green' | 'red' | 'neutral';
    icon?: LucideIcon;
    color: 'blue' | 'purple' | 'green' | 'orange';
    className?: string;
}

export function AnalyticsCard({
    title,
    value,
    trend,
    trendColor = 'neutral',
    icon: Icon,
    color,
    className
}: AnalyticsCardProps) {
    const colorStyles = {
        blue: 'from-blue-500/20 to-blue-600/5 border-blue-500/30 text-blue-400 shadow-blue-500/10',
        purple: 'from-purple-500/20 to-purple-600/5 border-purple-500/30 text-purple-400 shadow-purple-500/10',
        green: 'from-green-500/20 to-green-600/5 border-green-500/30 text-green-400 shadow-green-500/10',
        orange: 'from-orange-500/20 to-orange-600/5 border-orange-500/30 text-orange-400 shadow-orange-500/10',
    };

    const valueColorStyles = {
        blue: 'text-blue-100 drop-shadow-[0_0_8px_rgba(59,130,246,0.5)]',
        purple: 'text-purple-100 drop-shadow-[0_0_8px_rgba(168,85,247,0.5)]',
        green: 'text-green-100 drop-shadow-[0_0_8px_rgba(34,197,94,0.5)]',
        orange: 'text-orange-100 drop-shadow-[0_0_8px_rgba(249,115,22,0.5)]',
    };

    const trendColorStyles = {
        green: 'text-emerald-400',
        red: 'text-rose-400',
        neutral: 'text-muted-foreground',
    };

    return (
        <div className={cn(
            "relative overflow-hidden rounded-2xl p-4 border bg-gradient-to-br backdrop-blur-sm transition-all hover:scale-[1.02]",
            colorStyles[color],
            className
        )}>
            {/* Background Glow Effect */}
            <div className={cn(
                "absolute -right-4 -top-4 w-24 h-24 rounded-full blur-3xl opacity-20",
                color === 'blue' && "bg-blue-500",
                color === 'purple' && "bg-purple-500",
                color === 'green' && "bg-green-500",
                color === 'orange' && "bg-orange-500",
            )} />

            <div className="relative z-10 flex flex-col justify-between h-full min-h-[100px]">
                <div className="flex justify-between items-start mb-2">
                    <span className="text-xs font-medium text-white/60 uppercase tracking-wider">{title}</span>
                    {Icon && <Icon size={16} className="opacity-70" />}
                </div>

                <div className="space-y-1">
                    <div className={cn("text-2xl font-bold tracking-tight", valueColorStyles[color])}>
                        {value}
                    </div>
                    {trend && (
                        <div className={cn("text-[10px] font-medium flex items-center gap-1", trendColorStyles[trendColor])}>
                            {trend}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
