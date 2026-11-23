import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { detectProgressFromLog } from "$lib/deploy-utils";
import { sendSlackNotification } from "$lib/slack";
import type { WorkspaceProject } from "$lib/stores/projects.svelte";
import { confirmState } from "$lib/stores/confirm.svelte";
import { t } from "$lib/i18n/index.svelte";
import { incompleteSetup, type ProjectKind } from "$lib/components/project-setup/types";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";

async function notifyDeployDone(title: string, body: string) {
  const granted = (await isPermissionGranted()) || (await requestPermission()) === "granted";
  if (granted) sendNotification({ title, body });
}

type DeployStatus = "idle" | "activating" | "deploying" | "success" | "error";
type DeployPlatform = "ios" | "android" | "all";
type ReleaseTrack = "production" | "test";

interface DeployEvent {
  type: "started" | "log" | "error" | "done";
  processId?: string;
  prompt?: string;
  message?: string;
  success?: boolean;
  exitCode?: number | null;
  cancelled?: boolean;
}

class DeployCancelled extends Error {}

class DeployState {
  deployingProjectId = $state<string | null>(null);
  deployStatus = $state<DeployStatus>("idle");
  deployMessage = $state("");
  deployProgress = $state(0);
  deployElapsed = $state(0);
  deployLogs = $state<string[]>([]);
  deployWarnings = $state<string[]>([]);
  isStopping = $state(false);
  logsDialogOpen = $state(false);
  activeProject = $state<WorkspaceProject | null>(null);

  whatsNewDialogOpen = $state(false);
  pendingProject = $state<WorkspaceProject | null>(null);

  errorDialogOpen = $state(false);
  firstReleaseOpen = $state(false);
  firstReleaseStores = $state<("ios" | "android")[]>([]);
  errorTitle = $state("");
  errorLogs = $state<string[]>([]);

  /** Set once the deploy process runs, so a stop can end it. */
  private processId: string | null = null;
  /** Set by a stop pressed before the process exists (project activating, or a question waiting). */
  private stopRequested = false;
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

  /** Set while the setup dialog is shown before a deploy because something is still missing. */
  setupKind = $state<ProjectKind | null>(null);

  async handleDeploy(project: WorkspaceProject, workspacePath: string) {
    this.pendingProject = project;
    // Single-app projects may have skipped parts of the setup; ask for them again now.
    if (project.kind) {
      this.setupKind = await incompleteSetup(workspacePath).catch(() => null);
      if (this.setupKind) return;
    }
    this.whatsNewDialogOpen = true;
  }

  finishSetup(completed: boolean) {
    this.setupKind = null;
    if (completed) this.whatsNewDialogOpen = true;
    else this.pendingProject = null;
  }

  cancelDeploy() {
    this.whatsNewDialogOpen = false;
    this.pendingProject = null;
  }

  /** Stops the running deploy, whichever step it is in. */
  async stopDeploy() {
    if (this.isStopping || (this.deployStatus !== "activating" && this.deployStatus !== "deploying")) return;
    this.isStopping = true;
    this.stopRequested = true;
    this.deployLogs = [...this.deployLogs, `⏹️ ${t("deploy.stopTriggered")}`];
    this.deployMessage = t("deploy.stopTriggered");
    // A question the deploy waits on is answered with "no".
    if (confirmState.open) confirmState.cancel();
    if (this.processId) await invoke("deploy_cancel", { processId: this.processId }).catch(() => {});
  }

  openLogsDialog() {
    this.logsDialogOpen = true;
  }

