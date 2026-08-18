import { invoke } from "@tauri-apps/api/core";
import type { WorkspaceProject } from "$lib/stores/projects.svelte";

export async function sendSlackNotification(
  workspacePath: string,
  project: WorkspaceProject,
  success: boolean,
  version: string,
  errorMessage?: string,
  duration?: number
): Promise<void> {
  try {
    await invoke("slack_notify", {
      workspace: workspacePath,
      projectName: project.appName,
      projectId: project.id,
      platform: "all",
      version,
      success,
      message: errorMessage,
      duration,
    });
  } catch (err) {
    console.error("Failed to send Slack notification:", err);
  }
}
