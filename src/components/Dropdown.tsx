import { Show, For, createSignal, createEffect, onCleanup } from "solid-js";
import { IconChevronDown, IconCheck } from "@tabler/icons-solidjs";

export interface DropdownOption {
  value: string;
  label: string;
}

export interface DropdownGroup {
  label?: string;
  options: DropdownOption[];
}

interface DropdownProps {
  value: string;
  groups: DropdownGroup[];
  placeholder?: string;
  disabled?: boolean;
  tone?: "primary" | "secondary";
  onChange: (value: string) => void;
}

export function Dropdown(props: DropdownProps) {
  const [open, setOpen] = createSignal(false);
  const [pos, setPos] = createSignal({
    left: 0,
    top: 0,
    width: 0,
    flip: false,
  });
  let buttonRef: HTMLButtonElement | undefined;
  let menuRef: HTMLDivElement | undefined;

  const selectedLabel = () => {
    for (const group of props.groups) {
      for (const option of group.options) {
        if (option.value === props.value) return option.label;
      }
    }
    return props.placeholder ?? "Select...";
  };

  const toggle = () => {
    if (props.disabled) return;
    if (open()) {
      setOpen(false);
      return;
    }
    const rect = buttonRef!.getBoundingClientRect();
    const below = window.innerHeight - rect.bottom;
    const flip = below < 224 && rect.top > 224;
    setPos({
      left: Math.max(8, Math.min(rect.left, window.innerWidth - 200)),
      top: flip ? Math.max(8, rect.top - 4) : rect.bottom + 4,
      width: Math.max(rect.width, 180),
      flip,
    });
    setOpen(true);
  };

  const choose = (value: string) => {
    props.onChange(value);
    setOpen(false);
  };

  createEffect(() => {
    if (!open()) return;
    const onPointerDown = (e: Event) => {
      const target = e.target as Node;
      if (menuRef?.contains(target) || buttonRef?.contains(target)) return;
      setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    const onScroll = (e: Event) => {
      if (menuRef && !menuRef.contains(e.target as Node)) setOpen(false);
    };
    const onResize = () => setOpen(false);
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKey);
    document.addEventListener("scroll", onScroll, true);
    window.addEventListener("resize", onResize);
    onCleanup(() => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("scroll", onScroll, true);
      window.removeEventListener("resize", onResize);
    });
  });

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        onClick={toggle}
        disabled={props.disabled}
        class={`w-full flex items-center justify-between gap-2 px-3 py-2 ${
          props.tone === "primary"
            ? "bg-[var(--color-bg-primary)]"
            : "bg-[var(--color-bg-secondary)]"
        } rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)] transition-all ${
          props.disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"
        }`}
      >
        <span class="truncate">{selectedLabel()}</span>
        <IconChevronDown
          class={`w-4 h-4 shrink-0 text-[var(--color-icon)] transition-transform ${
            open() ? "rotate-180" : ""
          }`}
          strokeWidth={1.5}
        />
      </button>
      <Show when={open()}>
        <div
          ref={menuRef}
          class="fixed z-[70] rounded-xl shadow-xl border border-[var(--color-border)] bg-[var(--color-bg-primary)] py-1.5 max-h-56 overflow-y-auto custom-scrollbar"
          style={{
            left: `${pos().left}px`,
            width: `${pos().width}px`,
            ...(pos().flip
              ? { bottom: `${window.innerHeight - pos().top}px` }
              : { top: `${pos().top}px` }),
          }}
        >
          <Show
            when={props.groups.some((group) => group.options.length > 0)}
            fallback={
              <p class="px-4 py-2 text-sm text-[var(--color-text-tertiary)]">
                {props.placeholder ?? "No options available"}
              </p>
            }
          >
            <For each={props.groups}>
              {(group) => (
                <Show when={group.options.length > 0}>
                  <Show when={group.label}>
                    <p class="px-4 pt-2 pb-1 text-[11px] uppercase tracking-wider text-[var(--color-text-tertiary)] font-bold">
                      {group.label}
                    </p>
                  </Show>
                  <For each={group.options}>
                    {(option) => (
                      <button
                        type="button"
                        onClick={() => choose(option.value)}
                        class={`w-full flex items-center justify-between gap-2 px-4 py-2 text-sm text-left transition-colors ${
                          option.value === props.value
                            ? "text-[var(--color-text-primary)] bg-[var(--color-bg-secondary)]"
                            : "text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-secondary)] hover:text-[var(--color-text-primary)]"
                        }`}
                      >
                        <span class="truncate">{option.label}</span>
                        <Show when={option.value === props.value}>
                          <IconCheck
                            class="w-4 h-4 shrink-0 text-[var(--color-success)]"
                            strokeWidth={1.5}
                          />
                        </Show>
                      </button>
                    )}
                  </For>
                </Show>
              )}
            </For>
          </Show>
        </div>
      </Show>
    </>
  );
}
