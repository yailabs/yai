# Pre-Pilot Report (Law + YAI Runtime)
Date: 2026-03-10
Scope: preparazione test case reali pre-pilot per Ecomidia/Fabio + verticale transazionale/finanziario.

## 1) Executive summary
Il setup attuale e' sufficiente per iniziare pre-test manuali reali su CLI/runtime per due filoni:
- Filone A (Fabio/Ecomidia): PA, BI/Analytics, Cloud Storage, Cyber Security.
- Filone B (nuovo investimento): transazioni/finanza/banking.

Baseline tecnica oggi:
- Law pack embedded caricato e risolto runtime-side (families + specializations + overlays).
- Enforcement chiuso su record persistiti con authority gate e outcome/linkage.
- Read path DB-first attivo su superfici inspect/query principali.
- Graph materialization attiva e read surfaces baseline disponibili.
- Data lifecycle scaffolding presente (retention/archive/pruning/compaction layer in progress).

## 2) Stato reale repository `law` (embedded)
Path base: `embedded/law`

### 2.1 Control families disponibili
Fonte: `embedded/law/control-families/index/families.index.json`

- D1 `digital` (status: `active-wave-1-pillar`, maturity: `structured`)
- D2 `physical` (status: `active`, maturity: `seed`)
- D3 `biological` (status: `active`, maturity: `seed`)
- D4 `social-institutional` (status: `active`, maturity: `seed`)
- D5 `economic` (status: `active-wave-1`, maturity: `dense`)
- D6 `operational` (status: `active`, maturity: `seed`)
- D7 `cognitive-cultural` (status: `active`, maturity: `seed`)
- D8 `scientific` (status: `active-wave-1-pillar`, maturity: `structured`)
- D9 `environmental-climatological` (status: `active`, maturity: `seed`)

### 2.2 Domain specializations (indice e maturity)
Fonte: `embedded/law/specializations/index/specializations.index.json`

Conteggi correnti:
- digital: 6
- economic: 9
- scientific: 6

Stato correnti:
- `active`: 7
- `active-planned`: 9
- `active-seed`: 3
- `draft-seed`: 2

#### Digital
- `network-egress` (active)
- `remote-publication` (active-planned)
- `external-commentary` (active-planned)
- `artifact-distribution` (active-planned)
- `remote-retrieval` (active-planned)
- `digital-sink-control` (active-planned)

#### Economic
- `payments` (active)
- `transfers` (active)
- `settlements` (active)
- `fraud-risk-controls` (active)
- `treasury` (active-seed)
- `accounting-integrity` (active-seed)
- `credit-exposure` (active-seed)
- `invoicing` (draft-seed)
- `pricing-and-quoting` (draft-seed)

#### Scientific
- `parameter-governance` (active)
- `reproducibility-control` (active)
- `experiment-configuration` (active-planned)
- `black-box-evaluation` (active-planned)
- `dataset-integrity` (active-planned)
- `result-publication-control` (active-planned)

### 2.3 Overlay pack disponibili
Fonti:
- `embedded/law/overlays/regulatory/index/regulatory.index.json`
- `embedded/law/overlays/sector/index/sector.index.json`
- `embedded/law/overlays/contextual/index/contextual.index.json`

Regulatory attivi:
- `gdpr-eu`
- `ai-act`
- `retention-governance`
- `security-supply-chain`

Sector:
- `sector.finance` (active)
- `sector.healthcare` (active-seed)
- `sector.public-sector` (active-seed)
- `sector.manufacturing` (seed)

Contextual:
- `context.organization`, `context.workspace`, `context.session`, `context.experimental` (seed)

## 3) Cosa e' gia' implementato in `yai` e usabile ora

### 3.1 Enforcement + authority su flusso runtime vero
- authority command gate e policy gate integrati nel finalize enforcement.
- output enforcement persistito in classi DB:
  - `enforcement_outcome`
  - `enforcement_linkage`
- authority persistito in:
  - `authority`
  - `authority_resolution`

Riferimenti codice principali:
- `lib/core/enforcement/enforcement.c`
- `lib/core/authority/authority_registry.c`
- `lib/core/authority/identity.c`
- `lib/core/session/session.c`
- `lib/core/session/session_utils.c`
- `lib/data/records/event_records.c`

