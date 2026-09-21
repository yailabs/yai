import { useEffect, useRef } from "react";
import { basicSetup } from "codemirror";
import { defaultKeymap, historyKeymap, indentWithTab, redo, selectAll, undo } from "@codemirror/commands";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { openSearchPanel, searchKeymap } from "@codemirror/search";
import { EditorState, StateEffect, type Extension } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { tags } from "@lezer/highlight";
import type { SurfaceBufferSnapshot, SurfaceBufferService } from "../../workbench/surface/buffers";
import { detectEditorLanguage, loadEditorLanguage } from "./editorLanguage";

const yaiEditorTheme = EditorView.theme({
  "&": { height: "100%", backgroundColor: "#0d0e10", color: "#d5d9df", fontSize: "13px" },
  ".cm-scroller": { fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace", lineHeight: "1.6", overflow: "auto" },
  ".cm-content": { padding: "12px 0 80px", caretColor: "#78a9ff" },
  ".cm-line": { padding: "0 18px 0 8px" },
  ".cm-gutters": { backgroundColor: "#0d0e10", color: "#59616b", border: "none", paddingLeft: "5px" },
  ".cm-activeLine, .cm-activeLineGutter": { backgroundColor: "rgba(120,169,255,.055)" },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": { backgroundColor: "rgba(82,126,184,.34) !important" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "#78a9ff" },
  ".cm-search": { backgroundColor: "#181a1e", borderTop: "1px solid rgba(255,255,255,.08)", color: "#d5d9df" },
  ".cm-search input": { backgroundColor: "#0a0b0d", color: "#f1f2f4", border: "1px solid rgba(255,255,255,.1)", borderRadius: "4px" },
  ".cm-search button": { backgroundColor: "#252a31", color: "#d5d9df", border: "0", borderRadius: "4px" },
  ".cm-foldPlaceholder": { backgroundColor: "#202228", border: "none", color: "#7e838d" },
}, { dark: true });

const yaiSyntax = HighlightStyle.define([
  { tag: tags.comment, color: "#68717d", fontStyle: "italic" },
  { tag: [tags.keyword, tags.modifier, tags.operatorKeyword], color: "#85a9df" },
  { tag: [tags.typeName, tags.className, tags.namespace], color: "#7fc0b5" },
  { tag: [tags.function(tags.variableName), tags.function(tags.propertyName), tags.labelName], color: "#d6bd7a" },
  { tag: [tags.propertyName, tags.attributeName], color: "#a8bdd8" },
  { tag: [tags.string, tags.special(tags.string)], color: "#8fc59f" },
  { tag: [tags.number, tags.bool, tags.null], color: "#c69ad9" },
  { tag: [tags.punctuation, tags.bracket], color: "#9299a4" },
  { tag: [tags.heading, tags.strong], color: "#d9e2ee", fontWeight: "700" },
  { tag: [tags.link, tags.url], color: "#79aeea", textDecoration: "underline" },
  { tag: [tags.monospace, tags.processingInstruction], color: "#d1a88c" },
  { tag: tags.invalid, color: "#ef8c94", textDecoration: "underline wavy" },
]);

const editorSessions = new WeakMap<SurfaceBufferService, Map<string, { state: EditorState; scrollTop: number; scrollLeft: number }>>();

interface Props {
  buffers: SurfaceBufferService;
  identity: string;
  title: string;
  path?: string;
  mediaType?: string;
  snapshot: SurfaceBufferSnapshot;
  readOnly: boolean;
  onChange(value: string): void;
  onCommand?(handler: (name: string) => void): () => void;
}

export default function CodeEditorSurface({ buffers, identity, title, path, mediaType, snapshot, readOnly, onChange, onCommand }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  useEffect(() => {
    let disposed = false;
    let instance: EditorView | undefined;
    const language = detectEditorLanguage(path, mediaType);
    void loadEditorLanguage(language).then((languageExtension) => {
      if (disposed || !host.current) return;
      const extensions: Extension[] = [
        basicSetup,
        yaiEditorTheme,
        syntaxHighlighting(yaiSyntax),
        languageExtension,
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, indentWithTab]),
        EditorState.readOnly.of(readOnly),
        EditorView.contentAttributes.of({ "aria-label": `Edit ${title}`, "data-editor-language": language, spellcheck: "false" }),
        EditorView.updateListener.of((update) => { if (update.docChanged) onChangeRef.current(update.state.doc.toString()); }),
      ];
      const nonce = document.querySelector<HTMLStyleElement>("#studio-style-nonce")?.nonce;
      if (nonce) extensions.push(EditorView.cspNonce.of(nonce));
      const sessions = editorSessions.get(buffers) ?? new Map();
      editorSessions.set(buffers, sessions);
      const saved = sessions.get(identity);
      const state = saved?.state.doc.toString() === snapshot.value
        ? saved.state.update({ effects: StateEffect.reconfigure.of(extensions) }).state
        : EditorState.create({ doc: snapshot.value, extensions });
      instance = new EditorView({ state, parent: host.current });
      if (saved) { instance.scrollDOM.scrollTop = saved.scrollTop; instance.scrollDOM.scrollLeft = saved.scrollLeft; }
      view.current = instance;
    });
    return () => {
      disposed = true;
      if (instance) editorSessions.get(buffers)?.set(identity, { state: instance.state, scrollTop: instance.scrollDOM.scrollTop, scrollLeft: instance.scrollDOM.scrollLeft });
      instance?.destroy();
      if (view.current === instance) view.current = null;
    };
  }, [buffers, identity, mediaType, path, readOnly, snapshot.source.key, title]);

  useEffect(() => {
    const current = view.current;
    if (!current || current.state.doc.toString() === snapshot.value) return;
    current.dispatch({ changes: { from: 0, to: current.state.doc.length, insert: snapshot.value } });
  }, [snapshot.value]);

  useEffect(() => onCommand?.((name) => {
    const current = view.current;
    if (!current) return;
    if (name === "undo") undo(current);
    if (name === "redo") redo(current);
    if (name === "selectAll") selectAll(current);
    if (name === "replace" || name === "find") openSearchPanel(current);
  }), [onCommand]);

  return <div className="code-editor" ref={host} data-language={detectEditorLanguage(path, mediaType)} />;
}
