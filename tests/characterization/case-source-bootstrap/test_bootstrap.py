#!/usr/bin/env python3
"""Source-bootstrap product proof: actual ./yai, fresh LMDB, files and SQLite.

The HTTP peer is an ordinary denied source, never a model/provider fixture.
"""
import hashlib
import http.server
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
CASE, TENANT, PERSON = "case:bootstrap", "tenant:bootstrap", "participant:operator"


def main():
    with tempfile.TemporaryDirectory(prefix="yai-source-bootstrap-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), NO_COLOR="1")
        order = 0
        print(json.dumps(dict(run_id=run.name, cwd=str(ROOT), home=env["YAI_HOME"],
            pre_state="fresh empty disposable home", provider_mode="no_provider",
            baseline=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip())), flush=True)

        def cli(*args, reject=False):
            nonlocal order
            order += 1
            start = time.perf_counter()
            command = ["./yai", *map(str, args)]
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
            print(json.dumps(dict(run_id=run.name, order=order, command=command, exit=result.returncode,
                elapsed_ms=round((time.perf_counter()-start)*1000, 3), stdout=result.stdout[:16384],
                stdout_excerpt=len(result.stdout)>16384, stderr=result.stderr)), flush=True)
            assert (result.returncode != 0) == reject, result
            if not reject and "--json" in args:
                return json.loads(result.stdout)["data"]["value"]
            return result.stdout

        def write(name, value):
            path = run / name
            path.write_text(json.dumps(value))
            return path

        def setup(case):
            cli("case", "create", case, "--tenant", TENANT)
            cli("case", "participant", "role", "add", case, "--participant", PERSON, "--role", "operation-proposer")
            cli("case", "participant", "link-principal", case, "--participant", PERSON, "--principal", "self")

        def source(name, resource, action, roles=("knowledge",), bootstrap=False):
            return dict(name=name, resource="resource:"+resource, roles=list(roles), action=action,
                media_type="application/json" if bootstrap else "application/octet-stream", bootstrap_policy=bootstrap)

        def resource(name, address, operations, prefixes=(), names=()):
            return dict(schema="yai.resource_definition.v1", attachment_id="resource:"+name,
                policy_owner=PERSON, participant_ids=[PERSON], operations=operations, read_prefixes=list(prefixes),
                names=list(names), max_output_bytes=65536, max_items=128, address=address)

        def perimeter(name, sources, resources=()):
            return dict(schema="yai.source_perimeter.v1", name=name, participant=PERSON, resources=list(resources), sources=sources)

        def sources(action, *args, case=CASE):
            return cli("case", "sources", action, case, *args, "--json")

        def by_name(value):
            return {s["name"]: s for s in value["sources"]}

        cli("init", "--tenant", TENANT, "--organization", "organization:bootstrap")
        setup(CASE)
        files = run / "files"
        (files / "docs").mkdir(parents=True)
        (files / "docs/guide.txt").write_text("Exact organizational source, not a Case fact.\n")
        (files / "docs/duplicate.txt").write_bytes((files / "docs/guide.txt").read_bytes())
        (files / "secret.txt").write_text("UNDECLARED_SIBLING_NEVER_ACQUIRE")
        rules = []
        for action, family, effect in [("discovery.enumerate", "discovery", "allow"),
            ("content.admit", "discovery", "allow"), ("content.read", "discovery", "allow"),
            ("database.query", "database", "allow"), ("http.fetch", "http_service", "deny")]:
            rules.append(dict(kind="operation_restriction", rule_id=action, operation_kind=action,
                resource_kind=family, effect=effect, reason="Explicit bounded source bootstrap policy"))
        policy = dict(schema="yai.policy_source_input.v4", policy_key="bootstrap", source_version="1",
            owner_ref="organization:bootstrap", source_origin=dict(source_system="bootstrap-product", source_uri="test://company/security/v1"),
            validity=dict(mode="unbounded"), rules=rules)
        (files / "policy.json").write_text(json.dumps(policy))
        dbroot = run / "database"
        dbroot.mkdir()
        with sqlite3.connect(dbroot / "company.sqlite") as db:
            db.execute("CREATE TABLE departments(name TEXT)")
            db.execute("INSERT INTO departments VALUES ('engineering')")
        requests = []

        class DeniedPeer(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                requests.append(self.path)
                payload = b"FORBIDDEN_REMOTE_PAYLOAD_NEVER_ACQUIRE"
                self.send_response(200)
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), DeniedPeer)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        try:
            declarations = [source("security", "documents", dict(action="discover", path="policy.json"), ("policy", "knowledge"), True),
                source("guide", "documents", dict(action="discover", path="docs/guide.txt")),
                source("duplicate", "documents", dict(action="discover", path="docs/duplicate.txt")),
                source("missing", "documents", dict(action="discover", path="docs/missing.txt")),
                source("database", "database", dict(action="database_query", name="metadata"), ("knowledge", "operational")),
                source("denied", "remote", dict(action="http_fetch", name="private"))]
            resources = [resource("documents", dict(kind="discovery", root=str(files)), ["discover", "admit_content", "content_read"], ("policy.json", "docs")),
                resource("database", dict(kind="sqlite", root=str(dbroot), path="company.sqlite", queries={"metadata":"SELECT name FROM sqlite_master WHERE type='table'"}), ["database_query"], names=("metadata",)),
                resource("remote", dict(kind="http_service", endpoint=dict(endpoint=f"http://127.0.0.1:{server.server_port}/company", allowed_ip_addresses=["127.0.0.1"], credential_ref=None), paths={"private":"private"}), ["http_fetch"], names=("private",))]
            manifest = write("perimeter.json", perimeter("company", declarations, resources))
            inv = sources("declare", "--file", manifest)
            assert inv["coverage"]["declared"] == 6
            before = inv["generation"]
            assert sources("declare", "--file", manifest)["generation"] == before
            inv = sources("acquire")
            items = by_name(inv)
            assert items["security"]["phase"] == "acquired"
            assert all(items[n]["phase"] == "discovered" for n in items if n != "security")
            assert not requests
            policy_revision = items["security"]["progress"]["revision"]
            assert len(policy_revision["items"]) == 1
            assert policy_revision["items"][0]["backing"]["kind"] == "policy"
            cli("case", "sources", "publish", CASE, "--source", "guide", "--reason", "must not promote knowledge", reject=True)
            inv = sources("publish", "--source", "security", "--reason", "Inspected exact rules; explicit policy publication")
            assert inv["effective_policy"]["readiness"] == "ready"
            sources("acquire", "--limit", "1")
            # Each call is a fresh product process: persisted partial progress,
            # not an in-process cache, is the only source of resume information.
            inv = sources("resume")
            items = by_name(inv)
            assert items["guide"]["phase"] == items["database"]["phase"] == items["duplicate"]["phase"] == "acquired"
            assert items["denied"]["phase"] == "denied" and items["missing"]["phase"] == "inaccessible"
            assert requests == [], "DENY must precede remote acquisition"
            for path in (run / "home").rglob("*"):
                if path.is_file():
                    assert b"FORBIDDEN_REMOTE_PAYLOAD_NEVER_ACQUIRE" not in path.read_bytes()
                    assert b"UNDECLARED_SIBLING_NEVER_ACQUIRE" not in path.read_bytes()
            r1 = items["guide"]["progress"]["revision"]
            duplicate = items["duplicate"]["progress"]["revision"]
            assert r1["revision_id"] != duplicate["revision_id"]
            assert r1["items"][0]["digest"] == duplicate["items"][0]["digest"]
            assert sources("resume", "--source", "guide")["generation"] == inv["generation"]
            assert sources("read", "--source", "guide")["items"][0]["text"].startswith("Exact organizational")
            (files / "docs/guide.txt").write_text("Explicit updated source revision, old backing preserved.\n")
            updated = by_name(sources("acquire", "--source", "guide", "--refresh"))["guide"]
            assert updated["progress"]["revision"]["revision_id"] != r1["revision_id"]
            assert sources("read", "--source", "guide", "--revision", r1["revision_id"])["items"][0]["text"].startswith("Exact organizational")
            same = by_name(sources("acquire", "--source", "guide", "--refresh"))["guide"]
            assert same["progress"]["revision"]["revision_id"] == updated["progress"]["revision"]["revision_id"]
            assert same["progress"]["revision"]["items"][0]["backing"] == updated["progress"]["revision"]["items"][0]["backing"]
            assert same["progress"]["detail"] == "unchanged_revision_reused"
            assert updated["progress"]["detail"] == "changed_revision_acquired_old_backing_retained"
            (files / "docs/missing.txt").write_text("Recovered after interruption.\n")
            assert by_name(sources("resume", "--source", "missing"))["missing"]["phase"] == "acquired"
            (files / "docs/new.txt").write_text("Incremental admission under existing governance.\n")
            added = source("new", "documents", dict(action="discover", path="docs/new.txt"))
            sources("declare", "--file", write("incremental.json", perimeter("incremental", [added])))
            assert by_name(sources("acquire", "--source", "new"))["new"]["phase"] == "acquired"
            ordinary_policy = dict(policy, policy_key="document-not-authority")
            (files / "docs/policy-data.json").write_text(json.dumps(ordinary_policy))
            catalog_before = cli("policy", "list", "--tenant", TENANT)
            knowledge_policy = source("policy-as-data", "documents", dict(action="discover", path="docs/policy-data.json"))
            sources("declare", "--file", write("policy-as-data.json", perimeter("incremental", [knowledge_policy])))
            documentary = by_name(sources("acquire", "--source", "policy-as-data"))["policy-as-data"]
            assert documentary["progress"]["revision"]["items"][0]["backing"]["kind"] == "content"
            assert cli("policy", "list", "--tenant", TENANT) == catalog_before
            cli("case", "sources", "publish", CASE, "--source", "policy-as-data", "--reason", "knowledge must not govern", reject=True)
            before = sources("inventory")["generation"]
            sources("read", "--source", "new")
            assert sources("inventory")["generation"] == before
            sources("revoke", "--source", "guide", "--reason", "Withdraw current source relation")
            cli("case", "sources", "read", CASE, "--source", "guide", "--revision", r1["revision_id"], reject=True)
            cli("case", "verify", CASE)
            old = sources("inventory")
            cli("case", "policy", "rebuild", "--case", CASE)
            rebuilt = sources("inventory")
            assert old["sources"] == rebuilt["sources"]
            # A second Case can reuse the immutable policy original/catalog
            # without acquiring any right to the first Case's ordinary sources.
            other = "case:policy-only"
            setup(other)
            cli("case", "sources", "read", other, "--source", "new", reject=True)
            other_files = run / "policy-only"
            other_files.mkdir()
            (other_files / "policy.json").write_bytes((files / "policy.json").read_bytes())
            single = perimeter("policy-only", [source("security", "policy", dict(action="discover", path="policy.json"), ("policy",), True)],
                [resource("policy", dict(kind="discovery", root=str(other_files)), ["discover"], ("policy.json",))])
            sources("declare", "--file", write("policy-only.json", single), case=other)
            only = by_name(sources("acquire", case=other))["security"]
            assert only["progress"]["revision"]["items"][0]["backing"] == policy_revision["items"][0]["backing"]
            assert sources("publish", "--source", "security", "--reason", "Independent Case binding", case=other)["effective_policy"]["readiness"] == "ready"
            assert sources("inventory", case=other)["coverage"]["declared"] == 1
            cli("case", "verify", other)
            # Inventory is a read over exact declarations, not an ambient crawl.
            bad = source("sibling", "documents", dict(action="discover", path="secret.txt"), ("policy",), True)
            cli("case", "sources", "declare", CASE, "--file", write("sibling.json", perimeter("bad", [bad])), reject=True)
            cli("case", "sources", "declare", CASE, "--file", write("malformed.json", {"schema":"unknown"}), reject=True)
            # Larger explicit perimeter: fixed source bounds, no claim of a
            # universal production size or Case-age-independent host cost.
            large = []
            for n in range(32):
                path = f"docs/scale-{n:03}.txt"
                (files / path).write_text(f"Exact source item {n}; metadata is not knowledge.\n")
                large.append(source(f"scale-{n:03}", "documents", dict(action="discover", path=path)))
            started = time.perf_counter()
            declared = sources("declare", "--file", write("large.json", perimeter("scale", large)))
            declaration_ms = (time.perf_counter()-started)*1000
            started = time.perf_counter()
            sources("inventory")
            inventory_ms = (time.perf_counter()-started)*1000
            started = time.perf_counter()
            sources("acquire", "--limit", "8")
            acquisition_ms = (time.perf_counter()-started)*1000
            started = time.perf_counter()
            complete = sources("resume")
            resume_ms = (time.perf_counter()-started)*1000
            scaled = [s for s in complete["sources"] if s["perimeter"] == "scale"]
            assert len(scaled) == 32 and all(s["phase"] == "acquired" for s in scaled)
            total_bytes = sum(i["bytes"] for s in scaled for i in s["progress"]["revision"]["items"])
            print(json.dumps(dict(characterization="source_perimeter", source_count=len(complete["sources"]),
                scaled_sources=32, items_acquired=32, bytes_acquired=total_bytes, declare_ms=round(declaration_ms,3),
                inventory_ms=round(inventory_ms,3), first_batch_ms=round(acquisition_ms,3), resume_ms=round(resume_ms,3),
                final_counts=complete["coverage"]["counts"], posture="observed_not_SLA")), flush=True)
            cli("case", "verify", CASE)
            # Catalog revocation has no Case generation dependency.
            artifact = policy_revision["items"][0]["backing"]["artifact_id"]
            cli("policy", "revoke", artifact, "--reason", "Withdraw acquisition authority")
            cli("case", "sources", "read", CASE, "--source", "new", reject=True)
            cli("case", "sources", "declare", CASE, "--file", write("bad-reentry.json", perimeter("reentry", [dict(declarations[0], name="reentry")])), reject=True)
            print("source_bootstrap_product=PASS policy_first=true dual_role_one_backing=true denied_payloads=0 resumed=true revisions=true current_revocation=true models=0", flush=True)
        finally:
            server.shutdown()
            server.server_close()


if __name__ == "__main__":
    main()
