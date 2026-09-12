const pad = (n: number): string => String(n).padStart(2, "0");

export function epochToLocalInput(epoch: number | null | undefined): string {
  if (epoch == null) return "";
  const date = new Date(epoch * 1000);
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

export function localInputToEpoch(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed === "") return null;
  const parsed = Date.parse(trimmed);
  return Number.isNaN(parsed) ? null : Math.floor(parsed / 1000);
}

export function parseOptionalInt(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed === "") return null;
  const parsed = Number(trimmed);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
}
