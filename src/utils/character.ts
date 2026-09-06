export function sexLabel(code: string | null | undefined): string {
  if (code === "m") return "Male";
  if (code === "f") return "Female";
  if (code == null) return "Unknown";
  return code;
}
