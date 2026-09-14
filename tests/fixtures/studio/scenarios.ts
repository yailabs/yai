import type { WorkspacePresentation } from "../../../studio/src/clients/presentation";

type BaseWorkspacePresentation = Omit<WorkspacePresentation, "information">;

// Authored synthetic visual scenarios. No private Case export, legal advice,
// actual provider measurements, filesystem observations, or execution receipts.
const provenance =
  "Authored synthetic scenario · presentation fixture v1 · no live Case or runtime";
export const scenarios: readonly BaseWorkspacePresentation[] = [
  {
    fixture: {
      id: "ordinary",
      label: "Ordinary · Contract review",
      provenance,
    },
    case: {
      label: "Contract review",
      reference: "CASE / 014",
      context: "Northline / Services agreement",
      purpose: "Bring the open questions into one reviewable agreement.",
      currentWork: "Prepare the discussion brief",
      participant: "elena",
    },
    participants: [
      {
        id: "elena",
        name: "Elena Rossi",
        initials: "ER",
        kind: "Human",
        role: "Case owner",
      },
      {
        id: "marco",
        name: "Marco Costa",
        initials: "MC",
        kind: "Human",
        role: "Commercial review",
      },
      {
        id: "analyst",
        name: "Review assistant",
        initials: "RA",
        kind: "AI participant",
        role: "Document comparison",
      },
    ],
    provider: {
      name: "OpenAI-compatible",
      location: "Cloud",
      model: "Review model · example",
      posture: "Offline example",
      note: "Illustrative provider context. No credentials, connection or qualification evidence.",
    },
    initial: {
      tabs: ["brief", "agreement"],
      active: "brief",
      bottom: "Evidence",
    },
    materials: [
      {
        id: "brief",
        name: "Discussion brief",
        path: "Artifacts / discussion-brief.md",
        category: "artifact",
        format: "Document",
        provenance: "Synthetic artifact · revision 3 · Review assistant",
        changed: true,
        body: {
          kind: "document",
          eyebrow: "DISCUSSION BRIEF / REVISION 03",
          title: "A shared starting point.",
          intro:
            "Three questions to settle before the services agreement moves to final review. Prepared from the agreement, the addendum and our meeting notes.",
          sections: [
            {
              title: "01  Scope & delivery",
              body: "The agreement describes the initial delivery, but the acceptance window is still open. Our notes ask for ten working days and a named reviewer.",
              points: [
                "Confirm who accepts each milestone.",
                "Keep changes to scope separate from routine support.",
              ],
            },
            {
              title: "02  Data at the end of the contract",
              body: "The addendum describes return or deletion on request. The service schedule does not name the export format or the person responsible for requesting it.",
            },
            {
              title: "03  Next conversation",
              body: "Elena and Marco will compare these questions with the supplier. This brief records discussion points; it does not approve terms or change the source agreement.",
            },
          ],
          references: ["agreement", "addendum", "notes"],
        },
      },
      {
        id: "agreement",
        name: "Services agreement",
        path: "Sources / services-agreement.pdf",
        category: "source",
        format: "Text excerpt",
        provenance: "Synthetic source excerpt · section 4 · no PDF renderer",
        body: {
          kind: "document",
          eyebrow: "SOURCE EXCERPT / SERVICES AGREEMENT",
          title: "Delivery & acceptance",
          intro:
            "Section 4 · illustrative text for visual review. The original document is not loaded.",
          sections: [
            {
              title: "4.1 Delivery",
              body: "The supplier will deliver the materials listed in the service schedule at each agreed milestone.",
            },
            {
              title: "4.2 Acceptance",
              body: "The parties will agree an acceptance period and designate a reviewer before delivery begins.",
            },
            {
              title: "Reading context",
              body: "This is authored fixture content. It makes no legal assessment and carries no authority over a real Case.",
            },
          ],
        },
      },
      {
        id: "addendum",
        name: "Data addendum",
        path: "Sources / data-addendum.pdf",
        category: "source",
        format: "Text excerpt",
        provenance: "Synthetic source excerpt · section 8",
        body: {
          kind: "document",
          eyebrow: "SOURCE EXCERPT / DATA ADDENDUM",
          title: "Return of customer data",
          intro: "The two documents leave a practical handoff question.",
          sections: [
            {
              title: "8.1 End of service",
              body: "On written request, the supplier will return or delete customer data. The export format and responsible contact are to be confirmed.",
            },
          ],
        },
      },
      {
        id: "notes",
        name: "Meeting notes",
        path: "Sources / client-notes.md",
        category: "source",
        format: "Document",
        provenance: "Synthetic human notes · Elena Rossi",
        body: {
          kind: "document",
          eyebrow: "MEETING NOTES / 12 SEPTEMBER",
          title: "What we need to clarify",
          intro: "Notes for the next supplier conversation.",
          sections: [
            {
              title: "Delivery",
              body: "Ask for ten working days to review each milestone. Marco will identify a reviewer.",
            },
            {
              title: "Handoff",
              body: "Ask for a documented export format and a named contact for the data return.",
            },
          ],
        },
      },
      {
        id: "work",
        name: "Review preparation",
        path: "Work / review-preparation",
        category: "work",
        format: "Work",
        provenance: "Synthetic work snapshot · no live workflow",
        body: {
          kind: "work",
          title: "Prepare the discussion",
          description:
            "Organize source questions before a human review. No contract changes are authorized by this fixture.",
        },
      },
      {
        id: "provider",
        name: "Provider context",
        path: "Providers / review-model",
        category: "provider",
        format: "Provider",
        provenance: "Synthetic provider presentation",
        body: { kind: "provider", title: "Review model" },
      },
    ],
    activity: [
      {
        id: "n1",
        kind: "notice",
        time: "09:12",
        title: "YAI · Source context",
        text: "Three source excerpts are included in this fixture.",
      },
      {
        id: "t1",
        kind: "turn",
        participant: "elena",
        time: "09:14",
        text: "Let’s keep the next conversation focused. Which points still need an answer from the supplier?",
      },
      {
        id: "t2",
        kind: "turn",
        participant: "analyst",
        time: "09:15",
        text: "The acceptance window and data handoff need clarification. I’ve brought those questions into the discussion brief, with the source excerpts alongside them.",
      },
      {
        id: "e1",
        kind: "execution",
        time: "09:15",
        title: "Artifact prepared",
        text: "Discussion brief · revision 3",
        material: "brief",
      },
      {
        id: "t3",
        kind: "turn",
        participant: "marco",
        time: "09:18",
        text: "I’ll confirm the milestone reviewer. Keep the data export question in the brief for Elena.",
      },
    ],
    executions: [
      {
        id: "EX-014",
        title: "Prepare discussion brief",
        detail: "Synthetic artifact preparation",
        posture: "completed",
        time: "09:15",
      },
    ],
    evidence: [
      {
        id: "EV-01",
        title: "Acceptance window",
        origin: "Services agreement · §4.2",
        detail: "Named period still to be agreed",
        material: "agreement",
      },
      {
        id: "EV-02",
        title: "Data return",
        origin: "Data addendum · §8.1",
        detail: "Export format not specified",
        material: "addendum",
      },
      {
        id: "EV-03",
        title: "Review expectations",
        origin: "Meeting notes",
        detail: "Ten working days requested",
        material: "notes",
      },
    ],
    problems: [],
    output: [
      "09:15  [fixture] Discussion brief prepared.",
      "09:15  [fixture] Source links retained in this presentation.",
      "No live execution output is attached.",
    ],
  },
  {
    fixture: {
      id: "developer",
      label: "Developer · Runtime qualification",
      provenance,
    },
    case: {
      label: "Runtime qualification",
      reference: "CASE / 027",
      context: "yailabs / yai",
      purpose: "Keep a completed result subject to current authority.",
      currentWork: "Review the result-reuse change",
      participant: "operator",
    },
    participants: [
      {
        id: "operator",
        name: "Francesco",
        initials: "FM",
        kind: "Human",
        role: "Operator",
      },
      {
        id: "code",
        name: "Code participant",
        initials: "CP",
        kind: "AI participant",
        role: "Implementation & tests",
      },
    ],
    provider: {
      name: "YVEX",
      location: "Local",
      model: "Qwen · example target",
      posture: "Offline example",
      note: "Authored model label, not a discovered deployment. Load, residency and device telemetry are unavailable.",
    },
    initial: {
      tabs: ["diff", "plan", "test"],
      active: "diff",
      bottom: "Output",
    },
    materials: [
      {
        id: "diff",
        name: "result_access.rs",
        path: "Files / src / result_access.rs",
        category: "artifact",
        format: "Diff",
        changed: true,
        provenance:
          "Illustrative code · proposed change · not repository contents",
        body: {
          kind: "diff",
          title: "Recheck before returning a result",
          description:
            "Recorded output does not carry permission into the next read.",
          before: "fixture / original",
          after: "fixture / proposed",
          lines: [
            {
              text: "// Illustrative code for design review; not executable YAI source.",
            },
            { text: "pub fn read_completed_result(" },
            { text: "    request: &Request," },
            { text: "    subject: &Participant," },
            { text: "    store: &Store," },
            { text: ") -> Result<VisibleResult> {" },
            { text: "    let recorded = store.result(&request.id)?;" },
            { text: "" },
            {
              text: "    // A completed attempt can reuse its recorded output.",
            },
            { text: "    return Ok(recorded.into());", change: "remove" },
            {
              text: "    // Reuse never inherits historical permission.",
              change: "add",
            },
            {
              text: "    let access = current_access(store, subject, request)?;",
              change: "add",
            },
            {
              text: "    access.require_disclosure(&recorded)?;",
              change: "add",
            },
            { text: "", change: "add" },
            { text: "    Ok(VisibleResult::from(recorded))", change: "add" },
            { text: "}" },
            { text: "" },
            { text: "#[test]" },
            { text: "fn revoked_access_does_not_reuse_a_completed_result() {" },
            { text: "    let case = fixture_with_completed_result();" },
            { text: "    case.revoke_access();" },
            {
              text: "    assert!(case.read_result().is_denied());",
              change: "add",
            },
            { text: "}" },
          ],
        },
      },
      {
        id: "plan",
        name: "Qualification notes",
        path: "Artifacts / qualification-notes.md",
        category: "artifact",
        format: "Document",
        provenance: "Synthetic technical notes · revision 2",
        body: {
          kind: "document",
          eyebrow: "QUALIFICATION / RESULT REUSE",
          title: "History is not permission.",
          intro:
            "The review focuses on the boundary between a recorded result and permission to disclose it.",
          sections: [
            {
              title: "Invariant",
              body: "Requalify current access before returning a cached result. Reuse must not dispatch a second provider request.",
            },
            {
              title: "Negative controls",
              body: "Exercise revocation at the same Case generation, participant substitution and a missing source. Preserve the completed attempt.",
              points: [
                "A denied reader receives no protected result.",
                "A permitted reader reuses the original result.",
              ],
            },
            {
              title: "Evidence posture",
              body: "The examples in this workspace are synthetic. They do not certify the current repository or a live provider.",
            },
          ],
          references: ["test", "source"],
        },
      },
      {
        id: "test",
        name: "result_reuse.test",
        path: "Files / tests / result_reuse.test",
        category: "artifact",
        format: "Document",
        provenance: "Synthetic test specification",
        body: {
          kind: "document",
          eyebrow: "TEST SPECIFICATION / NEGATIVE CONTROL",
          title: "Revoke, then retry.",
          intro:
            "A completed request must not carry old authority into a new read.",
          sections: [
            {
              title: "Given",
              body: "A participant can read a completed result. No new provider request is needed.",
            },
            {
              title: "When",
              body: "The relevant source permission is revoked without changing the stored result.",
            },
            {
              title: "Then",
              body: "The same read refuses. There is no redispatch, protected output or rewritten history.",
            },
          ],
        },
      },
      {
        id: "source",
        name: "yailabs/yai",
        path: "Sources / yailabs/yai",
        category: "source",
        format: "Repository",
        provenance: "Synthetic repository description · host files not read",
        body: {
          kind: "document",
          eyebrow: "SOURCE / REPOSITORY",
          title: "yailabs/yai",
          intro:
            "A repository source in the fixture Case. No checkout is read by Studio.",
          sections: [
            {
              title: "Relevant material",
              body: "The sample focuses on current result disclosure and result-reuse tests. Code displayed in this workspace is illustrative.",
            },
          ],
          references: ["diff", "test"],
        },
      },
      {
        id: "work",
        name: "Result reuse qualification",
        path: "Work / result-reuse",
        category: "work",
        format: "Work",
        provenance: "Synthetic work snapshot",
        body: {
          kind: "work",
          title: "Result reuse qualification",
          description:
            "Review one proposed change alongside its evidence. These test outcomes are static fixture values.",
        },
      },
      {
        id: "provider",
        name: "Local inference",
        path: "Providers / local-inference",
        category: "provider",
        format: "Provider",
        provenance: "Synthetic provider context",
        body: { kind: "provider", title: "Local inference" },
      },
    ],
    activity: [
      {
        id: "n1",
        kind: "notice",
        time: "14:06",
        title: "YAI · Work context",
        text: "Result reuse qualification · synthetic snapshot.",
      },
      {
        id: "t1",
        kind: "turn",
        participant: "operator",
        time: "14:07",
        text: "Keep the original result. Check current access before returning any of its content.",
      },
      {
        id: "t2",
        kind: "turn",
        participant: "code",
        time: "14:11",
        text: "The proposed change moves disclosure qualification ahead of reuse. The negative control covers a revoke at the same generation.",
      },
      {
        id: "e1",
        kind: "execution",
        time: "14:12",
        title: "Tests completed",
        text: "Result reuse · 3 passed, 0 failed (fixture)",
        material: "test",
      },
      {
        id: "t3",
        kind: "turn",
        participant: "operator",
        time: "14:14",
        text: "Good. I’ll review the diff with the retained output before deciding what to publish.",
      },
    ],
    executions: [
      {
        id: "EX-027",
        title: "Result reuse controls",
        detail: "3 tests · synthetic outcome",
        posture: "completed",
        time: "14:12",
      },
      {
        id: "EX-028",
        title: "Patch review",
        detail: "Operator review remains outstanding",
        posture: "waiting for review",
        time: "14:14",
      },
    ],
    evidence: [
      {
        id: "EV-11",
        title: "Revoked result read",
        origin: "Result reuse controls",
        detail: "Denied · zero redispatches (fixture)",
        material: "test",
      },
      {
        id: "EV-12",
        title: "Proposed access check",
        origin: "result_access.rs",
        detail: "Illustrative diff · not an applied edit",
        material: "diff",
      },
    ],
    problems: [
      {
        id: "P-01",
        severity: "warning",
        title: "Review is still outstanding",
        detail: "The proposed change has not been accepted.",
        material: "plan",
      },
    ],
    output: [
      "14:12:03  [fixture] result_reuse_controls",
      "  PASS  permitted_reader_reuses_original_result",
      "  PASS  revoked_access_refuses_before_disclosure",
      "  PASS  participant_substitution_refused",
      "",
      "3 passed · 0 failed · synthetic test output",
      "No process was started by Studio.",
    ],
  },
  {
    fixture: {
      id: "execution",
      label: "Execution · Release handoff",
      provenance,
    },
    case: {
      label: "Release handoff",
      reference: "CASE / 031",
      context: "Workspace / release-candidate",
      purpose: "Prepare the release material without publishing before review.",
      currentWork: "Assemble a reviewable handoff",
      participant: "operator",
    },
    participants: [
      {
        id: "operator",
        name: "Francesco",
        initials: "FM",
        kind: "Human",
        role: "Reviewer",
      },
      {
        id: "code",
        name: "Release participant",
        initials: "RP",
        kind: "AI participant",
        role: "Handoff preparation",
      },
    ],
    provider: {
      name: "OpenAI-compatible",
      location: "Local",
      model: "Release model · example",
      posture: "Offline example",
      note: "No connection is made. Execution states below are authored visual test inputs, not provider observations.",
    },
    initial: {
      tabs: ["work", "review", "manifest"],
      active: "work",
      bottom: "Executions",
    },
    materials: [
      {
        id: "work",
        name: "Release preparation",
        path: "Work / release-preparation",
        category: "work",
        format: "Work",
        provenance: "Frozen synthetic execution snapshot · 16:24",
        body: {
          kind: "work",
          title: "A reviewable handoff.",
          description:
            "Gather the release material, retain the validation output and leave publication with the reviewer.",
        },
      },
      {
        id: "review",
        name: "Publication review",
        path: "Reviews / publication-review",
        category: "artifact",
        format: "Review",
        provenance: "Synthetic review request · no decision controls",
        body: {
          kind: "document",
          eyebrow: "REVIEW REQUEST / PUBLICATION",
          title: "Before anything leaves the Case.",
          intro:
            "The prepared handoff is waiting for a human decision. This fixture cannot approve or publish it.",
          sections: [
            {
              title: "Proposed consequence",
              body: "Publish the prepared release material to the designated destination only after current authorization and review.",
            },
            {
              title: "Unresolved evidence",
              body: "The packaging check has failed. A new qualified result is required before the review can conclude.",
            },
            {
              title: "Authority remains in YAI",
              body: "Opening this review changes navigation only. A live review action will require the qualified YAI application boundary.",
            },
          ],
          references: ["manifest", "checks"],
        },
      },
      {
        id: "manifest",
        name: "Handoff manifest",
        path: "Artifacts / handoff-manifest.md",
        category: "artifact",
        format: "Document",
        changed: true,
        provenance: "Synthetic artifact · candidate revision",
        body: {
          kind: "document",
          eyebrow: "ARTIFACT / RELEASE CANDIDATE",
          title: "Handoff manifest",
          intro: "The material under review, before publication.",
          sections: [
            {
              title: "Included",
              body: "Release notes, validation summary and artifact inventory.",
            },
            {
              title: "Outstanding",
              body: "A successful packaging result and the explicit publication decision.",
            },
          ],
          references: ["checks"],
        },
      },
      {
        id: "checks",
        name: "Validation notes",
        path: "Sources / validation-notes.txt",
        category: "source",
        format: "Document",
        provenance: "Synthetic validation source · no real test transcript",
        body: {
          kind: "document",
          eyebrow: "SOURCE / VALIDATION NOTES",
          title: "Packaging needs attention.",
          intro:
            "An example failure state, retained rather than hidden by a success summary.",
          sections: [
            {
              title: "Observed in this fixture",
              body: "The expected release-notes artifact is absent from the packaging input.",
            },
            {
              title: "Next step",
              body: "Resolve the missing input, rerun through the qualified execution boundary and retain the resulting evidence.",
            },
          ],
        },
      },
      {
        id: "provider",
        name: "Provider context",
        path: "Providers / release-model",
        category: "provider",
        format: "Provider",
        provenance: "Synthetic provider context",
        body: { kind: "provider", title: "Release model" },
      },
    ],
    activity: [
      {
        id: "t1",
        kind: "turn",
        participant: "operator",
        time: "16:20",
        text: "Prepare the handoff, but leave publication for review. Keep failures visible.",
      },
      {
        id: "t2",
        kind: "turn",
        participant: "code",
        time: "16:22",
        text: "The manifest is prepared. The packaging check needs attention; I’ve included its result with the handoff.",
      },
      {
        id: "e1",
        kind: "execution",
        time: "16:23",
        title: "Packaging check failed",
        text: "Missing release-notes input (fixture)",
        material: "checks",
      },
      {
        id: "r1",
        kind: "review",
        time: "16:24",
        title: "Publication awaits review",
        text: "No publication has been authorized.",
        material: "review",
      },
      {
        id: "n1",
        kind: "notice",
        time: "16:24",
        title: "Frozen execution snapshot",
        text: "Running and waiting states are static fixtures. No work is executing.",
      },
    ],
    executions: [
      {
        id: "EX-031",
        title: "Assemble handoff",
        detail: "Manifest prepared",
        posture: "completed",
        time: "16:22",
      },
      {
        id: "EX-032",
        title: "Packaging check",
        detail: "Missing release-notes input",
        posture: "failed",
        time: "16:23",
      },
      {
        id: "EX-033",
        title: "Collect evidence",
        detail: "Static in-progress example",
        posture: "running",
        time: "16:24",
      },
      {
        id: "EX-034",
        title: "Publish handoff",
        detail: "Decision required · no effect",
        posture: "waiting for review",
        time: "16:24",
      },
    ],
    evidence: [
      {
        id: "EV-21",
        title: "Handoff manifest",
        origin: "Assemble handoff",
        detail: "Candidate artifact prepared",
        material: "manifest",
      },
      {
        id: "EV-22",
        title: "Packaging result",
        origin: "Packaging check",
        detail: "Failure retained · input absent",
        material: "checks",
      },
    ],
    problems: [
      {
        id: "P-11",
        severity: "error",
        title: "Missing release-notes input",
        detail: "Packaging cannot complete with the current input.",
        material: "checks",
      },
      {
        id: "P-12",
        severity: "warning",
        title: "Publication awaits review",
        detail: "No publication decision in this fixture.",
        material: "review",
      },
    ],
    output: [
      "16:22  [fixture] Handoff manifest prepared",
      "16:23  [fixture] Packaging check: FAILED",
      "       Missing input: release-notes.md",
      "16:24  [fixture] Publication: waiting for review",
      "",
      "Frozen example. No live process or provider connection.",
    ],
  },
];
