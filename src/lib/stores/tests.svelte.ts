import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { WorkspaceProject } from "$lib/stores/projects.svelte";
import type { BuildStatus } from "$lib/stores/build.svelte";

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
      const jobId = await invoke<string>("tests_run_start", { workspacePath });
      await new Promise<void>((resolve) => {
        listen<TestEvent>(`test-run-event-${jobId}`, ({ payload }) => {
          if (payload.type === "log") {
            this.logs.push(payload.message ?? "");
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
