import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { fileInput } from "./inputs";

/** No raw HTML or remote fetches. Links traverse qualified inventory through Workbench. */
export default function MarkdownPreview({ content, workspace, input, actions }: SurfaceRendererProps & { content: string }) {
  const qualifiedFile = (href?: string) => {
    if (!href || /^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(href)) return undefined;
    try {
      const base = new URL(input.metadata?.path ?? "", "https://qualified.invalid/");
      const path = decodeURIComponent(new URL(href, base).pathname).replace(/^\//, "");
      return workspace.environment.files.find(file => file.path === path);
    } catch { return undefined; }
  };
  return <div className="markdown-preview searchable-content"><Markdown remarkPlugins={[remarkGfm]} skipHtml components={{
    a: ({ href, children }) => { const file = qualifiedFile(href); return file ? <button className="markdown-link" onClick={() => actions.openSurface(fileInput(workspace, file.id))}>{children}</button> : <span className="markdown-reference" title={href ? `${href} · Not exposed as navigable Case material` : "Unsafe or unavailable link"}>{children}</span>; },
    img: ({ src, alt }) => { const file = qualifiedFile(typeof src === "string" ? src : undefined); return file ? <button className="markdown-link" onClick={() => actions.openSurface(fileInput(workspace, file.id))}>Image: {alt || file.path}</button> : <span className="markdown-reference">Image: {alt || "Not exposed as Case material"}</span>; },
  }}>{content}</Markdown></div>;
}
