import { Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { ArrowLeft, RefreshCw, Eye, EyeOff, Settings, ScrollText } from "lucide-solid";

interface SidebarProps {
    onBack: () => void;
    onRefresh: () => void;
    showSpoilers: boolean;
    onToggleSpoilers: () => void;
}

export function Sidebar(props: SidebarProps) {
    const navigate = useNavigate();

    return (
        <aside class="w-[80px] flex flex-col items-center py-6 bg-[var(--color-bg-primary)] border-r border-[var(--color-border)] shrink-0 gap-6 z-20">
            <button
                onClick={props.onBack}
                class="p-3 rounded-xl bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] text-[var(--color-icon)] hover:text-[var(--color-text-primary)] transition-colors shadow-sm"
                title="Back"
            >
                <ArrowLeft class="w-6 h-6" />
            </button>

            <div class="h-px w-10 bg-[var(--color-border)] my-2"></div>

            <button
                onClick={props.onRefresh}
                class="p-3 rounded-xl hover:bg-[var(--color-bg-secondary)] text-[var(--color-icon)] hover:text-[var(--color-text-primary)] transition-colors"
                title="Refresh Data"
            >
                <RefreshCw class="w-6 h-6" />
            </button>

            <button
                onClick={props.onToggleSpoilers}
                class={`p-3 rounded-xl hover:bg-[var(--color-bg-secondary)] transition-colors relative ${props.showSpoilers ? "text-[var(--color-accent)]" : "text-[var(--color-icon)] hover:text-[var(--color-text-primary)]"}`}
                title="Toggle Spoilers"
            >
                <Show when={props.showSpoilers} fallback={<EyeOff class="w-6 h-6" />}>
                    <Eye class="w-6 h-6" />
                </Show>
            </button>

            <div class="flex-1"></div>

            <button
                onClick={() => navigate("/logs")}
                class="p-3 rounded-xl hover:bg-[var(--color-bg-secondary)] text-[var(--color-icon)] hover:text-[var(--color-text-primary)] transition-colors"
                title="Logs"
            >
                <ScrollText class="w-6 h-6" />
            </button>

            <button
                onClick={() => navigate("/settings")}
                class="p-3 rounded-xl hover:bg-[var(--color-bg-secondary)] text-[var(--color-icon)] hover:text-[var(--color-text-primary)] transition-colors"
                title="Settings"
            >
                <Settings class="w-6 h-6" />
            </button>
        </aside>
    );
}
