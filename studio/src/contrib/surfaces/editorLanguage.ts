import type { Extension } from "@codemirror/state";

export type EditorLanguage = "markdown" | "json" | "toml" | "yaml" | "xml" | "rust" | "typescript" | "javascript" | "python" | "shell" | "c" | "cpp" | "css" | "html" | "plain";

export function detectEditorLanguage(path: string | undefined, mediaType: string | undefined): EditorLanguage {
  const media = mediaType?.split(";", 1)[0].toLocaleLowerCase();
  if (media === "text/markdown") return "markdown";
  if (media === "application/json" || media === "application/ld+json") return "json";
  if (media === "application/toml") return "toml";
  if (media === "application/yaml" || media === "application/x-yaml") return "yaml";
  if (media === "application/xml" || media === "text/xml" || media === "image/svg+xml") return "xml";
  if (media === "text/css") return "css";
  if (media === "text/html") return "html";
  const name = path?.toLocaleLowerCase() ?? "";
  if (/\.(md|mdown|markdown)$/.test(name)) return "markdown";
  if (/\.(json|jsonc|jsonl)$/.test(name)) return "json";
  if (/\.toml$/.test(name) || /(^|\/)cargo\.lock$/.test(name)) return "toml";
  if (/\.(yaml|yml)$/.test(name)) return "yaml";
  if (/\.(xml|svg)$/.test(name)) return "xml";
  if (/\.rs$/.test(name)) return "rust";
  if (/\.(tsx?|mts|cts)$/.test(name)) return "typescript";
  if (/\.(jsx?|mjs|cjs)$/.test(name)) return "javascript";
  if (/\.py$/.test(name)) return "python";
  if (/\.(sh|bash|zsh)$/.test(name)) return "shell";
  if (/\.(cc|cpp|cxx|hh|hpp|hxx)$/.test(name)) return "cpp";
  if (/\.(c|h)$/.test(name)) return "c";
  if (/\.css$/.test(name)) return "css";
  if (/\.html?$/.test(name)) return "html";
  return "plain";
}

export async function loadEditorLanguage(language: EditorLanguage): Promise<Extension> {
  switch (language) {
    case "markdown": return (await import("@codemirror/lang-markdown")).markdown();
    case "json": return (await import("@codemirror/lang-json")).json();
    case "yaml": return (await import("@codemirror/lang-yaml")).yaml();
    case "xml": return (await import("@codemirror/lang-xml")).xml();
    case "rust": return (await import("@codemirror/lang-rust")).rust();
    case "typescript": return (await import("@codemirror/lang-javascript")).javascript({ typescript: true, jsx: true });
    case "javascript": return (await import("@codemirror/lang-javascript")).javascript({ jsx: true });
    case "python": return (await import("@codemirror/lang-python")).python();
    case "css": return (await import("@codemirror/lang-css")).css();
    case "html": return (await import("@codemirror/lang-html")).html();
    case "c": return (await import("@codemirror/lang-cpp")).cpp();
    case "cpp": return (await import("@codemirror/lang-cpp")).cpp();
    case "toml": {
      const [{ StreamLanguage }, { toml }] = await Promise.all([import("@codemirror/language"), import("@codemirror/legacy-modes/mode/toml")]);
      return StreamLanguage.define(toml);
    }
    case "shell": {
      const [{ StreamLanguage }, { shell }] = await Promise.all([import("@codemirror/language"), import("@codemirror/legacy-modes/mode/shell")]);
      return StreamLanguage.define(shell);
    }
    default: return [];
  }
}
