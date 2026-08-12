export const BADGE_OPTIONS = [
  { value: "LIVE_EVENT", label: "Live Event" },
  { value: "PREMIERE", label: "Premiere" },
  { value: "CHALLENGE", label: "Challenge" },
  { value: "COMPETITION", label: "Competition" },
  { value: "NEW_SEASON", label: "New Season" },
  { value: "MAJOR_UPDATE", label: "Major Update" },
  { value: "SPECIAL_EVENT", label: "Special Event" },
];

export const PURPOSE_OPTIONS = [
  { value: "APPROPRIATE_FOR_ALL_USERS", label: "Appropriate for all users" },
  { value: "ATTRACT_NEW_USERS", label: "Attract new users" },
  { value: "KEEP_ACTIVE_USERS_INFORMED", label: "Keep active users informed" },
  { value: "BRING_BACK_LAPSED_USERS", label: "Bring back lapsed users" },
];

export const PRIORITY_OPTIONS = [
  { value: "NORMAL", label: "Normal" },
  { value: "HIGH", label: "High" },
];

export const EVENT_STATE_LABELS: Record<string, { label: string; className: string }> = {
  DRAFT: { label: "Draft", className: "bg-secondary text-muted-foreground" },
  READY_FOR_REVIEW: { label: "Ready for Review", className: "bg-blue-500/10 text-blue-600" },
  WAITING_FOR_REVIEW: { label: "Waiting for Review", className: "bg-blue-500/10 text-blue-600" },
  IN_REVIEW: { label: "In Review", className: "bg-amber-500/10 text-amber-600" },
  REJECTED: { label: "Rejected", className: "bg-red-500/10 text-red-600" },
  ACCEPTED: { label: "Accepted", className: "bg-green-500/10 text-green-600" },
  APPROVED: { label: "Approved", className: "bg-green-500/10 text-green-600" },
  PUBLISHED: { label: "Published", className: "bg-green-500/10 text-green-600" },
  PAST: { label: "Past", className: "bg-secondary text-muted-foreground" },
  ARCHIVED: { label: "Archived", className: "bg-secondary text-muted-foreground" },
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