### 3.2 DB stack operativo (LMDB + DuckDB + Redis hook)
- LMDB: append/query base per record classes.
- DuckDB: mirror append/count, attivo se libreria presente.
- Redis: mirror refs/materialization, attivo se `YAI_REDIS_ENABLE=1`.

Riferimenti:
- `lib/data/store/file_store.c` (LMDB + Redis mirror)
- `lib/data/store/duckdb_store.c`
- `lib/graph/materialization/from_runtime_records.c` (Redis emit graph)
- `Makefile` (feature gating compile-time)

Nota stato dipendenze locale:
- DuckDB risulta gia' presente in `/opt/homebrew/opt/duckdb`.

### 3.3 Query surfaces runtime (DB-first)
- family enforcement/authority/governance/events/evidence/artifacts/graph gia' interrogabili.
- enforcement ora include anche `authority_requirement_coverage` per requisito.

Riferimento:
- `lib/core/session/utils/session_utils_surface_views.inc.c`

### 3.4 Graph baseline
- materializzazione da runtime records attiva.
- summary workspace disponibile.
- integrazione lifecycle in corso (core vs tail da completare in DP-15B hard mode).

## 4) Gap reali ancora aperti (da trattare prima del pilot esteso)
- Alcuni namespace/naming legacy (`brain_*`/`mind_*`) ancora presenti in parti runtime/graph.
- Alcune directory nuove hanno ancora copertura funzionale parziale (non vuote ma da densificare).
- Redis e' agganciato come mirror/event bus leggero: manca ancora disciplina operativa completa (health policy, retry, backpressure).
- Per graph query/materialization serve ulteriore hardening su coverage cross-scenario e noised-tail governance.
- Enforcement e authority devono essere ulteriormente verticalizzati per policy class-specific (finance/PA risk modes).

## 5) Matrice test case pre-pilot (manual CLI) - Fabio/Ecomidia

## 5.1 PA data governance + cyber perimeter
Caso A1: output amministrativo verso sink esterno non trusted
- Family/specialization: `digital.network-egress`
- Overlay: `gdpr-eu`, `security-supply-chain`, `sector.public-sector`, `context.workspace`
- Obiettivo: verificare deny/review quando manca contract authority o sink attestato.
- Evidenze attese: decision trace, sink attestation gap, authority scope, review chain.

Caso A2: pubblicazione documento istituzionale con policy rigidita'
- Family/specialization: `digital.remote-publication` + `digital-sink-control`
- Overlay: `gdpr-eu`, `retention-governance`, `sector.public-sector`
- Obiettivo: test review escalation su destinazioni non conformi.

## 5.2 BI/Analytics
Caso B1: modifica parametri modello di reporting critico
- Family/specialization: `scientific.parameter-governance`
- Overlay: `ai-act`, `gdpr-eu`, `sector.finance` (se dati finanziari), `context.session`
- Obiettivo: review_required su drift parametri sensibili, deny su lock break senza contract.

Caso B2: verifica dataset integrity prima di dashboard publish
- Family/specialization: `scientific.dataset-integrity`
- Overlay: `ai-act`, `gdpr-eu`, `retention-governance`
- Obiettivo: deny/quarantine se mancano lineage/attestation.

## 5.3 Cloud storage / retrieval / distribution
Caso C1: retrieval da source remota non trusted
- Family/specialization: `digital.remote-retrieval`
- Overlay: `security-supply-chain`, `gdpr-eu`
- Obiettivo: review/deny su provenance gap.

Caso C2: artifact distribution cross-workspace
- Family/specialization: `digital.artifact-distribution`
- Overlay: `retention-governance`, `context.organization`
- Obiettivo: controllo leakage boundary e policy di destinazione.

## 5.4 Cyber security operativo
Caso D1: anomalia egress + escalation
- Family/specialization: `digital.network-egress` + `digital-sink-control`
- Overlay: `security-supply-chain`, `sector.public-sector`
- Obiettivo: quarantena controllata, evidenza completa, enforce path non bypassabile.

## 6) Matrice test case pre-pilot (transazionale/finanziario)

