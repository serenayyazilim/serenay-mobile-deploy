import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceProject {
  id: string;
  bundleId: string;
  appName: string;
}

export type ProjectSortBy = "name" | "version";
export type ProjectSortDirection = "asc" | "desc";

// Compares "19.0.3+190000300" style version strings numerically (build number ignored).
function compareVersions(a: string, b: string): number {
  if (!a && !b) return 0;
  if (!a) return -1;
  if (!b) return 1;

  const partsA = a.split("+")[0].split(".").map((n) => parseInt(n, 10) || 0);
  const partsB = b.split("+")[0].split(".").map((n) => parseInt(n, 10) || 0);

  for (let i = 0; i < Math.max(partsA.length, partsB.length); i++) {
    const diff = (partsA[i] ?? 0) - (partsB[i] ?? 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

class ProjectsState {
  projects = $state<WorkspaceProject[]>([]);
  versions = $state<Record<string, string>>({});
  loading = $state(true);
  searchQuery = $state("");
  sortBy = $state<ProjectSortBy>("name");
  sortDirection = $state<ProjectSortDirection>("asc");

  filtered = $derived.by(() => {
    const query = this.searchQuery.trim().toLowerCase();
    const base = query
      ? this.projects.filter((p) => p.appName.toLowerCase().includes(query) || p.bundleId.toLowerCase().includes(query))
      : this.projects;

    const sign = this.sortDirection === "asc" ? 1 : -1;
    return [...base].sort((a, b) => {
      const cmp =
        this.sortBy === "name"
          ? a.appName.localeCompare(b.appName, "tr", { sensitivity: "base" })
          : compareVersions(this.versions[a.id] ?? "", this.versions[b.id] ?? "");
      return cmp * sign;
    });
  });

  setSort(by: ProjectSortBy) {
    if (this.sortBy === by) {
      this.sortDirection = this.sortDirection === "asc" ? "desc" : "asc";
    } else {
      this.sortBy = by;
      this.sortDirection = "asc";
    }
  }

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
