import type { MaterialReadProjection } from "../../clients/live";

export interface MaterialFileIdentity {
  objectRef: string;
  caseRef: string;
  sourceRef: string;
  revisionRef: string;
  path: string;
  digest: string;
  generation: number;
  mediaType: string;
  bytes: number;
}

export interface MaterialIdentityToken extends MaterialFileIdentity {
  key: string;
}

export function materialIdentity(value: MaterialFileIdentity): MaterialIdentityToken {
  return {
    ...value,
    key: [
      value.caseRef,
      value.objectRef,
      value.sourceRef,
      value.revisionRef,
      value.path,
      value.digest,
      String(value.generation),
      value.mediaType,
      String(value.bytes),
    ].map((part) => `${part.length}:${part}`).join("|"),
  };
}

export function validateMaterialRead(
  expected: MaterialIdentityToken,
  actual: MaterialReadProjection,
): string | undefined {
  const fields: ReadonlyArray<[string, string | number, string | number]> = [
    ["case_ref", expected.caseRef, actual.case_ref],
    ["source_ref", expected.sourceRef, actual.source_ref],
    ["revision_ref", expected.revisionRef, actual.revision_ref],
    ["path", expected.path, actual.path],
    ["digest", expected.digest, actual.digest],
    ["generation", expected.generation, actual.generation],
    ["media_type", expected.mediaType, actual.media_type],
    ["bytes", expected.bytes, actual.bytes],
  ];
  for (const [field, wanted, received] of fields) {
    if (wanted !== received) {
      return `material_identity_mismatch:${field}:expected=${wanted}:actual=${received}`;
    }
  }
  if (actual.encoding === "utf-8" && new TextEncoder().encode(actual.content).byteLength !== actual.bytes) {
    return `material_identity_mismatch:content_bytes:expected=${actual.bytes}:actual=${new TextEncoder().encode(actual.content).byteLength}`;
  }
  return undefined;
}

export class MaterialReadFence {
  private sequence = 0;
  private current?: { sequence: number; key: string };

  begin(key: string) {
    const request = { sequence: ++this.sequence, key };
    this.current = request;
    return request;
  }

  accepts(request: { sequence: number; key: string }, activeKey: string) {
    return this.current?.sequence === request.sequence
      && this.current.key === request.key
      && request.key === activeKey;
  }

  cancel(request: { sequence: number; key: string }) {
    if (this.current?.sequence === request.sequence) this.current = undefined;
  }
}
