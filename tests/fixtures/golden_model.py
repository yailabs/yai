"""Deterministic native-function model peer for AUTOMATED Golden proof only.

No filesystem/database access here: every evidence value comes back through
YAI's real capability adapter and canonical function-result messages.
This is neither a production model nor human/YVEX qualification.
"""
import json


def frame_from_request(request):
    texts = []
    for message in request["messages"]:
        content = message.get("content")
        if isinstance(content, str):
            texts.append(content)
        elif isinstance(content, list):
            texts.extend(part["text"] for part in content if part.get("type") == "text")
    return next(json.loads(text.split("\n", 1)[1]) for text in texts
                if text.startswith("YAI typed ContextFrame:\n"))


def plan_patch_reply(request):
    frame = frame_from_request(request)
    contract = frame["output_contract"]
    assert contract["kind"] == "workflow_plan_patch"
    contract = contract["contract"]
    assert contract["schema"] == "yai.workflow_plan_patch.v1"
    return json.dumps({"schema": contract["schema"],
        "base_effective_topology_digest": contract["base_effective_topology_digest"],
        "operations": [
            {"operation":"add_node","node":{"node_id":"verify-history","kind":"human_input","actor_slot":"operator",
                "prompt":"Inspect exact review, effect and test result lineage before release verification.",
                "required_roles":["workflow-input"],"input_kind":"text","max_bytes":1024}},
            {"operation":"disable_edge","from":"repair","to":"verify"},
            {"operation":"add_edge","edge":{"from":"repair","to":"verify-history"}},
            {"operation":"add_edge","edge":{"from":"verify-history","to":"verify"}}
        ]})


