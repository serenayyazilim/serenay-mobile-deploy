// Frontend side of the MCP server (src-tauri/src/mcp.rs): reports the open workspace and the
// deploy / run / test state to it, and runs the jobs it hands over as if their button was pressed.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { workspaceState, type WorkspaceMode } from "$lib/stores/workspace.svelte";
import { projectsState } from "$lib/stores/projects.svelte";
import { deployState } from "$lib/stores/deploy.svelte";
import { buildState, type FlutterDevice } from "$lib/stores/build.svelte";
import { testsState } from "$lib/stores/tests.svelte";
import { t } from "$lib/i18n/index.svelte";

type McpAction =
  | { action: "deploy"; projectId: string; platform: "ios" | "android" | "all"; track: "production" | "test"; whatsNew: string; bumpVersion: boolean }
  | { action: "run"; projectId: string; device: FlutterDevice | null }
  | { action: "hotReload" }
  | { action: "hotRestart" }
  | { action: "stop" }
  | { action: "tests"; projectId: string }
  | { action: "openWorkspace"; path: string; mode: WorkspaceMode };

async function runAction(a: McpAction) {
  if (a.action === "openWorkspace") {
    await invoke("workspace_recent_add", { path: a.path, name: null }).catch(() => {});
    await workspaceState.setWorkspace(a.path, a.mode);
    await projectsState.load(a.path);
    return;
  }
  if (a.action === "hotReload") return buildState.hotReload();
  if (a.action === "hotRestart") return buildState.hotRestart();
  if (a.action === "stop") return buildState.stopBuild();

  const workspace = workspaceState.path;
  const project = projectsState.projects.find((p) => p.id === a.projectId);
  if (!workspace || !project) return;

  if (a.action === "deploy") {
    deployState.pendingProject = project;
    return deployState.confirmDeploy(
      workspace,
      projectsState.versions,
      () => projectsState.fetchVersions(workspace),
      a.whatsNew.trim() || t("whatsNew.defaultText"),
      a.platform,
      a.bumpVersion,
      a.track
    );
  }
  if (a.action === "run") {
    buildState.selectedProject = project;
    return buildState.handleDeviceSelect(workspace, a.device);
  }
  if (a.action === "tests") return testsState.run(project, workspace);
}

export function connectMcp() {
  const stopEffect = $effect.root(() => {
    $effect(() => {
      const status = {
        workspace: workspaceState.path,
        deploy: {
          project: deployState.activeProject?.appName ?? null,
          status: deployState.deployStatus,
          message: deployState.deployMessage,
          progress: deployState.deployProgress,
          elapsedSeconds: deployState.deployElapsed,
          waitingForTwoFactorCode: deployState.twoFactorOpen,
          lastLogs: deployState.deployLogs.slice(-30),
        },
        run: {
          project: buildState.selectedProject?.appName ?? null,
          status: buildState.buildStatus,
          lastLogs: buildState.buildLogs.slice(-30),
        },
        tests: {
          project: testsState.project?.appName ?? null,
          status: testsState.status,
          results: testsState.results,
          lastLogs: testsState.logs.slice(-30),
        },
      };
      invoke("mcp_status_set", { status }).catch(() => {});
    });
  });
  const unlisten = listen<McpAction>("mcp-action", ({ payload }) => runAction(payload).catch(() => {}));

  return () => {
    stopEffect();
    unlisten.then((fn) => fn());
    invoke("mcp_status_set", { status: null }).catch(() => {});
  };
}
