export interface KnowledgeRequest { case_id: string; source: string | null; revision: string | null; max_units: number }
export interface KnowledgeUnit { id: string; source: string; text: string; kind: string; posture: string }
export interface KnowledgeView { id: string; case_id: string; units: KnowledgeUnit[]; sources: Array<{ id: string; path: string }>; relations: Array<{ id: string; from: string; to: string; kind: string }> }
export interface KnowledgeSearchResult { view: KnowledgeView; hits: Array<{ document_id: string; score_micros: number; matched_terms: string[] }> }
export interface KnowledgeResolveResult { view: KnowledgeView; unit: KnowledgeUnit }
export interface KnowledgeNavigationResult { view_ref: string; navigation: string }
