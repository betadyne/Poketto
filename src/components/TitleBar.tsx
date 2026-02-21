import { Minus, Square, X, Copy } from "lucide-solid";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { createSignal, onMount } from "solid-js";

export function TitleBar() {
  const [isMaximized, setIsMaximized] = createSignal(false);
  const appWindow = getCurrentWindow();

  onMount(async () => {
    setIsMaximized(await appWindow.isMaximized());

    await appWindow.onResized(async () => {
      setIsMaximized(await appWindow.isMaximized());
    });
  });

  const handleMinimize = async (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    await appWindow.minimize();
  };

  const handleMaximize = async (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    await appWindow.toggleMaximize();
  };

  const handleClose = async (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    await appWindow.close();
  };

  const handleDragStart = async () => {
    await appWindow.startDragging();
  };

  const preventDrag = (e: MouseEvent) => {
    e.stopPropagation();
  };

  return (
    <div class="h-8 bg-[var(--color-bg-primary)] border-b border-[var(--color-border)] flex items-center select-none">
      <div
        class="flex-1 h-full cursor-default"
        onMouseDown={handleDragStart}
      ></div>

      <div class="flex items-center h-full" onMouseDown={preventDrag}>
        <button
          onClick={handleMinimize}
          class="h-full w-11 flex items-center justify-center text-[var(--color-icon)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-secondary)] transition-colors"
          title="Minimize"
        >
          <Minus class="w-4 h-4" />
        </button>

        <button
          onClick={handleMaximize}
          class="h-full w-11 flex items-center justify-center text-[var(--color-icon)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-secondary)] transition-colors"
          title={isMaximized() ? "Restore" : "Maximize"}
        >
          {isMaximized() ? (
            <Copy class="w-3.5 h-3.5 rotate-180" />
          ) : (
            <Square class="w-3 h-3" />
          )}
        </button>

        <button
          onClick={handleClose}
          class="h-full w-11 flex items-center justify-center text-[var(--color-icon)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger-light)] transition-colors"
          title="Close"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
