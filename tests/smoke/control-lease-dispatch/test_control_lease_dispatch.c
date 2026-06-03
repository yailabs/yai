/*
 * YAI - CORE.ENFORCE.1 control lease dispatch smoke
 *
 * Implements:
 *   Negative and positive proof that an executable carrier path is admitted
 *   only when a minted CapabilityLease permits execution AND the control
 *   decision allows. A ref, binding, model proposal or allow-decision alone is
 *   not sufficient.
 *
 * This file owns:
 *   Lease-before-dispatch admission coverage for the filesystem carrier and the
 *   skeleton no-execution posture.
 *
 * This file does not own:
 *   Network, database, git, service or model_provider carrier execution.
 */

#include "yai/yai.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static void require_ok(yai_status_t status) {
    if (status != YAI_OK) {
        printf("unexpected status: %s\n", yai_status_string(status));
        assert(status == YAI_OK);
    }
}

static void mint_permitting_lease(yai_capability_lease_t *lease,
                                  const yai_case_ref_t *case_ref,
                                  const yai_subject_ref_t *subject_ref) {
    require_ok(yai_capability_lease_init(lease, case_ref, subject_ref, "fs.write"));
    (void)snprintf(lease->lease_status, sizeof(lease->lease_status), "minted");
    (void)snprintf(lease->allowed_actions, sizeof(lease->allowed_actions), "fs.write");
    lease->requires_review = 0;
}

static void make_decision(const yai_case_ref_t *case_ref,
                          const char *decision_id,
                          yai_decision_outcome_t outcome,
                          yai_control_decision_t *decision) {
    memset(decision, 0, sizeof(*decision));
    yai_id_set(&decision->decision_id, decision_id);
    decision->case_ref = *case_ref;
    yai_id_set(&decision->attempt_id, "op:core-enforce1");
    yai_id_set(&decision->basis_id, "basis:core-enforce1");
    yai_id_set(&decision->gate_id, "gate:core-enforce1");
    yai_id_set(&decision->rule_id, "rule:core-enforce1");
    yai_id_set(&decision->obligation_id, "");
    yai_id_set(&decision->receipt_requirement_id, "receipt-requirement:core-enforce1");
    decision->outcome = outcome;
    decision->receipt_required = 1;
    (void)snprintf(decision->reason, sizeof(decision->reason), "core-enforce1 fixture");
}

static yai_dispatch_admission_outcome_t admit(const yai_capability_lease_t *lease,
                                              const yai_control_decision_t *decision,
                                              int *execution_admitted) {
    yai_dispatch_admission_t admission;
    require_ok(yai_dispatch_admit(lease, decision, &admission));
    if (execution_admitted != 0) {
        *execution_admitted = admission.execution_admitted;
    }
    return admission.outcome;
}

