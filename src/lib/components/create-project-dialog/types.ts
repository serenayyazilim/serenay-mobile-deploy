export type WizardStep = 1 | 2 | 3 | 4 | 5 | 6 | 7;

export const STEP_IDS: WizardStep[] = [1, 2, 3, 4, 5, 6, 7];

export interface FirebaseStepInfo {
  step: string;
  success: boolean;
  message: string;
}

export interface SentryAuthState {
  authenticated: boolean;
  org: string | null;
  message: string | null;
}

export interface CreateResult {
  success: boolean;
  message: string;
}

export interface FormState {
  projectId: string;
  appName: string;
  bgColor: string;
  icon512Path: string | null;
  icon1024Path: string | null;
  appLogoPath: string | null;
  splashPath: string | null;
  serconf: Record<string, string | number | boolean>;

  firebaseAutoCreate: boolean;
  firebaseAndroidPath: string | null;
  firebaseIosPath: string | null;
  firebaseCreated: boolean;

  sentryAutoCreate: boolean;
  sentryDsn: string;
  sentryOrgSlug: string;
}

export function createInitialForm(): FormState {
  return {
    projectId: "",
    appName: "",
    bgColor: "#FFFFFF",
    icon512Path: null,
    icon1024Path: null,
    appLogoPath: null,
    splashPath: null,
    serconf: {},

    firebaseAutoCreate: true,
    firebaseAndroidPath: null,
    firebaseIosPath: null,
    firebaseCreated: false,

    sentryAutoCreate: true,
    sentryDsn: "",
    sentryOrgSlug: "",
  };
}
