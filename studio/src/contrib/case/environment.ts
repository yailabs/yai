import type { CasePresentation } from "../../clients/dataSource";
import type { IconName } from "../../components/Icon";

export function environmentKindIcon(kind: string): IconName {
  return ({ discovery: "repository", filesystem: "environment", database: "database", database_query: "database", http_fetch: "network", http_service: "network", process_runner: "terminal", mcp: "resource" } as Record<string, IconName>)[kind] ?? "source";
}

/** Roles are projected YAI facts, never inferred from extensions or content. */
export function isPolicySource(source: CasePresentation["environment"]["sources"][number]) {
  return source.roles.includes("policy");
}

export function environmentMaterials(workspace: CasePresentation) {
  const sources = workspace.environment.sources.filter(source => !isPolicySource(source) || source.roles.some(role => role !== "policy"));
  const policyOnly = new Set(workspace.environment.sources.filter(source => isPolicySource(source) && !sources.includes(source)).map(source => source.id));
  return { sources, files: workspace.environment.files.filter(file => !policyOnly.has(file.source_ref)), policyOnlyCount: policyOnly.size };
}

export type FileTreeNode = { name: string; path: string; children: FileTreeNode[]; fileId?: string };

export function buildFileTree(files: CasePresentation["environment"]["files"]): FileTreeNode[] {
  const root: FileTreeNode = { name: "", path: "", children: [] };
  for (const file of [...files].sort((a, b) => a.path.localeCompare(b.path))) {
    let parent = root;
    const parts = file.path.split("/").filter(Boolean);
    parts.forEach((part, index) => {
      const path = parts.slice(0, index + 1).join("/");
      let node = parent.children.find((candidate) => candidate.name === part);
      if (!node) { node = { name: part, path, children: [] }; parent.children.push(node); }
      if (index === parts.length - 1) node.fileId = file.id;
      parent = node;
    });
  }
  return root.children;
}
