export type ProjectKind = "flutter" | "expo" | "reactNative" | "nativeIos" | "nativeAndroid";
export type StorePlatform = "ios" | "android";

export const KIND_LABELS: Record<ProjectKind, string> = {
  flutter: "Flutter",
  expo: "Expo",
  reactNative: "React Native",
  nativeIos: "iOS (Swift)",
  nativeAndroid: "Android (Kotlin)",
};

export const STORE_NAMES: Record<StorePlatform, string> = { ios: "App Store", android: "Google Play" };

export interface Readiness {
  kind: ProjectKind;
  ios: { fastlaneMissing: string[]; teamId: string | null } | null;
  android: { fastlaneMissing: string[]; playKey: boolean | null; releaseSigning: boolean } | null;
}

export type StoreRecord = { status: "found"; locales: string[] } | { status: "missing" } | { status: "error"; message: string };
export type StoreChecks = Record<StorePlatform, StoreRecord | null>;
