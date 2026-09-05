import { Show, createEffect, onCleanup } from "solid-js";
import {
  IconPlayerPlayFilled,
  IconDeviceGamepad2,
  IconEyeOff,
  IconDotsVertical,
} from "@tabler/icons-solidjs";
import type { Game } from "../types";

interface GameCardProps {
  game: Game;
  isRunning: boolean;
  showHidden: boolean;
  formatPlayTime: (m: number) => string;
  onPlay: (id: string) => void;
  onRemove: (id: string) => void;
  onEditSettings: (game: Game) => void;
  onClick: (game: Game) => void;
  onHide: (id: string, hidden: boolean) => void;
  activeContextMenu: string | null;
  onContextMenuOpen: (gameId: string) => void;
}

export function GameCard(props: GameCardProps) {
  let menuRef: HTMLDivElement | undefined;
  let cardRef: HTMLDivElement | undefined;

  const showMenu = () => props.activeContextMenu === props.game.id;

  const getMenuPosition = () => {
    if (!cardRef) return { x: 0, y: 0 };
    const rect = cardRef.getBoundingClientRect();
    return {
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2,
    };
  };

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    props.onContextMenuOpen(props.game.id);
  };

  const handleThreeDotsClick = (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    props.onContextMenuOpen(props.game.id);
  };

  const closeMenu = () => {
    if (props.activeContextMenu === props.game.id) {
      props.onContextMenuOpen("");
    }
  };

  createEffect(() => {
    if (showMenu()) {
      const handleOutsideClick = (e: MouseEvent) => {
        if (menuRef && !menuRef.contains(e.target as Node)) {
          closeMenu();
        }
      };

      setTimeout(() => {
        window.addEventListener("click", handleOutsideClick);
        window.addEventListener("contextmenu", handleOutsideClick);
      }, 0);

      onCleanup(() => {
        window.removeEventListener("click", handleOutsideClick);
        window.removeEventListener("contextmenu", handleOutsideClick);
      });
    }
  });

  return (
    <>
      <div
        ref={cardRef}
        class={`group relative aspect-[2/3] bg-[var(--color-bg-secondary)] rounded-2xl overflow-hidden cursor-pointer transition-all duration-300 hover:scale-105 hover:shadow-xl ${
          props.game.is_hidden && props.showHidden ? "opacity-50 grayscale" : ""
        } ${props.isRunning ? "ring-2 ring-[var(--color-success)] shadow-[0_0_20px_var(--color-success-light)]" : ""}`}
        onClick={() => props.onClick(props.game)}
        onContextMenu={handleContextMenu}
      >
        <Show
          when={props.game.cover_url}
          fallback={
            <div class="w-full h-full flex flex-col items-center justify-center p-4 bg-[var(--color-bg-secondary)]">
              <IconDeviceGamepad2 class="w-16 h-16 text-[var(--color-icon)] mb-4" strokeWidth={1.5} />
              <h3 class="text-[var(--color-text-secondary)] text-center font-bold line-clamp-2">
                {props.game.title}
              </h3>
            </div>
          }
        >
          <img
            src={props.game.cover_url!}
            alt={props.game.title}
            class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
            loading="lazy"
          />
        </Show>

        <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-60 group-hover:opacity-90 transition-opacity"></div>

        <div class="absolute inset-0 p-4 flex flex-col justify-between opacity-0 group-hover:opacity-100 transition-opacity duration-300">
          <div class="flex justify-between items-start">
            <button
              onClick={handleThreeDotsClick}
              class="bg-black/80 rounded-full px-2 py-1 hover:bg-black/90 transition-colors"
            >
              <IconDotsVertical class="w-4 h-4 text-white" strokeWidth={1.5} />
            </button>
          </div>

          <div class="space-y-3 translate-y-4 group-hover:translate-y-0 transition-transform duration-300 pb-2">
            <h3 class="text-white font-bold leading-tight drop-shadow-md line-clamp-2">
              {props.game.title}
            </h3>

            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-white/80 bg-black/50 px-2 py-1 rounded-md">
                {props.formatPlayTime(props.game.play_time)}
              </span>

              <button
                onClick={(e) => {
                  e.stopPropagation();
                  props.onPlay(props.game.id);
                }}
                class="w-10 h-10 rounded-full bg-[var(--color-accent)] text-white flex items-center justify-center hover:scale-110 active:scale-95 transition-all shadow-lg"
              >
                <IconPlayerPlayFilled class="w-4 h-4 fill-current ml-0.5" />
              </button>
            </div>
          </div>
        </div>

        <Show when={props.isRunning}>
          <div class="absolute top-4 left-4 px-2 py-1 bg-[var(--color-success)] text-white text-xs font-bold rounded shadow-lg animate-pulse">
            RUNNING
          </div>
        </Show>

        <Show when={props.game.is_hidden && props.showHidden}>
          <div class="absolute top-4 right-4 bg-black/80 rounded p-1.5">
            <IconEyeOff class="w-4 h-4 text-white/70" strokeWidth={1.5} />
          </div>
        </Show>
      </div>

      <Show when={showMenu()}>
        <div
          ref={menuRef}
          class="fixed z-50 bg-[var(--color-bg-primary)] rounded-xl shadow-xl py-1.5 min-w-[180px] animate-in fade-in zoom-in-95 duration-100"
          style={{
            left: `${getMenuPosition().x}px`,
            top: `${getMenuPosition().y}px`,
            transform: "translate(-50%, -50%)",
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onPlay(props.game.id);
              closeMenu();
            }}
            class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-secondary)] transition-colors"
          >
             Play Game
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onEditSettings(props.game);
              closeMenu();
            }}
            class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-secondary)] transition-colors"
          >
             Edit Settings
          </button>
          <div class="h-px bg-[var(--color-border)] my-1 mx-2" />
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onHide(props.game.id, !props.game.is_hidden);
              closeMenu();
            }}
            class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-secondary)] transition-colors"
          >
            <Show
              when={props.game.is_hidden}
              fallback={
                <>
                   Hide Game
                </>
              }
            >
               Unhide Game
            </Show>
          </button>
          <div class="h-px bg-[var(--color-border)] my-1 mx-2" />
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onRemove(props.game.id);
              closeMenu();
            }}
            class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger-light)] transition-colors"
          >
             Remove Game
          </button>
        </div>
      </Show>
    </>
  );
}
