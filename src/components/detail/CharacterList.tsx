import { createMemo, For, Show } from "solid-js";
import type { VndbCharacter, VndbImage } from "../../types";
import { ROLE_NAMES } from "../../constants";
import { CharacterCard } from "./CharacterCard";

interface CharacterListProps {
  characters: VndbCharacter[];
  vnId: string;
  showSpoilers: boolean;
  shouldBlur: (img: VndbImage | null) => boolean;
}

export function CharacterList(props: CharacterListProps) {
  const groupedCharacters = createMemo(() => {
    const vnId = props.vnId;
    const filtered = props.characters.filter((c) => {
      const v = c.vns?.find((x) => x.id === vnId);
      return props.showSpoilers || (v?.spoiler || 0) === 0;
    });

    const groups: Record<string, VndbCharacter[]> = {};
    filtered.forEach((char) => {
      const vnInfo = char.vns?.find((v) => v.id === vnId);
      const role = vnInfo?.role || "appears";
      if (!groups[role]) groups[role] = [];
      groups[role].push(char);
    });

    Object.values(groups).forEach((chars) =>
      chars.sort((a, b) => a.name.localeCompare(b.name)),
    );

    return ["main", "primary", "side", "appears"]
      .filter((role) => groups[role]?.length)
      .map((role) => ({ role, chars: groups[role] }));
  });

  const visibleCount = createMemo(() =>
    groupedCharacters().reduce((sum, g) => sum + g.chars.length, 0),
  );

  return (
    <div class="max-w-6xl mx-auto">
      <Show
        when={props.characters.length > 0}
        fallback={
          <div class="text-[var(--color-text-tertiary)] text-center py-20 text-lg">
            No character data available.
          </div>
        }
      >
        <div class="flex items-center justify-between mb-8">
          <h2 class="text-2xl font-bold text-[var(--color-text-primary)]">
            Characters ({visibleCount()})
          </h2>
          <div class="text-sm text-[var(--color-text-secondary)] bg-[var(--color-bg-secondary)] px-4 py-2 rounded-lg">
            {props.showSpoilers ? "Show Spoilers: ON" : "Show Spoilers: OFF"}
          </div>
        </div>

        <div class="space-y-10">
          <For each={groupedCharacters()}>
            {(group) => (
              <div>
                <div class="flex items-center gap-4 mb-6">
                  <h3 class="text-xl font-bold text-[var(--color-text-primary)] font-['Plus_Jakarta_Sans']">
                    {ROLE_NAMES[group.role] || group.role}
                  </h3>
                  <span class="text-sm text-[var(--color-text-tertiary)] bg-[var(--color-bg-secondary)] px-3 py-1 rounded-full">
                    {group.chars.length}{" "}
                    {group.chars.length === 1 ? "character" : "characters"}
                  </span>
                  <div class="flex-1 h-px bg-[var(--color-border)]" />
                </div>

                <div class="space-y-6">
                  <For each={group.chars}>
                    {(char) => (
                      <CharacterCard
                        character={char}
                        vnId={props.vnId}
                        showSpoilers={props.showSpoilers}
                        shouldBlur={props.shouldBlur}
                      />
                    )}
                  </For>
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
