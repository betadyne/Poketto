import { Show, For, createSignal, createMemo, onMount, createEffect } from "solid-js";
import {
  Library as LibraryIcon,
  Settings as SettingsIcon,
  ScrollText,
  LogOut,
  RefreshCw,
  FolderOpen,
  ArrowDownToLine,
  Copy,
  Search,
  X,
  FileText,
} from "lucide-solid";
import { useNavigate } from "@solidjs/router";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

declare const __APP_VERSION__: string;

type LogLevel = "INFO" | "WARN" | "ERROR" | "DEBUG" | "TRACE";
type LevelFilter = "ALL" | LogLevel;

interface ParsedLine {
  date: string;
  time: string;
  target: string;
  level: LogLevel;
  message: string;
  raw: string;
}

function parseLine(raw: string): ParsedLine | null {
  const match = raw.match(
    /^\[(\d{4}-\d{2}-\d{2})\]\[(\d{2}:\d{2}:\d{2})\]\[([^\]]+)\]\[(\w+)\]\s(.*)$/
  );
  if (!match) return null;
  return {
    date: match[1],
    time: match[2],
    target: match[3],
    level: match[4] as LogLevel,
    message: match[5],
    raw,
  };
}

function levelColor(level: LogLevel): string {
  switch (level) {
    case "ERROR":
      return "text-[var(--color-danger)]";
    case "WARN":
      return "text-[var(--color-warning)]";
    case "INFO":
      return "text-[var(--color-text-primary)]";
    case "DEBUG":
      return "text-[var(--color-text-secondary)]";
    case "TRACE":
      return "text-[var(--color-text-tertiary)]";
    default:
      return "text-[var(--color-text-primary)]";
  }
}

function levelBadgeBg(level: LogLevel): string {
  switch (level) {
    case "ERROR":
      return "bg-[var(--color-danger-light)] text-[var(--color-danger)]";
    case "WARN":
      return "bg-[var(--color-warning-light)] text-[var(--color-warning)]";
    case "INFO":
      return "bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)]";
    case "DEBUG":
      return "bg-[var(--color-bg-secondary)] text-[var(--color-text-tertiary)]";
    case "TRACE":
      return "bg-[var(--color-bg-secondary)] text-[var(--color-text-tertiary)]";
    default:
      return "bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)]";
  }
}

