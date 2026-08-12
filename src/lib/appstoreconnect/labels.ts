export const BADGE_OPTIONS = [
  { value: "LIVE_EVENT", label: "Canlı Etkinlik" },
  { value: "PREMIERE", label: "Prömiyer" },
  { value: "CHALLENGE", label: "Meydan Okuma" },
  { value: "COMPETITION", label: "Yarışma" },
  { value: "NEW_SEASON", label: "Yeni Sezon" },
  { value: "MAJOR_UPDATE", label: "Büyük Güncelleme" },
  { value: "SPECIAL_EVENT", label: "Özel Etkinlik" },
];

export const PURPOSE_OPTIONS = [
  { value: "APPROPRIATE_FOR_ALL_USERS", label: "Tüm kullanıcılar için uygun" },
  { value: "ATTRACT_NEW_USERS", label: "Yeni kullanıcı kazanma" },
  { value: "KEEP_ACTIVE_USERS_INFORMED", label: "Aktif kullanıcıları bilgilendirme" },
  { value: "BRING_BACK_LAPSED_USERS", label: "Ayrılan kullanıcıları geri kazanma" },
];

export const PRIORITY_OPTIONS = [
  { value: "NORMAL", label: "Normal" },
  { value: "HIGH", label: "Yüksek" },
];

export const EVENT_STATE_LABELS: Record<string, { label: string; className: string }> = {
  DRAFT: { label: "Taslak", className: "bg-secondary text-muted-foreground" },
  READY_FOR_REVIEW: { label: "İncelemeye Hazır", className: "bg-blue-500/10 text-blue-600" },
  WAITING_FOR_REVIEW: { label: "İnceleme Bekliyor", className: "bg-blue-500/10 text-blue-600" },
  IN_REVIEW: { label: "İnceleniyor", className: "bg-amber-500/10 text-amber-600" },
  REJECTED: { label: "Reddedildi", className: "bg-red-500/10 text-red-600" },
  ACCEPTED: { label: "Kabul Edildi", className: "bg-green-500/10 text-green-600" },
  APPROVED: { label: "Onaylandı", className: "bg-green-500/10 text-green-600" },
  PUBLISHED: { label: "Yayında", className: "bg-green-500/10 text-green-600" },
  PAST: { label: "Geçmiş", className: "bg-secondary text-muted-foreground" },
  ARCHIVED: { label: "Arşivlendi", className: "bg-secondary text-muted-foreground" },
};

export const LOCALE_SUGGESTIONS = [
  "tr", "en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "it", "pt-BR", "pt-PT",
  "ru", "ar-SA", "ja", "ko", "zh-Hans", "zh-Hant", "nl-NL", "sv", "pl", "el",
];

export function fromDatetimeLocal(value: string): string | null {
  if (!value) return null;
  const d = new Date(value);
  if (isNaN(d.getTime())) return null;
  return d.toISOString();
}
