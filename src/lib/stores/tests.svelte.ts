import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { WorkspaceProject } from "$lib/stores/projects.svelte";
import type { BuildStatus } from "$lib/stores/build.svelte";
import { createTestLogFormatter } from "$lib/log-format";
import { t } from "$lib/i18n/index.svelte";

export interface TestCase {
  name: string;
  status: "passed" | "failed" | "skipped";
  message: string | null;
}

interface TestEvent {
  type: "log" | "done";
  message?: string;
  success?: boolean;
  results?: TestCase[];
}

class TestsState {
  project = $state<WorkspaceProject | null>(null);
  status = $state<BuildStatus>("idle");
  logs = $state<string[]>([]);
  results = $state<TestCase[]>([]);
  dialogOpen = $state(false);

  async run(project: WorkspaceProject, workspacePath: string) {
    if (this.status === "running") return;
    this.project = project;
    this.status = "running";
    this.logs = [];
    this.results = [];
    this.dialogOpen = true;

    let unlisten: UnlistenFn | null = null;
    try {
      // Same as Start: switch the multi-app workspace to this project first (no-op in generic mode).
      this.logs.push(`📁 ${t("build.activatingProject")}`);
      await invoke("project_activate", { workspacePath, projectId: project.id, backup: false });
      this.logs.push(`✅ ${t("build.projectActivated")}`);
      const jobId = await invoke<string>("tests_run_start", { workspacePath });
      const pushLog = createTestLogFormatter(project.kind);
      await new Promise<void>((resolve) => {
        listen<TestEvent>(`test-run-event-${jobId}`, ({ payload }) => {
          if (payload.type === "log") {
            pushLog(this.logs, payload.message ?? "");
          } else {
            this.results = payload.results ?? [];
            this.logs.push(`${payload.success ? "✅" : "❌"} ${payload.message}`);
            this.status = payload.success ? "success" : "error";
            resolve();
          }
        }).then((fn) => (unlisten = fn));
      });
    } catch (error) {
      this.logs.push(`❌ ${error instanceof Error ? error.message : String(error)}`);
      this.status = "error";
    } finally {
      (unlisten as UnlistenFn | null)?.();
    }
  }
}

export const testsState = new TestsState();
