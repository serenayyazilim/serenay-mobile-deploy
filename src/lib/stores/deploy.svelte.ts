import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { detectProgressFromLog } from "$lib/deploy-utils";
import { sendSlackNotification } from "$lib/slack";
import type { WorkspaceProject } from "$lib/stores/projects.svelte";
import { t } from "$lib/i18n/index.svelte";

type DeployStatus = "idle" | "activating" | "deploying" | "success" | "error";
type DeployPlatform = "ios" | "android" | "all";

interface DeployEvent {
  type: "started" | "input_required" | "log" | "error" | "done";
  processId?: string;
  prompt?: string;
  message?: string;
  success?: boolean;
  exitCode?: number | null;
}

class DeployState {
  deployingProjectId = $state<string | null>(null);
  deployStatus = $state<DeployStatus>("idle");
  deployMessage = $state("");
  deployProgress = $state(0);
  deployElapsed = $state(0);

  whatsNewDialogOpen = $state(false);
  pendingProject = $state<WorkspaceProject | null>(null);

  twoFactorOpen = $state(false);
  twoFactorPrompt = $state("");
  private twoFactorResolve: ((code: string | null) => void) | null = null;

  errorDialogOpen = $state(false);
  errorTitle = $state("");
  errorLogs = $state<string[]>([]);

  private elapsedInterval: ReturnType<typeof setInterval> | null = null;
  private startTime = 0;

  private startElapsedTimer() {
    this.startTime = Date.now();
    this.deployElapsed = 0;
    this.elapsedInterval = setInterval(() => {
      this.deployElapsed = Math.round((Date.now() - this.startTime) / 1000);
    }, 1000);
  }

  private stopElapsedTimer() {
    if (this.elapsedInterval) {
      clearInterval(this.elapsedInterval);
      this.elapsedInterval = null;
    }
  }

  handleDeploy(project: WorkspaceProject) {
    this.pendingProject = project;
    this.whatsNewDialogOpen = true;
  }

  cancelDeploy() {
    this.whatsNewDialogOpen = false;
    this.pendingProject = null;
  }

  submitTwoFactor(code: string) {
    this.twoFactorOpen = false;
    this.twoFactorResolve?.(code);
    this.twoFactorResolve = null;
  }

  cancelTwoFactor() {
    this.twoFactorOpen = false;
    this.twoFactorResolve?.(null);
    this.twoFactorResolve = null;
  }

  async confirmDeploy(
    workspacePath: string,
    projectVersions: Record<string, string>,
    onVersionRefresh: () => Promise<void>,
    whatsNew: string,
    platform: DeployPlatform = "all"
  ) {
    const project = this.pendingProject;
    if (!project || !workspacePath) return;

    this.whatsNewDialogOpen = false;
    this.pendingProject = null;

    this.deployingProjectId = project.id;
    this.deployProgress = 0;
    this.startElapsedTimer();

    let lastError: string | undefined;
    const allLogs: string[] = [];
    let unlisten: UnlistenFn | null = null;

    try {
      this.deployStatus = "activating";
      this.deployMessage = t("deploy.activatingProject");
      this.deployProgress = 20;

      await invoke("project_activate", { workspacePath, projectId: project.id, backup: false });

      this.deployStatus = "deploying";
      this.deployMessage = t("deploy.starting");
      this.deployProgress = 30;

      const processId = await invoke<string>("deploy_start", { platform, workspacePath, whatsNew });

      await new Promise<void>((resolve, reject) => {
        listen<DeployEvent>(`deploy-event-${processId}`, async (event) => {
          const data = event.payload;

          if (data.type === "input_required") {
            this.twoFactorPrompt = data.prompt || "";
            this.twoFactorOpen = true;
            this.deployStatus = "deploying";
            this.deployMessage = t("deploy.waitingForAppleAuth");

            const code = await new Promise<string | null>((r) => {
              this.twoFactorResolve = r;
            });

            if (code) {
              await invoke("deploy_submit_two_factor_code", { processId, code }).catch(() => {});
              this.deployMessage = t("deploy.codeSubmitted");
            }
          } else if (data.type === "log") {
            const msg = data.message || "";
            allLogs.push(msg);
            this.deployStatus = "deploying";
            this.deployMessage = msg;
            const progress = detectProgressFromLog(msg);
            if (progress > 0) this.deployProgress = progress;
          } else if (data.type === "error") {
            const msg = data.message || "";
            allLogs.push(`❌ ${msg}`);
            lastError = msg;
            console.error("[Deploy Error]", msg);
          } else if (data.type === "done") {
            if (data.success) {
              resolve();
            } else {
              reject(new Error(lastError || t("deploy.failed")));
            }
          }
        }).then((fn) => (unlisten = fn));
      });

      this.stopElapsedTimer();
      const duration = Math.round((Date.now() - this.startTime) / 1000);
      this.deployStatus = "success";
      this.deployMessage = t("deploy.completed");
      this.deployProgress = 100;
      await onVersionRefresh();
      sendSlackNotification(workspacePath, project, true, projectVersions[project.id] || "19.0.x", undefined, duration);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.stopElapsedTimer();
      const duration = Math.round((Date.now() - this.startTime) / 1000);

      console.error("[Deploy Failed]", errorMsg);
      this.deployStatus = "error";
      this.deployMessage = errorMsg;

      if (!this.errorDialogOpen) {
        this.errorTitle = t("deploy.errorTitle", { name: project.appName });
        this.errorLogs = allLogs.length > 0 ? allLogs : [errorMsg];
        this.errorDialogOpen = true;
      }

      sendSlackNotification(workspacePath, project, false, projectVersions[project.id] || "19.0.x", errorMsg, duration);
    } finally {
      (unlisten as UnlistenFn | null)?.();
    }
  }

  resetDeploy() {
    this.deployingProjectId = null;
    this.deployStatus = "idle";
    this.deployMessage = "";
    this.deployProgress = 0;
    this.deployElapsed = 0;
    this.stopElapsedTimer();
  }

  closeErrorDialog() {
    this.errorDialogOpen = false;
  }
}

export const deployState = new DeployState();
