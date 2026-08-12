import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceProject {
  id: string;
  bundleId: string;
  appName: string;
}

class ProjectsState {
  projects = $state<WorkspaceProject[]>([]);
  versions = $state<Record<string, string>>({});
  loading = $state(true);
  searchQuery = $state("");

  filtered = $derived.by(() => {
    const query = this.searchQuery.trim().toLowerCase();
    if (!query) return this.projects;
    return this.projects.filter(
      (p) => p.appName.toLowerCase().includes(query) || p.bundleId.toLowerCase().includes(query)
    );
  });

  async fetchProjects(workspace: string) {
    try {
      this.projects = await invoke<WorkspaceProject[]>("projects_list", { workspace });
    } catch (error) {
      console.error("Failed to load projects:", error);
    } finally {
      this.loading = false;
    }
  }

  async fetchVersions(workspace: string) {
    try {
      const result = await invoke<{ versions: Record<string, string> }>("projects_versions", { workspace });
      this.versions = result.versions;
    } catch (error) {
      console.error("Failed to load versions:", error);
    }
  }

  async load(workspace: string) {
    this.loading = true;
    await Promise.all([this.fetchProjects(workspace), this.fetchVersions(workspace)]);
  }
}

export const projectsState = new ProjectsState();