def native_reply(request):
    tools = [tool["function"] for tool in request["tools"]]
    messages = request["messages"]
    results = [json.loads(message["content"]) for message in messages if message["role"] == "tool"]
    ordinal = len(results)
    user = next(message["content"] for message in reversed(messages) if message["role"] == "user")
    task = user[-1]["text"] if isinstance(user, list) else user
    repair = "GOLDEN-REPAIR:" in task
    investigate = "GOLDEN-INVESTIGATE:" in task
    for value in results:
        assert value["material_is_authority"] is False
        assert value["operation_id"] and value["transition_id"] and value["provider_result_id"]

    if repair:
        if ordinal >= 2:
            before = results[1]["outcome"]["data"]["observation"]["result"]
            assert isinstance(before["exit_code"], int) and before["exit_code"] != 0
            assert not before["timed_out"] and not before["output_limit_exceeded"]
        if ordinal == 5:
            assert '"exit_code": 0' in json.dumps(results[4])
            return {"role":"assistant","content":"Reviewed repair has an exact passing process receipt."}, "stop"
        actions = [
            ("filesystem.write","resource:workspace",{"path":"protected/release.conf","content":"forbidden change\n"}),
            ("process.run","resource:runner",{"name":"tests"}),
            ("filesystem.write","resource:workspace",{"path":"src/retry.py","content":
                'def delay_ms(attempt):\n    if not 1 <= attempt <= 20:\n        raise ValueError("attempt domain")\n    return min(250, 100 * (2 ** (attempt - 1)))\n'}),
            # Inspect the actual admitted source consequence before verification.
            ("filesystem.read","resource:workspace",{"path":"src/retry.py"}),
            ("process.run","resource:runner",{"name":"tests"}),
        ]
        operation, resource, arguments = actions[ordinal]
        definition = next(tool for tool in tools if f"Request {operation} on Case resource {resource} " in tool["description"])
        return {"role":"assistant","content":None,"tool_calls":[{"id":f"repair-call-{ordinal}","type":"function",
            "function":{"name":definition["name"],"arguments":json.dumps(arguments)}}]}, "tool_calls"

    if investigate and ordinal == 10:
        assert "stable" in json.dumps(results[2]) and "canary" in json.dumps(results[3])
        return {"role":"assistant","content":"Independent issue, code, release state, specification, migration and risk observations gathered; no code mutation performed."}, "stop"

    def find(value, predicate):
        if isinstance(value, dict):
            if predicate(value):
                return value
            for child in value.values():
                found = find(child, predicate)
                if found is not None:
                    return found
        if isinstance(value, list):
            for child in value:
                found = find(child, predicate)
                if found is not None:
                    return found
        return None

    frame = frame_from_request(request)
    # Bounded Projection may evict an already-consumed issue reference. The
    # exact canonical tool result is the admitted working-set continuation.
    issue = find(results if results else frame, lambda v: v.get("source_path") == "issue/issue.md" and "admission_id" in v)
    assert issue, "An exact admitted issue must be visible before Golden work"
    actions = [
        ("content.read", "resource:discovery", {"admission_id": issue["admission_id"]}),
        ("filesystem.read", "resource:workspace", {"path": "src/retry.py"}),
        ("database.query", "resource:database", {"name": "release"}),
        ("http.fetch", "resource:service", {"name": "status"}),
        ("mcp.catalog", "resource:mcp", {}),
        ("mcp.resource.read", "resource:mcp", {}),
        ("discovery.enumerate", "resource:discovery", {"path": "notes"}),
    ]
    if ordinal >= 7:
        candidate = find(results[6], lambda v: v.get("path") == "notes/migration.md" and "digest" in v)
        assert candidate and candidate["admitted"] is False
        actions.append(("content.admit", "resource:discovery", {"path": candidate["path"], "candidate_digest": candidate["digest"]}))
    if ordinal >= 8:
        admitted = find(results[7], lambda v: "admission_id" in v and "object" in v)
        assert admitted and admitted["source_path"] == "notes/migration.md"
        actions.append(("content.read", "resource:discovery", {"admission_id": admitted["admission_id"]}))
    if ordinal >= 9:
        evidence = json.dumps(results)
        assert "stable" in json.dumps(results[2]) and "canary" in json.dumps(results[3])
        assert "stable/canary=250" in json.dumps(results[5]) and "first retry as attempt 1" in evidence
        actions.extend([
            ("mcp.tool.call", "resource:mcp", {"delays_ms": [100, 200, 250, 250]}),
            ("filesystem.write", "resource:workspace", {"path": "protected/release.conf", "content": "forbidden change\n"}),
            ("process.run", "resource:runner", {"name": "tests"}),
            ("filesystem.write", "resource:workspace", {"path": "src/retry.py", "content":
                '"""Retry schedule qualified for GOLDEN-42 stable/canary."""\n\n'
                'def delay_ms(attempt):\n'
                '    if not 1 <= attempt <= 20:\n'
                '        raise ValueError("attempt must be between 1 and 20")\n'
                '    return min(250, 100 * (2 ** (attempt - 1)))\n'}),
            ("process.run", "resource:runner", {"name": "tests"}),
        ])
    if ordinal >= 11:
        assert results[10]["posture"] == "denied", "Protected write must be denied, not applied"
    if ordinal >= 12:
        before = find(results[11], lambda v: "exit_code" in v)
        assert before and isinstance(before["exit_code"], int) and before["exit_code"] != 0, "Initial real oracle must exit nonzero"
        assert not before["timed_out"] and not before["output_limit_exceeded"]
    if ordinal >= 14:
        after = find(results[13], lambda v: "exit_code" in v)
        assert after and after["exit_code"] == 0, "Final real oracle must pass"
        return {"role": "assistant", "content": "Golden free work complete: independent evidence, protected denial, reviewed source change and real passing test receipt."}, "stop"
    operation, resource, arguments = actions[ordinal]
    definition = next(tool for tool in tools if f"Request {operation} on Case resource {resource} " in tool["description"])
    return {"role": "assistant", "content": None, "tool_calls": [{"id": "golden-call-" + str(ordinal), "type": "function",
        "function": {"name": definition["name"], "arguments": json.dumps(arguments)}}]}, "tool_calls"
