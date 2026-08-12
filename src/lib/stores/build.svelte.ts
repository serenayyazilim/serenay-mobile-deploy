import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { WorkspaceProject } from "$lib/stores/projects.svelte";
import { t } from "$lib/i18n/index.svelte";

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

  handleBuildClick(project: WorkspaceProject) {
    this.selectedProject = project;
    this.deviceDialogOpen = true;
  }

  async handleDeviceSelect(workspacePath: string, device: FlutterDevice | null) {
    const project = this.selectedProject;
    if (!project || !workspacePath) return;

    this.buildingProjectId = project.id;
    this.buildStatus = "running";
    this.buildLogs = [];

    let unlisten: UnlistenFn | null = null;

    try {
      this.buildLogs = [...this.buildLogs, `📁 ${t("build.activatingProject")}`];
      await invoke("project_activate", { workspacePath, projectId: project.id, backup: false });
      this.buildLogs = [...this.buildLogs, `✅ ${t("build.projectActivated")}`];

      const jobId = await invoke<string>("flutter_build_start", {
        workspacePath,
        deviceId: device?.id || null,
        projectId: project.id,
      });

      await new Promise<void>((resolve) => {
        listen<BuildEvent>(`flutter-build-event-${jobId}`, (event) => {
          const data = event.payload;
          if (data.type === "log") {
            this.buildLogs = [...this.buildLogs.slice(-50), data.message || ""];
          } else if (data.type === "error") {
            this.buildLogs = [...this.buildLogs, `❌ ${data.message}`];
          } else if (data.type === "done") {
            if (data.success) {
              this.buildStatus = "success";
              this.buildLogs = [...this.buildLogs, `✅ ${data.message}`];
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
    this.buildingProjectId = null;
    this.buildStatus = "idle";
    this.buildLogs = [];
    this.selectedProject = null;
  }

  closeErrorDialog() {
    this.errorDialogOpen = false;
  }
}

export const buildState = new BuildState();
