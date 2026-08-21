import { invoke } from "@tauri-apps/api/core";
import type { WizardStep, FormState, FirebaseStepInfo, SentryAuthState, CreateResult } from "./types";
import { createInitialForm } from "./types";

export class CreateProjectWizard {
  workspacePath: string;
  onProjectCreated?: () => void;

  step = $state<WizardStep>(1);
  form = $state<FormState>(createInitialForm());
  projectIdError = $state<string | null>(null);

  bundleIdPrefix = $state("");

  firebaseLoading = $state(false);
  firebaseChecking = $state(false);
  firebaseLoggedIn = $state<boolean | null>(null);
  firebaseSteps = $state<FirebaseStepInfo[]>([]);
  firebaseError = $state<string | null>(null);

  sentryCheckLoading = $state(false);
  sentryLoading = $state(false);
  sentryAuth = $state<SentryAuthState | null>(null);
  sentryError = $state<string | null>(null);

  saving = $state(false);
  result = $state<CreateResult | null>(null);

  bundleId = $derived(this.bundleIdPrefix ? `${this.bundleIdPrefix}.${this.form.projectId}` : this.form.projectId);

  constructor(workspacePath: string, onProjectCreated?: () => void) {
    this.workspacePath = workspacePath;
    this.onProjectCreated = onProjectCreated;
  }

  reset() {
    this.step = 1;
    this.form = createInitialForm();
    this.projectIdError = null;
    this.firebaseLoading = false;
    this.firebaseChecking = false;
    this.firebaseLoggedIn = null;
    this.firebaseSteps = [];
    this.firebaseError = null;
    this.sentryCheckLoading = false;
    this.sentryLoading = false;
    this.sentryAuth = null;
    this.sentryError = null;
    this.saving = false;
    this.result = null;
    this.loadBundleIdPrefix();
  }

  async loadBundleIdPrefix() {
    try {
      const data = await invoke<{ configured: boolean; config: { bundleIdPrefix: string } | null }>("workspace_config_get", {
        workspace: this.workspacePath,
      });
      this.bundleIdPrefix = data.config?.bundleIdPrefix ?? "";
    } catch {
      this.bundleIdPrefix = "";
    }
  }

  setProjectId(value: string) {
    this.form.projectId = value;
    if (!value) {
      this.projectIdError = null;
    } else if (!/^[a-z0-9]+$/.test(value)) {
      this.projectIdError = "Only lowercase letters and digits are allowed";
    } else {
      this.projectIdError = null;
    }
  }

  canGoNext(): boolean {
    switch (this.step) {
      case 1:
        return /^[a-z0-9]+$/.test(this.form.projectId) && this.form.appName.trim().length > 0;
      case 2:
        return Boolean(this.form.icon512Path && this.form.icon1024Path);
      case 3:
        return Boolean(this.form.splashPath);
      case 4: {
        const apiUrl = this.form.serconf["API_URL"];
        return typeof apiUrl === "string" && apiUrl.trim().length > 0;
      }
      case 5:
      case 6:
      case 7:
        return true;
      default:
        return false;
    }
  }

  goNext() {
    if (this.canGoNext() && this.step < 7) this.step = ((this.step + 1) as WizardStep);
  }

  goBack() {
    if (this.step > 1) this.step = ((this.step - 1) as WizardStep);
  }

  goToStep(target: WizardStep) {
    if (target < this.step) this.step = target;
  }

  async checkFirebaseLogin() {
    if (this.firebaseChecking || this.firebaseLoggedIn !== null) return;
    this.firebaseChecking = true;
    try {
      const data = await invoke<{ accounts: string[]; currentAccount: string | null }>("firebase_accounts");
      this.firebaseLoggedIn = data.accounts.length > 0;
    } catch {
      this.firebaseLoggedIn = false;
    } finally {
      this.firebaseChecking = false;
    }
  }

  async handleFirebaseAutoCreate() {
    this.firebaseLoading = true;
    this.firebaseSteps = [];
    this.firebaseError = null;
    try {
      const data = await invoke<{ success: boolean; partial: boolean; message: string; steps: FirebaseStepInfo[] }>(
        "firebase_create_project",
        {
          projectId: this.form.projectId,
          appName: this.form.appName,
          bundleId: this.bundleId,
          workspacePath: this.workspacePath,
        }
      );
      this.firebaseSteps = data.steps;
      if (data.success || data.partial) {
        this.form.firebaseCreated = true;
      }
      if (!data.success) {
        this.firebaseError = data.message;
      }
    } catch (e) {
      this.firebaseError = String(e);
    } finally {
      this.firebaseLoading = false;
    }
  }

  async checkSentryAuth() {
    if (this.sentryCheckLoading || this.sentryAuth !== null) return;
    this.sentryCheckLoading = true;
    try {
      const data = await invoke<SentryAuthState>("sentry_check");
      this.sentryAuth = data;
      if (data.org && !this.form.sentryOrgSlug) {
        this.form.sentryOrgSlug = data.org;
      }
    } catch {
      this.sentryAuth = { authenticated: false, org: null, message: null };
    } finally {
      this.sentryCheckLoading = false;
    }
  }

  async handleSentryAutoCreate() {
    this.sentryLoading = true;
    this.sentryError = null;
    try {
      const data = await invoke<{ success: boolean; message: string; dsn: string }>("sentry_create_project", {
        projectId: this.form.projectId,
        orgSlug: this.form.sentryOrgSlug,
      });
      if (data.success && data.dsn) {
        this.form.sentryDsn = data.dsn;
      } else {
        this.sentryError = data.message;
      }
    } catch (e) {
      this.sentryError = String(e);
    } finally {
      this.sentryLoading = false;
    }
  }

  async handleCreate() {
    this.saving = true;
    this.result = null;
    try {
      const message = await invoke<string>("project_create", {
        params: {
          workspacePath: this.workspacePath,
          projectId: this.form.projectId,
          appName: this.form.appName.trim(),
          bgColor: this.form.bgColor,
          icon512Path: this.form.icon512Path,
          icon1024Path: this.form.icon1024Path,
          appLogoPath: this.form.appLogoPath,
          splashPath: this.form.splashPath,
          serconf: this.form.serconf,
          firebaseAndroidPath: this.form.firebaseCreated ? null : this.form.firebaseAndroidPath,
          firebaseIosPath: this.form.firebaseCreated ? null : this.form.firebaseIosPath,
          sentryDsn: this.form.sentryDsn.trim() || null,
          resume: false,
        },
      });
      this.result = { success: true, message };
      this.onProjectCreated?.();
    } catch (e) {
      this.result = { success: false, message: String(e) };
    } finally {
      this.saving = false;
    }
  }
}
