import { Show, createEffect, createSignal } from "solid-js";
import {
  IconBrandDiscord,
  IconDeviceFloppy,
  IconExternalLink,
} from "@tabler/icons-solidjs";
import { Dropdown } from "../Dropdown";
import type {
  CustomPresence,
  Game,
  PresenceActivityType,
  PresenceTimestampMode,
} from "../../types";
import {
  epochToLocalInput,
  localInputToEpoch,
  parseOptionalInt,
} from "../../utils";

interface PresenceTabProps {
  game: Game;
  onSave: (game: Game) => Promise<void>;
  onBack: () => void;
}
const ACTIVITY_OPTIONS = [
  { value: "", label: "Default (Playing)" },
  { value: "Listening", label: "Listening to" },
  { value: "Watching", label: "Watching" },
  { value: "Competing", label: "Competing in" },
];


const TIMESTAMP_OPTIONS = [
  { value: "SessionStart", label: "Session start" },
  { value: "Custom", label: "Custom schedule" },
];

const VERB_PHRASE: Record<string, string> = {
  Playing: "Playing a game",
  Listening: "Listening to",
  Watching: "Watching",
  Competing: "Competing in",
};

const inputClass =
  "w-full min-w-0 px-3 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]/40";

function SectionTitle(props: { children: string }) {
  return (
    <h4 class="text-xs uppercase tracking-widest text-[var(--color-text-tertiary)] font-bold">
      {props.children}
    </h4>
  );
}

function resolvePlaceholders(game: Game, text: string | null | undefined): string {
  return (text ?? "")
    .replace(/\{\{title\}\}/g, game.title)
    .replace(/\{\{state\}\}/g, game.discord_status?.trim() || "Idle")
    .replace(/\{\{cover\}\}/g, game.cover_url ?? "");
}

