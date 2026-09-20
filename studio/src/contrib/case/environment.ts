import type { CasePresentation } from "../../clients/dataSource";

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