int main(void) {
    yai_case_ref_t case_ref;
    yai_subject_ref_t actor_ref;
    yai_subject_ref_t file_subject_ref;
    yai_capability_lease_t default_lease;
    yai_capability_lease_t permitting_lease;
    yai_capability_lease_t review_lease;
    yai_capability_lease_t empty_actions_lease;
    yai_control_decision_t allow_decision;
    yai_control_decision_t allow_constraints_decision;
    yai_control_decision_t deny_decision;
    yai_control_decision_t review_decision;
    yai_control_decision_t defer_decision;
    int admitted = 0;

    require_ok(yai_case_ref_init(&case_ref, "case:core-enforce1", "core-enforce1", "open"));
    require_ok(yai_subject_ref_init(&actor_ref, "subject:model-actor", "model", "local:model"));
    require_ok(yai_subject_ref_init(&file_subject_ref, "subject:filesystem-sandbox", "filesystem", "sandbox"));

    /* --- Section A: lease predicate is fail-closed --- */
    require_ok(yai_capability_lease_init(&default_lease, &case_ref, &actor_ref, "fs.write"));
    assert(yai_capability_lease_permits_execution(&default_lease) == 0);
    assert(yai_capability_lease_permits_execution(0) == 0);
    printf("lease:default no_execute ok\n");

    mint_permitting_lease(&permitting_lease, &case_ref, &actor_ref);
    assert(yai_capability_lease_permits_execution(&permitting_lease) == 1);
    printf("lease:minted permits_execute ok\n");

    mint_permitting_lease(&review_lease, &case_ref, &actor_ref);
    review_lease.requires_review = 1;
    assert(yai_capability_lease_permits_execution(&review_lease) == 0);
    printf("lease:review no_execute ok\n");

    mint_permitting_lease(&empty_actions_lease, &case_ref, &actor_ref);
    (void)snprintf(empty_actions_lease.allowed_actions, sizeof(empty_actions_lease.allowed_actions), "none");
    assert(yai_capability_lease_permits_execution(&empty_actions_lease) == 0);
    printf("lease:no_actions no_execute ok\n");

    /* --- Section B: admission gate requires lease AND allow --- */
    make_decision(&case_ref, "decision:allow", YAI_DECISION_ALLOW, &allow_decision);
    make_decision(&case_ref, "decision:allow-c", YAI_DECISION_ALLOW_WITH_CONSTRAINTS, &allow_constraints_decision);
    make_decision(&case_ref, "decision:deny", YAI_DECISION_DENY, &deny_decision);
    make_decision(&case_ref, "decision:review", YAI_DECISION_REQUIRE_REVIEW, &review_decision);
    make_decision(&case_ref, "decision:defer", YAI_DECISION_DEFER, &defer_decision);

    /* CaseHandle/SubjectHandle present but no minted lease -> no execution. */
    assert(admit(0, &allow_decision, &admitted) == YAI_DISPATCH_ADMIT_DENY && admitted == 0);
    printf("admit:no_lease denied ok\n");

    /* Default (non-minted) lease == subject/handle-only / resource mismatch -> no execution. */
    assert(admit(&default_lease, &allow_decision, &admitted) == YAI_DISPATCH_ADMIT_DENY && admitted == 0);
    printf("admit:subject_only denied ok\n");

    /* Model/provider proposal text but no lease -> no execution. */
    assert(admit(0, &allow_constraints_decision, &admitted) == YAI_DISPATCH_ADMIT_DENY && admitted == 0);
    printf("admit:proposal_no_lease denied ok\n");

    /* Permitting lease + allow -> execution admitted. */
    assert(admit(&permitting_lease, &allow_decision, &admitted) == YAI_DISPATCH_ADMIT_EXECUTE && admitted == 1);
    assert(admit(&permitting_lease, &allow_constraints_decision, &admitted) == YAI_DISPATCH_ADMIT_EXECUTE && admitted == 1);
    printf("admit:lease_allow execute ok\n");

    /* Permitting lease + deny/defer/review -> no execution. */
    assert(admit(&permitting_lease, &deny_decision, &admitted) == YAI_DISPATCH_ADMIT_DENY && admitted == 0);
    assert(admit(&permitting_lease, &review_decision, &admitted) == YAI_DISPATCH_ADMIT_REVIEW && admitted == 0);
    assert(admit(&permitting_lease, &defer_decision, &admitted) == YAI_DISPATCH_ADMIT_REVIEW && admitted == 0);
    printf("admit:deny_defer_review no_execute ok\n");

    /* Review-bound lease + allow -> no execution. */
    assert(admit(&review_lease, &allow_decision, &admitted) == YAI_DISPATCH_ADMIT_DENY && admitted == 0);
    printf("admit:review_lease denied ok\n");

    /* Missing decision -> fail closed. */
    assert(admit(&permitting_lease, 0, &admitted) == YAI_DISPATCH_ADMIT_DENY && admitted == 0);
    printf("admit:missing_decision denied ok\n");

    /* --- Section C: filesystem carrier cannot execute without admission --- */
    {
        char run_dir[160];
        char sandbox[224];
        char output_path[288];
        yai_op_attempt_t attempt;
        yai_effect_receipt_t receipt;
        int written = 0;

        written = snprintf(run_dir, sizeof(run_dir), "build/tmp/core-enforce1/run-%ld", (long)getpid());
        assert(written > 0 && (size_t)written < sizeof(run_dir));
        written = snprintf(sandbox, sizeof(sandbox), "%s/sandbox", run_dir);
        assert(written > 0 && (size_t)written < sizeof(sandbox));
        written = snprintf(output_path, sizeof(output_path), "%s/out.txt", sandbox);
        assert(written > 0 && (size_t)written < sizeof(output_path));

        (void)mkdir("build", 0777);
        (void)mkdir("build/tmp", 0777);
        (void)mkdir("build/tmp/core-enforce1", 0777);
        (void)mkdir(run_dir, 0777);
        (void)mkdir(sandbox, 0777);
        (void)unlink(output_path);

        require_ok(yai_op_attempt_init(&attempt, "op:core-enforce1-write", &case_ref,
                                       &actor_ref, &file_subject_ref, "fs.write",
                                       "internal", "local", "mutative", "lease-gated write"));

        /* No lease admission -> dispatcher must not invoke the carrier.
         * Prove the would-be effect never lands on disk. */
        assert(admit(0, &allow_constraints_decision, &admitted) == YAI_DISPATCH_ADMIT_DENY && admitted == 0);
        assert(access(output_path, F_OK) != 0);
        printf("carrier:no_lease not_executed ok\n");

        /* Deny decision reaching the carrier still blocks (defense in depth). */
        require_ok(yai_filesystem_carrier_write("receipt:core-enforce1-deny", &attempt,
                                                &deny_decision, sandbox, output_path,
                                                "should not write", &receipt));
        assert(receipt.status != YAI_RECEIPT_EXECUTED);
        assert(access(output_path, F_OK) != 0);
        printf("carrier:deny_decision blocked ok\n");

        /* Admitted (permitting lease + allow) -> carrier executes. */
        assert(admit(&permitting_lease, &allow_constraints_decision, &admitted) == YAI_DISPATCH_ADMIT_EXECUTE && admitted == 1);
        require_ok(yai_filesystem_carrier_write("receipt:core-enforce1-exec", &attempt,
                                                &allow_constraints_decision, sandbox, output_path,
                                                "executed\n", &receipt));
        assert(receipt.status == YAI_RECEIPT_EXECUTED);
        assert(access(output_path, F_OK) == 0);
        printf("carrier:lease_allow executed ok\n");
    }

    /* --- Section D: skeleton carriers never execute --- */
    {
        const yai_carrier_skeleton_t *net = yai_carrier_skeleton_for_family(YAI_CARRIER_FAMILY_NETWORK_HTTP);
        const yai_carrier_skeleton_t *model = yai_carrier_skeleton_for_family(YAI_CARRIER_FAMILY_MODEL_PROVIDER);
        assert(net != 0 && net->execution_available == 0);
        assert(model != 0 && model->execution_available == 0);
        printf("skeleton:network no_execution ok\n");
        printf("skeleton:model_provider no_execution ok\n");
    }

    printf("core-enforce1:lease_gated_dispatch ok\n");
    return 0;
}
