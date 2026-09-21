import { useEffect, useMemo, useRef, useState } from "react";
import type { MaterialBody } from "../../clients/presentation";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { SurfaceBufferSource } from "../../workbench/surface/buffers";
import { MaterialReadFence, materialIdentity, validateMaterialRead, type MaterialIdentityToken } from "./materialIdentity";

export function materialText(body: MaterialBody) {
  if (body.kind === "text") return body.content;
  if (body.kind === "diff") return body.lines.map((line) => `${line.change === "add" ? "+ " : line.change === "remove" ? "- " : "  "}${line.text}`).join("\n");
  if (body.kind === "document") return [body.title, body.intro, ...body.sections.flatMap((section) => [section.title, section.body, ...(section.points ?? [])])].join("\n\n");
  return undefined;
}

export function expectedMaterialIdentity({ workspace, input }: Pick<SurfaceRendererProps, "workspace" | "input">): MaterialIdentityToken | undefined {
  const file = workspace.environment.files.find((item) => item.id === input.objectRef);
  if (!file || !input.objectRef) return undefined;
  return materialIdentity({
    objectRef: input.objectRef,
    caseRef: workspace.case.case_ref,
    sourceRef: file.source_ref,
    revisionRef: file.revision_ref,
    path: file.path,
    digest: file.digest,
    generation: workspace.case.generation,
    mediaType: file.media_type,
    bytes: file.bytes,
  });
}

export function bufferSource(identity: MaterialIdentityToken): SurfaceBufferSource {
  return {
    key: identity.key,
    caseRef: identity.caseRef,
    objectRef: identity.objectRef,
    sourceRef: identity.sourceRef,
    revisionRef: identity.revisionRef,
    path: identity.path,
    digest: identity.digest,
    generation: identity.generation,
  };
}

interface ReadableMaterialState {
  identityKey?: string;
  content?: string;
  source?: SurfaceBufferSource;
  error?: string;
  loading: boolean;
}

export function useReadableMaterial(props: SurfaceRendererProps): ReadableMaterialState {
  const { workspace, input, readMaterial } = props;
  const fixture = workspace.presentation.materials?.find((item) => item.id === input.objectRef)
    ?? workspace.presentation.materials?.find((item) => item.path === input.metadata?.path);
  const authored = fixture ? materialText(fixture.body) : undefined;
  const expected = useMemo(
    () => expectedMaterialIdentity({ workspace, input }),
    [input, workspace],
  );
  const activeKey = expected?.key ?? `authored:${workspace.case.case_ref}:${input.identity}:${workspace.case.generation}`;
  const fence = useRef(new MaterialReadFence());
  const [state, setState] = useState<ReadableMaterialState>({ identityKey: activeKey, loading: authored === undefined });

  useEffect(() => {
    if (authored !== undefined) {
      const source = expected ? bufferSource(expected) : {
        key: activeKey,
        caseRef: workspace.case.case_ref,
        objectRef: input.objectRef ?? input.identity,
        sourceRef: input.metadata?.sourceRef ?? input.objectRef ?? input.identity,
        revisionRef: input.metadata?.revisionRef ?? `fixture:${workspace.case.generation}`,
        path: input.metadata?.path ?? fixture?.path ?? input.title,
        digest: input.metadata?.digest ?? `fixture:${activeKey}`,
        generation: workspace.case.generation,
      };
      setState({ identityKey: activeKey, content: authored, source, loading: false });
      return;
    }
    if (!expected) {
      setState({ identityKey: activeKey, error: "No qualified file revision is attached to this Surface.", loading: false });
      return;
    }
    const request = fence.current.begin(expected.key);
    setState({ identityKey: expected.key, loading: true });
    void readMaterial({
      case_ref: expected.caseRef,
      source_ref: expected.sourceRef,
      revision_ref: expected.revisionRef,
      path: expected.path,
      expected_generation: expected.generation,
    }).then((result) => {
      if (!fence.current.accepts(request, expected.key)) return;
      if (result.result_state !== "success" || !result.data) {
        setState({ identityKey: expected.key, error: result.error?.safe_message ?? `Material read ${result.result_state}.`, loading: false });
        return;
      }
      const mismatch = validateMaterialRead(expected, result.data);
      if (mismatch) {
        setState({ identityKey: expected.key, error: `Exact material identity mismatch. ${mismatch}`, loading: false });
        return;
      }
      if (result.data.encoding !== "utf-8") {
        setState({ identityKey: expected.key, error: "This exact revision is binary and requires a trusted binary renderer.", loading: false });
        return;
      }
      setState({ identityKey: expected.key, content: result.data.content, source: bufferSource(expected), loading: false });
    });
    return () => fence.current.cancel(request);
  }, [activeKey, authored, expected, fixture?.path, input.identity, input.metadata, input.objectRef, input.title, readMaterial, workspace.case.case_ref, workspace.case.generation]);

  return state.identityKey === activeKey ? state : { identityKey: activeKey, loading: true };
}