function PresencePreview(props: { game: Game; presence: CustomPresence }) {
  const activity = () => props.presence.activity_type ?? "Playing";
  const details = () => resolvePlaceholders(props.game, props.presence.details) || props.game.title;
  const state = () => resolvePlaceholders(props.game, props.presence.state);
  const name = () => props.presence.name?.trim() || "Poketto";
  const largeImage = () => resolvePlaceholders(props.game, props.presence.large_image);
  const buttons = () =>
    [
      props.presence.button1_text?.trim() || null,
      props.presence.button2_text?.trim() || null,
    ].filter((label): label is string => label !== null);

  return (
    <div class="rounded-xl bg-[#2b2d31] p-4 max-w-sm text-white">
      <p class="text-[11px] font-bold uppercase text-[#b5bac1] mb-1">
        {VERB_PHRASE[activity()] ?? VERB_PHRASE.Playing}
      </p>
      <p class="text-sm font-bold mb-2">{name()}</p>
      <div class="flex gap-3">
        <div class="relative shrink-0">
          <Show
            when={largeImage().startsWith("http")}
            fallback={
              <div class="w-[60px] h-[60px] rounded-lg bg-[#5865F2] flex items-center justify-center text-lg font-bold">
                {(details().trim()[0] ?? "?").toUpperCase()}
              </div>
            }
          >
            <img
              src={largeImage()}
              alt=""
              class="w-[60px] h-[60px] rounded-lg object-cover"
              loading="lazy"
            />
          </Show>
          <Show when={(props.presence.small_image ?? "").startsWith("http")}>
            <img
              src={props.presence.small_image!}
              alt=""
              title={props.presence.small_text ?? ""}
              class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full object-cover border-2 border-[#2b2d31]"
              loading="lazy"
            />
          </Show>
        </div>
        <div class="min-w-0 text-[13px] leading-5">
          <p class="truncate">{details()}</p>
          <Show when={state()}>
            <p class="text-[#dbdee1] truncate">{state()}</p>
          </Show>
          <Show when={props.presence.timestamp_mode !== "Custom"}>
            <p class="text-[#dbdee1]">Timer starts at launch</p>
          </Show>
          <Show
            when={
              props.presence.timestamp_mode === "Custom" &&
              props.presence.custom_start != null
            }
          >
            <p class="text-[#dbdee1]">
              From {new Date(props.presence.custom_start! * 1000).toLocaleString()}
            </p>
          </Show>
        </div>
      </div>
      <Show when={buttons().length > 0}>
        <div class="flex gap-2 mt-3">
          {buttons().map((label) => (
            <span class="flex-1 text-center text-[13px] font-medium bg-[#5865F2] rounded px-3 py-1.5">
              {label}
            </span>
          ))}
        </div>
      </Show>
    </div>
  );
}

export function PresenceTab(props: PresenceTabProps) {
  const [draft, setDraft] = createSignal<CustomPresence>(
    props.game.custom_presence ?? {},
  );
  const [editingId, setEditingId] = createSignal(props.game.id);
  const [isSaving, setIsSaving] = createSignal(false);

  createEffect(() => {
    if (props.game.id !== editingId()) {
      setEditingId(props.game.id);
      setDraft(props.game.custom_presence ?? {});
    }
  });

  const set = <K extends keyof CustomPresence>(
    key: K,
    value: CustomPresence[K],
  ) => setDraft((prev) => ({ ...prev, [key]: value }));

  const setText =
    (key: "name" | "details" | "details_url" | "state" | "state_url" | "client_id" | "large_image" | "large_text" | "small_image" | "small_text" | "button1_text" | "button1_url" | "button2_text" | "button2_url") =>
    (event: InputEvent & { currentTarget: HTMLInputElement }) => {
      const value = event.currentTarget.value.trim();
      set(key, value === "" ? null : value);
    };

  const save = async (presence: CustomPresence | null) => {
    if (isSaving()) return;
    setIsSaving(true);
    try {
      await props.onSave({ ...props.game, custom_presence: presence });
    } finally {
      setIsSaving(false);
    }
  };

  const enable = () =>
    save({ details: "{{title}}", state: "{{state}}" });

  const text = (key: Parameters<typeof setText>[0], placeholder: string) => (
    <input
      type="text"
      value={(draft()[key] ?? "") as string}
      placeholder={placeholder}
      onInput={setText(key)}
      class={inputClass}
    />
  );

  return (
    <div class="max-w-6xl mx-auto">
      <Show
        when={props.game.custom_presence}
        fallback={
          <div class="max-w-2xl mx-auto text-center py-16 px-4 space-y-4">
            <IconBrandDiscord
              class="w-12 h-12 mx-auto text-[var(--color-accent)]"
              strokeWidth={1.5}
            />
            <h3 class="text-2xl font-bold text-[var(--color-text-primary)]">
              Enable Custom Presence?
            </h3>
            <p class="text-[var(--color-text-secondary)] leading-relaxed">
              Custom presence replaces Poketto's default Discord status for
              this game only. Other games keep the default status. Use{" "}
              <code class="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-sm">
                {"{{title}}"}
              </code>
              ,{" "}
              <code class="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-sm">
                {"{{state}}"}
              </code>{" "}
              and{" "}
              <code class="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-sm">
                {"{{cover}}"}
              </code>{" "}
              as placeholders — they resolve to this game's title, status
              text and cover while you play.
            </p>
            <div class="flex flex-col sm:flex-row gap-3 justify-center pt-2">
              <button
                onClick={enable}
                disabled={isSaving()}
                class="px-6 py-2.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] disabled:opacity-50 rounded-full text-white font-bold text-sm transition-colors"
              >
                {isSaving() ? "Enabling..." : "Enable Custom Presence"}
              </button>
              <button
                onClick={props.onBack}
                class="px-6 py-2.5 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-full text-[var(--color-text-primary)] font-bold text-sm transition-colors"
              >
                Not Now
              </button>
            </div>
          </div>
        }
      >
        <div class="grid gap-8 lg:grid-cols-[1fr_320px]">
          <div class="space-y-6 min-w-0">
            <div class="space-y-3">
              <SectionTitle>Activity</SectionTitle>
              <div class="grid sm:grid-cols-2 gap-3">
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Application ID (empty uses the Poketto app)
                  </span>
                  {text("client_id", "e.g. 123456789012345678")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Display name (empty uses Poketto)
                  </span>
                  {text("name", "e.g. My VN Time")}
                </label>
              </div>
              <div class="grid sm:grid-cols-2 gap-3">
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Type
                  </span>
                  <Dropdown
                    value={draft().activity_type ?? ""}
                    groups={[{ options: ACTIVITY_OPTIONS }]}
                    onChange={(value) =>
                      set(
                        "activity_type",
                        value === "" ? null : (value as PresenceActivityType),
                      )
                    }
                  />
                </label>
              </div>
            </div>

            <div class="space-y-3">
              <SectionTitle>Text</SectionTitle>
              <div class="grid sm:grid-cols-2 gap-3">
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Details (first line)
                  </span>
                  {text("details", "{{title}}")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Details link
                  </span>
                  {text("details_url", "https://...")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    State (second line)
                  </span>
                  {text("state", "{{state}}")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    State link
                  </span>
                  {text("state_url", "https://...")}
                </label>
              </div>
            </div>

            <div class="space-y-3">
              <SectionTitle>Party</SectionTitle>
              <div class="grid grid-cols-2 gap-3 max-w-xs">
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Size
                  </span>
                  <input
                    type="number"
                    min="0"
                    value={draft().party_size ?? ""}
                    onInput={(e) => set("party_size", parseOptionalInt(e.currentTarget.value))}
                    class={`${inputClass} [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none`}
                  />
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Max
                  </span>
                  <input
                    type="number"
                    min="0"
                    value={draft().party_max ?? ""}
                    onInput={(e) => set("party_max", parseOptionalInt(e.currentTarget.value))}
                    class={`${inputClass} [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none`}
                  />
                </label>
              </div>
            </div>

            <div class="space-y-3">
              <SectionTitle>Timestamps</SectionTitle>
              <label class="block space-y-1 max-w-xs">
                <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                  Mode
                </span>
                <Dropdown
                  value={draft().timestamp_mode ?? "SessionStart"}
                  groups={[{ options: TIMESTAMP_OPTIONS }]}
                  onChange={(value) =>
                    set("timestamp_mode", value as PresenceTimestampMode)
                  }
                />
              </label>
              <Show when={(draft().timestamp_mode ?? "SessionStart") === "Custom"}>
                <div class="grid sm:grid-cols-2 gap-3">
                  <label class="block space-y-1">
                    <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                      Start
                    </span>
                    <input
                      type="datetime-local"
                      step="1"
                      value={epochToLocalInput(draft().custom_start)}
                      onInput={(e) =>
                        set("custom_start", localInputToEpoch(e.currentTarget.value))
                      }
                      class={inputClass}
                    />
                  </label>
                  <label class="block space-y-1">
                    <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                      End (shows a progress bar)
                    </span>
                    <input
                      type="datetime-local"
                      step="1"
                      value={epochToLocalInput(draft().custom_end)}
                      onInput={(e) =>
                        set("custom_end", localInputToEpoch(e.currentTarget.value))
                      }
                      class={inputClass}
                    />
                  </label>
                </div>
              </Show>
            </div>

            <div class="space-y-3">
              <SectionTitle>Images</SectionTitle>
              <div class="grid sm:grid-cols-2 gap-3">
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Large image URL or asset key
                  </span>
                  {text("large_image", "{{cover}}")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Large hover text
                  </span>
                  {text("large_text", "e.g. Cover art")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Small image URL or asset key
                  </span>
                  {text("small_image", "https://...")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Small hover text
                  </span>
                  {text("small_text", "e.g. Route A")}
                </label>
              </div>
            </div>

            <div class="space-y-3">
              <SectionTitle>Buttons</SectionTitle>
              <div class="grid sm:grid-cols-2 gap-3">
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Button 1 label
                  </span>
                  {text("button1_text", "e.g. View on VNDB")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Button 1 link
                  </span>
                  {text("button1_url", "https://...")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Button 2 label
                  </span>
                  {text("button2_text", "e.g. Source")}
                </label>
                <label class="block space-y-1">
                  <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                    Button 2 link
                  </span>
                  {text("button2_url", "https://...")}
                </label>
              </div>
            </div>

            <div class="flex flex-col sm:flex-row gap-3 pt-2">
              <button
                onClick={() => save(draft())}
                disabled={isSaving()}
                class="px-6 py-2.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] disabled:opacity-50 rounded-full text-white font-bold text-sm transition-colors flex items-center justify-center gap-2"
              >
                <IconDeviceFloppy class="w-4 h-4" strokeWidth={1.5} />
                {isSaving() ? "Saving..." : "Save Presence"}
              </button>
              <button
                onClick={() => save(null)}
                disabled={isSaving()}
                class="px-6 py-2.5 bg-[var(--color-danger-light)] hover:opacity-80 disabled:opacity-50 rounded-full text-[var(--color-danger)] font-bold text-sm transition-colors"
              >
                Disable Custom Presence
              </button>
            </div>
          </div>

          <div class="space-y-3 lg:pt-8">
            <SectionTitle>Preview</SectionTitle>
            <PresencePreview game={props.game} presence={draft()} />
            <a
              href="https://docs.customrp.xyz/setting-up"
              target="_blank"
              rel="noreferrer"
              class="inline-flex items-center gap-1 text-xs text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)]"
            >
              <IconExternalLink class="w-3.5 h-3.5" strokeWidth={1.5} />
              CustomRP setup guide (client IDs, asset keys)
            </a>
          </div>
        </div>
      </Show>
    </div>
  );
}