## 6.1 Pagamenti
Caso F1: payment authorization con dual control
- Specialization: `economic.payments`
- Overlay: `sector.finance`, `gdpr-eu`, `retention-governance`
- Obiettivo: nessun self-approval, review se high impact.
- Artefatti attesi: authority decision, approval_chain, overlay hits.

## 6.2 Transfers
Caso F2: transfer cross-scope con rischio frode
- Specialization: `economic.transfers`
- Overlay: `sector.finance`, `security-supply-chain`
- Obiettivo: review_required/deny quando authority contract incompleto.

## 6.3 Settlements
Caso F3: settlement con mismatch evidenze
- Specialization: `economic.settlements`
- Overlay: `sector.finance`, `retention-governance`
- Obiettivo: blocco su gap evidenze obbligatorie.

## 6.4 Fraud controls
Caso F4: operazione high-impact marcata fraud-risk
- Specialization: `economic.fraud-risk-controls`
- Overlay: `sector.finance`, `gdpr-eu`
- Obiettivo: review escalation automatica e tracciata.

## 6.5 Credit/treasury (seed)
Caso F5: esposizione credito fuori soglia
- Specialization: `economic.credit-exposure` (seed)
- Obiettivo: validare baseline + requisiti minimi evidence.

Caso F6: movimentazione treasury sensibile
- Specialization: `economic.treasury` (seed)
- Obiettivo: verificare readiness semantica e gap prima di pilot pieno.

## 7) Procedura manuale CLI consigliata (pre-pilot)
Base operativa attuale (runtime control-call):
1. `yai.workspace.create`
2. `yai.workspace.set`
3. `yai.workspace.domain_set --family <...> --specialization <...>`
4. `yai.workspace.run <scenario_action ...>`
5. query verifica:
- `yai.workspace.query enforcement`
- `yai.workspace.query authority`
- `yai.workspace.query governance`
- `yai.workspace.events.tail`
- `yai.workspace.evidence.list`
- `yai.workspace.graph.summary`

Checklist pass/fail minima per ogni caso:
- enforcement outcome presente + linkage presente
- authority decision coerente con policy/overlay
- evidence requirements soddisfatti o gap esplicito
- read path DB-first confermato
- graph summary coerente con decision/evidence/artifact relations
- nessun leakage cross-workspace

## 8) Cosa puoi dire a Fabio settimana prossima (statement concreto)
- Abbiamo gia' avviato test manuali reali su domini Ecomidia (PA, BI/analytics, cloud, cyber) usando specialization+overlay concreti del pack law.
- In parallelo abbiamo aperto test transazionali finance (payments/transfers/settlements/fraud) con authority+enforcement persistiti e query DB-first.
- Non stiamo facendo demo sintetiche: stiamo validando flussi con evidence trail, authority gates, enforcement outcome/linkage e graph summary runtime.

## 9) Piano operativo immediato (7 giorni)
1. Eseguire 2 casi per verticale (A/B/C/D + F1..F4) con log comparabili.
2. Salvare per ogni run: input, effect, authority decision, evidence gap/hits, graph summary.
3. Marcare gap hardening per DP-15B esteso (compaction/pruning/anti-leakage).
4. Preparare pacchetto demo con 3 storie end-to-end:
- PA cyber egress control
- BI parameter governance + dataset integrity
- Finance transfer/fraud escalation

## 10) Allegato: file law chiave da cui partire (per chatbot)
- `embedded/law/control-families/index/families.index.json`
- `embedded/law/specializations/index/specializations.index.json`
- `embedded/law/runtime.entrypoints.json`
- `embedded/law/overlays/regulatory/index/regulatory.index.json`
- `embedded/law/overlays/sector/index/sector.index.json`
- `embedded/law/overlays/contextual/index/contextual.index.json`
- `embedded/law/specializations/payments/*`
- `embedded/law/specializations/transfers/*`
- `embedded/law/specializations/settlements/*`
- `embedded/law/specializations/fraud-risk-controls/*`
- `embedded/law/specializations/network-egress/*`
- `embedded/law/specializations/parameter-governance/*`
- `embedded/law/specializations/dataset-integrity/*`
