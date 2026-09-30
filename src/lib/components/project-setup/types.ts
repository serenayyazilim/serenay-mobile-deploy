import { invoke } from "@tauri-apps/api/core";

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

/** Kind to reopen the setup dialog with when something a deploy needs is still missing, else null. */
export async function incompleteSetup(workspacePath: string): Promise<ProjectKind | null> {
  const r = await invoke<Readiness>("setup_readiness", { workspacePath });
  if (r.kind === "expo") return r.kind;

  const androidMissing = !!r.android && (r.android.fastlaneMissing.length > 0 || r.android.playKey === false || !r.android.releaseSigning);
  let iosMissing = !!r.ios && (r.ios.fastlaneMissing.length > 0 || !r.ios.teamId);
  if (r.ios && !iosMissing) {
    iosMissing = !(await invoke<{ configured: boolean }>("asc_config_get", { workspace: workspacePath })).configured;
  }
  return iosMissing || androidMissing ? r.kind : null;
}
