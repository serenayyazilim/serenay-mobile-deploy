export function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  if (minutes < 60) return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
}

export function detectProgressFromLog(message: string): number {
  const m = message.toLowerCase();
  if (m.includes("clean") || m.includes("temizl")) return 40;
  if (m.includes("ios") && (m.includes("build") || m.includes("archiv"))) return 60;
  if (m.includes("android") && (m.includes("build") || m.includes("bundle"))) return 80;
  if (m.includes("upload") || m.includes("yükle") || m.includes("testflight") || m.includes("play store")) return 95;
  return 0;
}