  async confirmDeploy(
    workspacePath: string,
    projectVersions: Record<string, string>,
    onVersionRefresh: () => Promise<void>,
    whatsNew: string,
    platform: DeployPlatform = "all",
    bumpVersion: boolean = true,
    track: ReleaseTrack = "production",
    smokeTest: boolean = false,
    allowUntranslatedAuto: boolean = false
  ) {
    const project = this.pendingProject;
    if (!project || !workspacePath) return;

    this.whatsNewDialogOpen = false;
    this.pendingProject = null;

    const hasSplashImage = await invoke<boolean>("deploy_check_splash_image", { workspacePath, projectId: project.id }).catch(() => true);
    if (!hasSplashImage) {
      const proceed = await confirmState.ask(t("deploy.splashWarningTitle"), t("deploy.splashWarningDescription", { name: project.appName }));
      if (!proceed) return;
    }

    this.stopRequested = false;
    this.isStopping = false;
    this.processId = null;
    this.deployingProjectId = project.id;
    this.activeProject = project;
    this.deployProgress = 0;
    this.deployLogs = [];
    this.deployWarnings = [];
    this.logsDialogOpen = false;
    this.startElapsedTimer();

    let lastError: string | undefined;
    let unlisten: UnlistenFn | null = null;

    try {
      this.deployStatus = "activating";
      this.deployMessage = t("deploy.activatingProject");
      this.deployProgress = 20;

      await invoke("project_activate", { workspacePath, projectId: project.id, backup: false });
      if (this.stopRequested) throw new DeployCancelled();

      this.deployStatus = "deploying";
      this.deployMessage = t("deploy.starting");
      this.deployProgress = 30;

      const start = (allowUntranslated: boolean) => invoke<{ processId: string; warnings: string[] }>("deploy_start", { platform, workspacePath, whatsNew, bumpVersion, track, smokeTest, allowUntranslated });
      let started: { processId: string; warnings: string[] };
      try {
        started = await start(false);
      } catch (error) {
        const locales = /TRANSLATION_FAILED:(\S+)/.exec(String(error))?.[1];
        if (!locales) throw error;
        // Also shown in the MCP status, since the question can only be answered in the app.
        this.deployMessage = t("deploy.translationFailedTitle");
        const proceed = allowUntranslatedAuto || (await confirmState.ask(this.deployMessage, t("deploy.translationFailedDescription", { locales: locales.replaceAll(",", ", ") })));
        if (!proceed) {
          if (this.stopRequested) throw new DeployCancelled();
          this.resetDeploy();
          return;
        }
        this.deployMessage = t("deploy.starting");
        started = await start(true);
      }
      const { processId, warnings } = started;
      this.processId = processId;
      this.deployWarnings = warnings;
      this.deployLogs.push(...warnings.map((w) => `⚠️ ${w}`));
      // A stop that came in while the process was starting.
      if (this.stopRequested) await invoke("deploy_cancel", { processId }).catch(() => {});

      await new Promise<void>((resolve, reject) => {
        listen<DeployEvent>(`deploy-event-${processId}`, async (event) => {
          const data = event.payload;

          if (data.type === "log") {
            const msg = data.message || "";
            this.deployLogs.push(msg);
            this.deployStatus = "deploying";
            this.deployMessage = msg;
            const progress = detectProgressFromLog(msg);
            if (progress > 0) this.deployProgress = progress;
          } else if (data.type === "error") {
            const msg = data.message || "";
            this.deployLogs = [...this.deployLogs, `❌ ${msg}`];
            lastError = msg;
            console.error("[Deploy Error]", msg);
          } else if (data.type === "done") {
            if (data.cancelled) {
              reject(new DeployCancelled());
            } else if (data.success) {
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
      notifyDeployDone(project.appName, `✅ ${t("deploy.completed")}`).catch(() => {});
      await onVersionRefresh();
      sendSlackNotification(workspacePath, project, true, projectVersions[project.id] || "19.0.x", undefined, duration);
    } catch (error) {
      if (error instanceof DeployCancelled) {
        // Stopped on purpose: not a failure, so no error dialog and no failure notice in Slack.
        this.stopElapsedTimer();
        this.errorLogs = [...this.deployLogs, `⏹️ ${t("deploy.stopped")}`];
        this.resetDeploy();
        return;
      }
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.stopElapsedTimer();
      const duration = Math.round((Date.now() - this.startTime) / 1000);

      console.error("[Deploy Failed]", errorMsg);
      this.deployStatus = "error";
      this.deployMessage = errorMsg;
      notifyDeployDone(project.appName, `❌ ${t("deploy.failed")}`).catch(() => {});

      // The app isn't in the store yet: explain the manual first step instead of showing a log.
      const missingStore = /APP_NOT_FOUND:(ios|android)/.exec(errorMsg)?.[1] as "ios" | "android" | undefined;
      if (missingStore) {
        this.firstReleaseStores = [missingStore];
        this.firstReleaseOpen = true;
        this.resetDeploy();
      } else if (!this.errorDialogOpen) {
        this.errorTitle = t("deploy.errorTitle", { name: project.appName });
        this.errorLogs = this.deployLogs.length > 0 ? this.deployLogs : [errorMsg];
        this.errorDialogOpen = true;
      }

      sendSlackNotification(workspacePath, project, false, projectVersions[project.id] || "19.0.x", errorMsg, duration);
    } finally {
      (unlisten as UnlistenFn | null)?.();
    }
  }

  resetDeploy() {
    this.processId = null;
    this.stopRequested = false;
    this.isStopping = false;
    this.deployingProjectId = null;
    this.deployStatus = "idle";
    this.deployMessage = "";
    this.deployProgress = 0;
    this.deployElapsed = 0;
    this.deployLogs = [];
    this.deployWarnings = [];
    this.logsDialogOpen = false;
    this.activeProject = null;
    this.stopElapsedTimer();
  }

  closeErrorDialog() {
    this.errorDialogOpen = false;
  }
}

export const deployState = new DeployState();