export function Logs() {
  const navigate = useNavigate();

  const [lines, setLines] = createSignal<string[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [logPath, setLogPath] = createSignal<string>("");
  const [levelFilter, setLevelFilter] = createSignal<LevelFilter>("ALL");
  const [searchQuery, setSearchQuery] = createSignal("");
  const [autoScroll, setAutoScroll] = createSignal(true);
  const [copied, setCopied] = createSignal(false);

  let logContainerRef: HTMLDivElement | undefined;

  const parsedLines = createMemo(() => {
    const result: ParsedLine[] = [];
    for (const raw of lines()) {
      const parsed = parseLine(raw);
      if (parsed) result.push(parsed);
    }
    return result;
  });

  const filteredLines = createMemo(() => {
    let result = parsedLines();

    const filter = levelFilter();
    if (filter !== "ALL") {
      result = result.filter((l) => l.level === filter);
    }

    const query = searchQuery().toLowerCase();
    if (query) {
      result = result.filter(
        (l) =>
          l.message.toLowerCase().includes(query) ||
          l.target.toLowerCase().includes(query)
      );
    }

    return result;
  });

  const levelCounts = createMemo(() => {
    const counts = { ALL: 0, INFO: 0, WARN: 0, ERROR: 0, DEBUG: 0, TRACE: 0 };
    for (const line of parsedLines()) {
      counts.ALL++;
      if (line.level in counts) {
        counts[line.level as keyof typeof counts]++;
      }
    }
    return counts;
  });

  const loadLogs = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<string[]>("read_log_file", { limit: 5000 });
      setLines(result);
      const path = await invoke<string>("get_log_path");
      setLogPath(path);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadLogs();
  });

  createEffect(() => {
    filteredLines();
    if (autoScroll() && logContainerRef) {
      requestAnimationFrame(() => {
        logContainerRef!.scrollTop = logContainerRef!.scrollHeight;
      });
    }
  });

  const openLogFolder = async () => {
    const path = logPath();
    if (!path) return;
    const dir = path.substring(0, path.lastIndexOf("/"));
    try {
      await open(dir);
    } catch {
      await open(path.substring(0, path.lastIndexOf("\\")));
    }
  };

  const copyLogs = async () => {
    const text = filteredLines()
      .map((l) => l.raw)
      .join("\n");
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const filterButtons: { label: string; value: LevelFilter }[] = [
    { label: "All", value: "ALL" },
    { label: "Info", value: "INFO" },
    { label: "Warn", value: "WARN" },
    { label: "Error", value: "ERROR" },
  ];

  return (
    <div class="flex h-full bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] overflow-hidden font-['Nunito_Sans']">
      <aside class="w-64 bg-[var(--color-bg-primary)] flex flex-col border-r border-[var(--color-border)]">
        <div class="p-6">
          <h1 class="font-bold text-xl text-[var(--color-text-primary)] tracking-tight">
            Poketto
          </h1>
        </div>

        <div class="flex-1 overflow-y-auto px-4 custom-scrollbar flex flex-col gap-4">
          <nav class="space-y-1">
            <button
              onClick={() => navigate("/")}
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group"
            >
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <LibraryIcon class="w-5 h-5 text-[var(--color-icon)]" />
              </div>
              <span class="font-medium">My Games</span>
            </button>
          </nav>

          <div class="h-px bg-[var(--color-border)] w-full" />

          <nav class="space-y-1">
            <button
              onClick={() => navigate("/settings")}
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group"
            >
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <SettingsIcon class="w-5 h-5 text-[var(--color-icon)]" />
              </div>
              <span class="font-medium">Settings</span>
            </button>
            <button class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl transition-all">
              <div class="w-9 h-9 rounded-lg bg-[var(--color-accent)] flex items-center justify-center">
                <ScrollText class="w-5 h-5 text-white" />
              </div>
              <span class="font-medium text-[var(--color-text-primary)]">
                Logs
              </span>
            </button>
            <button class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group">
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <LogOut class="w-5 h-5 text-[var(--color-icon)]" />
              </div>
              <span class="font-medium">Log out</span>
            </button>
          </nav>
        </div>

        <div class="p-6 text-xs text-[var(--color-text-tertiary)] font-medium text-center">
          Poketto Version: {__APP_VERSION__}
        </div>
      </aside>

      <main class="flex-1 flex flex-col min-w-0 bg-[var(--color-bg-primary)]">
        <header class="h-20 px-8 flex items-center justify-between gap-4 border-b border-[var(--color-border)]">
          <h2 class="text-2xl font-bold text-[var(--color-text-primary)]">
            Logs
          </h2>

          <div class="flex items-center gap-2">
            <button
              onClick={() => setAutoScroll(!autoScroll())}
              class={`flex items-center gap-2 px-3 py-2 rounded-xl text-sm transition-colors ${
                autoScroll()
                  ? "bg-[var(--color-accent)] text-white"
                  : "bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
              }`}
              title="Auto-scroll to bottom"
            >
              <ArrowDownToLine class="w-4 h-4" />
              <span class="font-medium">Auto-scroll</span>
            </button>

            <button
              onClick={copyLogs}
              class="flex items-center gap-2 px-3 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-xl text-sm text-[var(--color-text-secondary)] transition-colors"
              title="Copy filtered logs"
            >
              <Copy class="w-4 h-4" />
              <span class="font-medium">{copied() ? "Copied" : "Copy"}</span>
            </button>

            <button
              onClick={openLogFolder}
              class="flex items-center gap-2 px-3 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-xl text-sm text-[var(--color-text-secondary)] transition-colors"
              title="Open log folder"
            >
              <FolderOpen class="w-4 h-4" />
              <span class="font-medium">Open Folder</span>
            </button>

            <button
              onClick={loadLogs}
              disabled={loading()}
              class="flex items-center gap-2 px-3 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] disabled:opacity-50 rounded-xl text-sm text-white font-medium transition-colors"
            >
              <RefreshCw class={`w-4 h-4 ${loading() ? "animate-spin" : ""}`} />
              <span>Refresh</span>
            </button>
          </div>
        </header>

        <div class="px-8 py-4 flex items-center gap-4 border-b border-[var(--color-border)]">
          <div class="flex gap-1 bg-[var(--color-bg-secondary)] rounded-lg p-1">
            <For each={filterButtons}>
              {(btn) => (
                <button
                  onClick={() => setLevelFilter(btn.value)}
                  class={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
                    levelFilter() === btn.value
                      ? "bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-sm"
                      : "text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
                  }`}
                >
                  {btn.label}
                  <span class="ml-1.5 text-xs opacity-60">
                    {levelCounts()[btn.value]}
                  </span>
                </button>
              )}
            </For>
          </div>

          <div class="flex-1 max-w-md relative group">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--color-icon)] group-focus-within:text-[var(--color-accent)] transition-colors" />
            <input
              type="text"
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.currentTarget.value)}
              placeholder="Filter logs..."
              class="w-full pl-9 pr-8 py-2 bg-[var(--color-bg-secondary)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)] transition-all font-medium"
            />
            <Show when={searchQuery()}>
              <button
                onClick={() => setSearchQuery("")}
                class="absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--color-icon)] hover:text-[var(--color-text-primary)] transition-colors"
              >
                <X class="w-4 h-4" />
              </button>
            </Show>
          </div>

          <span class="text-xs text-[var(--color-text-tertiary)] font-medium">
            {filteredLines().length} / {parsedLines().length} lines
          </span>
        </div>

        <Show
          when={!error()}
          fallback={
            <div class="flex-1 flex items-center justify-center">
              <div class="text-center space-y-3">
                <p class="text-[var(--color-danger)] font-medium">
                  Failed to load logs
                </p>
                <p class="text-sm text-[var(--color-text-tertiary)]">
                  {error()}
                </p>
                <button
                  onClick={loadLogs}
                  class="px-4 py-2 bg-[var(--color-accent)] text-white rounded-xl text-sm font-medium"
                >
                  Retry
                </button>
              </div>
            </div>
          }
        >
          <Show
            when={filteredLines().length > 0}
            fallback={
              <div class="flex-1 flex flex-col items-center justify-center text-[var(--color-text-tertiary)]">
                <FileText class="w-12 h-12 mb-3 opacity-20" />
                <Show
                  when={parsedLines().length > 0}
                  fallback={
                    <p>
                      {loading() ? "Loading logs..." : "No log entries found."}
                    </p>
                  }
                >
                  <p>No logs match the current filter.</p>
                </Show>
              </div>
            }
          >
            <div
              ref={logContainerRef}
              class="flex-1 overflow-y-auto px-8 py-4 custom-scrollbar"
            >
              <div class="space-y-px font-['JetBrains_Mono'] text-[13px] leading-6">
                <For each={filteredLines()}>
                  {(line) => (
                    <div
                      class={`flex items-start gap-3 px-3 py-1 rounded hover:bg-[var(--color-bg-secondary)] transition-colors group ${levelColor(line.level)}`}
                    >
                      <span class="text-[var(--color-text-tertiary)] shrink-0 select-none">
                        {line.time}
                      </span>
                      <span
                        class={`shrink-0 px-1.5 py-0 rounded text-[11px] font-bold uppercase select-none ${levelBadgeBg(line.level)}`}
                      >
                        {line.level.padEnd(5)}
                      </span>
                      <span class="text-[var(--color-text-tertiary)] shrink-0 truncate max-w-[200px] select-none text-xs leading-6">
                        {line.target}
                      </span>
                      <span class="flex-1 break-all">{line.message}</span>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </Show>
        </Show>
      </main>
    </div>
  );
}
