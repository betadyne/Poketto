import type { VndbImage } from "../bindings";

export function shouldBlur(
  img: VndbImage | null,
  blurNsfw: boolean
): boolean {
  if (!blurNsfw || !img) return false;
  return (img.sexual ?? 0) >= 1 || (img.violence ?? 0) >= 1;
}
