import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

/** Model links only navigate objects already disclosed in this Case projection. */
export default function NarrativeText({ text, references, inspect }: { text: string; references: readonly string[]; inspect(ref: string): void }) {
  const known = new Set(references);
  return <ReactMarkdown remarkPlugins={[remarkGfm]} skipHtml urlTransform={value => known.has(value) ? value : ""}
    components={{
      a: ({href, children}) => href && known.has(href) ? <button className="object-link" onClick={() => inspect(href)}>{children}</button> : <span>{children}</span>,
      img: () => null,
    }}>{text}</ReactMarkdown>;
}
