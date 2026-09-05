import { For, Show } from "solid-js";
import { IconUser } from "@tabler/icons-solidjs";
import type { VndbCharacter, VndbTrait, VndbImage } from "../../types";
import { ROLE_NAMES, TRAIT_ORDER } from "../../constants";
import { stripBBCode } from "../../utils";

interface CharacterCardProps {
  character: VndbCharacter;
  vnId: string;
  showSpoilers: boolean;
  shouldBlur: (img: VndbImage | null) => boolean;
}

function groupTraits(
  traits: VndbTrait[] | null | undefined,
  showSpoiler: boolean,
): [string, VndbTrait[]][] {
  if (!traits) return [];
  const groups: Record<string, VndbTrait[]> = {};
  traits
    .filter((t) => showSpoiler || t.spoiler === 0)
    .forEach((t) => {
      const g = t.group_name || "Other";
      if (!groups[g]) groups[g] = [];
      groups[g].push(t);
    });
  return TRAIT_ORDER.filter((g) => groups[g])
    .map((g) => [g, groups[g]] as [string, VndbTrait[]])
    .concat(Object.entries(groups).filter(([g]) => !TRAIT_ORDER.includes(g)));
}

function TraitValue(props: { traits: VndbTrait[] }) {
  return (
    <span>
      <For each={props.traits}>
        {(t, i) => (
          <>
            <Show when={i() > 0}>, </Show>
            <span class={(t.spoiler ?? 0) > 0 ? "text-orange-500" : "text-[var(--color-text-primary)]"}>
              {t.name}
              <Show when={(t.spoiler ?? 0) > 0}>
                <sup class="text-orange-500 text-[10px] ml-0.5">S</sup>
              </Show>
            </span>
          </>
        )}
      </For>
    </span>
  );
}

export function CharacterCard(props: CharacterCardProps) {
  const vn = () => props.character.vns?.find((v) => v.id === props.vnId);
  const isSpoiler = () => (vn()?.spoiler || 0) > 0;
  const traits = () => groupTraits(props.character.traits, props.showSpoilers);
  const sex = () => props.character.sex?.[0];
  const role = () => vn()?.role || "appears";
  const char = props.character;

  return (
    <div class="flex gap-6 bg-[var(--color-bg-primary)] rounded-2xl p-6 hover:bg-[var(--color-bg-secondary)] transition-colors">
      <div class="w-48 flex-shrink-0">
        <Show
          when={char.image?.url}
          fallback={
            <div class="aspect-[3/4] bg-[var(--color-border)] rounded-xl flex items-center justify-center">
              <IconUser class="w-12 h-12 text-[var(--color-icon)]" strokeWidth={1.5} />
            </div>
          }
        >
          <img
            src={char.image!.url}
            alt={char.name}
            loading="lazy"
            class={`w-full rounded-xl shadow-lg ${
              (!props.showSpoilers && isSpoiler()) ||
              props.shouldBlur(char.image ?? null)
                ? "blur-xl"
                : ""
            }`}
          />
        </Show>
      </div>

      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-3 mb-4 border-b border-[var(--color-border)] pb-4 flex-wrap">
          <h3
            class={`text-2xl font-bold font-['Nunito'] ${isSpoiler() ? "text-orange-500" : "text-[var(--color-text-primary)]"}`}
          >
            {char.name}
          </h3>
          <Show when={char.original}>
            <span class="text-[var(--color-text-tertiary)] text-lg">{char.original}</span>
          </Show>
          <span
            class={`text-xs px-2.5 py-1 rounded-full font-bold tracking-wide ${
              role() === "main"
                ? "bg-yellow-100 text-yellow-700"
                : role() === "primary"
                  ? "bg-[var(--color-accent)]/10 text-[var(--color-accent)]"
                  : role() === "side"
                    ? "bg-[var(--color-accent)]/10 text-[var(--color-accent)]"
                    : "bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)]"
            }`}
          >
            {ROLE_NAMES[role()] || role()}
          </span>
          <Show when={sex()}>
            <span class="text-[var(--color-accent)] text-sm ml-auto bg-[var(--color-accent)]/10 px-2 py-1 rounded">
              {sex() === "m" ? "Male" : sex() === "f" ? "Female" : sex()}
            </span>
          </Show>
          <Show when={isSpoiler()}>
            <span class="text-orange-500 text-xs px-2 py-1 bg-orange-100 rounded font-bold tracking-wide">
              SPOILER
            </span>
          </Show>
        </div>

        <table class="w-full text-sm text-[var(--color-text-secondary)]">
          <tbody>
            <Show when={char.aliases?.length}>
              <tr>
                <td class="text-[var(--color-text-tertiary)] py-1.5 pr-6 align-top w-32 font-medium">
                  Aliases
                </td>
                <td class="text-[var(--color-text-primary)] py-1.5">
                  {char.aliases!.join(", ")}
                </td>
              </tr>
            </Show>
            <Show when={char.age || char.birthday}>
              <tr>
                <td class="text-[var(--color-text-tertiary)] py-1.5 pr-6 align-top font-medium">
                  Age/Birthday
                </td>
                <td class="text-[var(--color-text-primary)] py-1.5">
                  <Show when={char.age}>{char.age} years</Show>
                  <Show when={char.age && char.birthday}>, </Show>
                  <Show when={char.birthday}>
                    {char.birthday![1]}/{char.birthday![0]}
                  </Show>
                </td>
              </tr>
            </Show>
            <Show
              when={
                char.height ||
                char.weight ||
                (char.bust && char.waist && char.hips)
              }
            >
              <tr>
                <td class="text-[var(--color-text-tertiary)] py-1.5 pr-6 align-top font-medium">
                  Measurements
                </td>
                <td class="text-[var(--color-text-primary)] py-1.5 flex gap-4">
                  <Show when={char.height}>
                    <span>H: {char.height}cm</span>
                  </Show>
                  <Show when={char.weight}>
                    <span>W: {char.weight}kg</span>
                  </Show>
                  <Show when={char.bust}>
                    <span>
                      BWH: {char.bust}-{char.waist}-{char.hips}{" "}
                      <Show when={char.cup}>({char.cup})</Show>
                    </span>
                  </Show>
                </td>
              </tr>
            </Show>
            <For each={traits()}>
              {([tgroup, items]) => (
                <tr>
                  <td class="text-[var(--color-text-tertiary)] py-1.5 pr-6 align-top font-medium">
                    {tgroup}
                  </td>
                  <td class="py-1.5">
                    <TraitValue traits={items} />
                  </td>
                </tr>
              )}
            </For>
            <Show when={char.description}>
              <tr>
                <td
                  class="text-[var(--color-text-tertiary)] py-3 pr-6 align-top font-medium"
                  colspan="2"
                >
                  <div class="text-[var(--color-text-secondary)] text-sm whitespace-pre-line leading-relaxed font-['Nunito_Sans'] bg-[var(--color-bg-primary)] p-4 rounded-lg mt-2">
                    {stripBBCode(char.description!)}
                  </div>
                </td>
              </tr>
            </Show>
          </tbody>
        </table>
      </div>
    </div>
  );
}
