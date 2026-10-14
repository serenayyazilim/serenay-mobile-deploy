import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { WorkspaceProject } from "$lib/stores/projects.svelte";
import { confirmState } from "$lib/stores/confirm.svelte";
import { t } from "$lib/i18n/index.svelte";
import { createLogFormatter } from "$lib/log-format";

export type BuildStatus = "idle" | "running" | "success" | "error";

export interface FlutterDevice {
  name: string;
  id: string;
  platform: string;
  type: string;
}

interface BuildEvent {
  type: "log" | "error" | "done";
  message?: string;
  success?: boolean;
}

class BuildState {
  buildingProjectId = $state<string | null>(null);
  buildLogs = $state<string[]>([]);
  buildStatus = $state<BuildStatus>("idle");
  deviceDialogOpen = $state(false);
  selectedProject = $state<WorkspaceProject | null>(null);

  errorDialogOpen = $state(false);
  errorTitle = $state("");
  errorLogs = $state<string[]>([]);

  logsDialogOpen = $state(false);
  jobId = $state<string | null>(null);
  isReloading = $state(false);
  isRestarting = $state(false);
  isStopping = $state(false);
  // Set by a stop pressed before `flutter_build_start` hands back a job ID (project still activating).
  private stopRequested = false;

  private autoCloseTimer: ReturnType<typeof setTimeout> | null = null;
  private readonly autoCloseDelayMs = 4000;

  private scheduleAutoClose() {
    if (this.autoCloseTimer) clearTimeout(this.autoCloseTimer);
    this.autoCloseTimer = setTimeout(() => this.resetBuild(), this.autoCloseDelayMs);
  }

  handleBuildClick(project: WorkspaceProject) {
    this.selectedProject = project;
    this.deviceDialogOpen = true;
  }

  async handleDeviceSelect(workspacePath: string, device: FlutterDevice | null) {
    const project = this.selectedProject;
    if (!project || !workspacePath) return;

    const isFlutter = !project.kind || project.kind === "flutter";
    const hasSplashImage =
      !isFlutter || (await invoke<boolean>("project_check_splash_image", { workspacePath, projectId: project.id }).catch(() => true));
    if (!hasSplashImage) {
      const proceed = await confirmState.ask(t("build.splashWarningTitle"), t("build.splashWarningDescription", { name: project.appName }));
      if (!proceed) return;
    }

    if (this.autoCloseTimer) {
      clearTimeout(this.autoCloseTimer);
      this.autoCloseTimer = null;
    }
    this.buildingProjectId = project.id;
    this.buildStatus = "running";
    this.buildLogs = [];
    this.jobId = null;
    this.stopRequested = false;

    let unlisten: UnlistenFn | null = null;

    try {
      this.buildLogs = [...this.buildLogs, `📁 ${t("build.activatingProject")}`];
      await invoke("project_activate", { workspacePath, projectId: project.id, backup: false });
      this.buildLogs = [...this.buildLogs, `✅ ${t("build.projectActivated")}`];
      if (this.stopRequested) {
        this.buildStatus = "success";
        this.buildLogs = [...this.buildLogs, "✅ App stopped"];
        this.scheduleAutoClose();
        return;
      }

      const jobId = await invoke<string>("flutter_build_start", {
        workspacePath,
        deviceId: device?.id || null,
        devicePlatform: device?.platform || null,
        projectId: project.id,
      });
      this.jobId = jobId;
      if (this.stopRequested) invoke("flutter_run_stop", { jobId }).catch(() => {});
      const pushLog = createLogFormatter();

      await new Promise<void>((resolve) => {
        listen<BuildEvent>(`flutter-build-event-${jobId}`, (event) => {
          const data = event.payload;
          if (data.type === "log") {
            pushLog(this.buildLogs, data.message || "");
          } else if (data.type === "error") {
            this.buildLogs = [...this.buildLogs, `❌ ${data.message}`];
          } else if (data.type === "done") {
            if (data.success) {
              this.buildStatus = "success";
              this.buildLogs = [...this.buildLogs, `✅ ${data.message}`];
              this.scheduleAutoClose();
            } else {
              this.buildStatus = "error";
              const newLogs = [...this.buildLogs, `❌ ${data.message}`];
              this.buildLogs = newLogs;
              this.errorTitle = t("build.buildError", { name: project.appName });
              this.errorLogs = newLogs;
              this.errorDialogOpen = true;
            }
            resolve();
          }
        }).then((fn) => (unlisten = fn));
      });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.buildStatus = "error";
      const newLogs = [...this.buildLogs, `❌ ${errorMsg}`];
      this.buildLogs = newLogs;
      this.errorTitle = t("build.buildError", { name: project.appName });
      this.errorLogs = newLogs;
      this.errorDialogOpen = true;
    } finally {
      (unlisten as UnlistenFn | null)?.();
    }
  }

  resetBuild() {
    if (this.autoCloseTimer) {
      clearTimeout(this.autoCloseTimer);
      this.autoCloseTimer = null;
    }
    this.buildingProjectId = null;
    this.buildStatus = "idle";
    this.buildLogs = [];
    this.selectedProject = null;
    this.logsDialogOpen = false;
    this.jobId = null;
  }

  openLogsDialog() {
    this.logsDialogOpen = true;
  }

  async hotReload() {
    if (!this.jobId || this.buildStatus !== "running" || this.isReloading) return;
    this.isReloading = true;
    this.buildLogs = [...this.buildLogs, `🔄 ${t("build.hotReloadTriggered")}`];
    try {
      await invoke("flutter_run_hot_reload", { jobId: this.jobId, restart: false });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.buildLogs = [...this.buildLogs, `❌ ${errorMsg}`];
    } finally {
      this.isReloading = false;
    }
  }

  async hotRestart() {
    if (!this.jobId || this.buildStatus !== "running" || this.isRestarting) return;
    this.isRestarting = true;
    this.buildLogs = [...this.buildLogs, `🔁 ${t("build.hotRestartTriggered")}`];
    try {
      await invoke("flutter_run_hot_reload", { jobId: this.jobId, restart: true });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.buildLogs = [...this.buildLogs, `❌ ${errorMsg}`];
    } finally {
      this.isRestarting = false;
    }
  }

  async stopBuild() {
    if (this.buildStatus !== "running" || this.isStopping || this.stopRequested) return;
    this.stopRequested = true;
    this.buildLogs = [...this.buildLogs, `⏹️ ${t("build.stopTriggered")}`];
    if (!this.jobId) return; // still activating: handled once `project_activate` returns
    this.isStopping = true;
    try {
      await invoke("flutter_run_stop", { jobId: this.jobId });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.buildLogs = [...this.buildLogs, `❌ ${errorMsg}`];
    } finally {
      this.isStopping = false;
    }
  }

  closeErrorDialog() {
    this.errorDialogOpen = false;
  }
}

export const buildState = new BuildState();
