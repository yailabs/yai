"""Falsify classification drift and false external claims without model calls."""
import contextlib
import csv
import importlib.util
import io
import os
import re
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("topology", ROOT / "tools/validation/topology.py")
topology = importlib.util.module_from_spec(spec)
spec.loader.exec_module(topology)


class TopologyTests(unittest.TestCase):
    def altered(self, change):
        rows = topology.read_catalog()
        change(rows)
        with tempfile.TemporaryDirectory(prefix="yai-topology-test-") as temp:
            path = Path(temp) / "classification.tsv"
            with path.open("w", newline="") as stream:
                writer = csv.DictWriter(stream, fieldnames=topology.FIELDS, delimiter="\t")
                writer.writeheader()
                writer.writerows(rows)
            return topology.read_catalog(path)

    def test_fixture_cannot_authorize_external_qualification(self):
        with self.assertRaisesRegex(ValueError, "fixture cannot qualify"):
            self.altered(lambda rows: next(r for r in rows if r['cadence']=='fast').update(proof_class="external"))

    def test_external_provider_cannot_enter_local_publication(self):
        with self.assertRaisesRegex(ValueError, "external provider in local gate"):
            self.altered(lambda rows: next(r for r in rows if r['cadence']=='fast').update(provider_mode="external_yvex", network="external"))

    def test_unclassified_script_and_make_entry_fail_audit(self):
        rows = topology.read_catalog()
        rows = [r for r in rows if r["id"] != "smoke-cognitive-execution-composition"]
        with self.assertRaisesRegex(ValueError, "unclassified smoke leaves"):
            topology.audit(rows, with_rust=False)

    def test_duplicate_identity_and_missing_source_fail_closed(self):
        with self.assertRaisesRegex(ValueError, "duplicate validation identity"):
            self.altered(lambda rows: rows.append(rows[0].copy()))
        with self.assertRaisesRegex(ValueError, "missing classified source"):
            self.altered(lambda rows: rows[0].update(path="tests/absent-test.py"))

    def test_combined_graph_has_shared_leaf_dependencies_not_repeated_recipes(self):
        rows = topology.read_catalog()
        release = {r['entrypoint'] for r in topology.lane_rows(rows, 'release')}
        characterization = {r['entrypoint'] for r in topology.lane_rows(rows, 'characterization')}
        self.assertLessEqual(characterization, release)
        self.assertIn('smoke-cognitive-execution-composition', release)
        self.assertNotIn('qualification-yvex-provider', release)
        self.assertTrue(all(r['provider_mode']=='no_provider' for r in topology.lane_rows(rows, 'fast')))

    def test_external_absence_is_nonpass_before_any_network_attempt(self):
        env = {k:v for k,v in os.environ.items() if not k.startswith(('YVEX_', 'YAI_EXTERNAL_'))}
        proc = subprocess.run(['bash', str(ROOT/'tests/integration/yvex/qualification_yvex_provider.sh')],
                              cwd=ROOT, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(proc.returncode, 3, proc.stdout+proc.stderr)
        self.assertIn('provider_mode: external_yvex', proc.stdout)
        self.assertIn('external_request_attempted: false', proc.stdout)
        self.assertIn('DEPLOYMENT_LIMITATION', proc.stdout)
        self.assertNotIn('qualification_state: passed', proc.stdout)

    def test_golden_is_an_explicit_cadence_not_an_external_claim(self):
        rows = topology.read_catalog()
        golden = topology.lane_rows(rows, 'golden-local')
        self.assertTrue(golden)
        self.assertTrue(all(r['proof_class']=='product' and r['provider_mode']=='loopback_fixture' for r in golden))
        self.assertFalse({r['id'] for r in golden} & {r['id'] for r in topology.lane_rows(rows, 'release')})
        env = {k:v for k,v in os.environ.items() if not k.startswith(('YVEX_', 'YAI_EXTERNAL_'))}
        proc = subprocess.run(['python3', str(ROOT/'tests/characterization/case-resource-access/test_reference_free.py'), '--external'],
                              cwd=ROOT, env=env, capture_output=True, text=True)
        self.assertEqual(proc.returncode, 3, proc.stdout+proc.stderr)
        self.assertIn('"external_request_attempted": false', proc.stdout)
        self.assertIn('DEPLOYMENT_LIMITATION', proc.stdout)

    def test_failure_remains_nonzero_and_localized(self):
        # The real executor is libtest/Make, not a result counter that can turn
        # an empty or failed selection into success. Invalid selections refuse.
        with self.assertRaisesRegex(ValueError, 'empty Rust execution group'):
            topology.run_rust(topology.read_catalog(), 'test-rust-unclassified')

    def test_cumulative_runbook_is_product_and_preserves_operator_authority(self):
        manual = (ROOT / 'docs/zero-to-current.md').read_text()
        terminal = (ROOT / 'cmd/yai/src/conversation_terminal.rs').read_text()
        self.assertIn('Authority: cumulative human-executable acceptance', manual)
        self.assertIn('HUMAN_GOLDEN_CASE = PENDING_OPERATOR', manual)
        self.assertIn('CANDIDATE_DIGEST', manual)
        self.assertIn('integrity_digest', manual)
        self.assertNotIn('target/debug/yai', manual)
        self.assertNotIn('provider_governance_server.py', manual)
        self.assertNotIn('test_reference_free.py', manual)
        commands = re.findall(r'^(/\w+)(?: |$)', manual, re.M)
        self.assertTrue(commands)
        for command in set(commands):
            self.assertIn('"' + command, terminal, command)
        for path in re.findall(r'python3 ([^ ]+)', manual):
            self.assertEqual(path, 'tests/cases/04-golden/world.py')
        self.assertIn('cumulative', (ROOT / 'AGENTS.md').read_text())

    def test_actual_child_failure_reports_class_and_preserves_exit(self):
        with tempfile.TemporaryDirectory(prefix='yai-topology-failure-') as temp:
            binary = Path(temp)/'failing-libtest-peer'
            binary.write_text('#!/bin/sh\nif [ "$1" = "--list" ]; then echo "deliberate_failure: test"; exit 0; fi\necho "deliberate test infrastructure failure" >&2\nexit 7\n')
            binary.chmod(0o700)
            row = dict(topology.read_catalog()[0], kind='rust', id='engine:deliberate_failure',
                       selector='deliberate_failure', entrypoint='test-rust-contract',
                       proof_class='contract', provider_mode='no_provider')
            output = io.StringIO()
            with patch.object(topology, 'binaries', return_value={'engine':str(binary)}), contextlib.redirect_stdout(output):
                code = topology.run_rust([row], 'test-rust-contract')
            self.assertEqual(code, 7)
            self.assertIn('"proof_class": "contract"', output.getvalue())
            self.assertIn('"exit": 7', output.getvalue())


if __name__ == '__main__':
    unittest.main(verbosity=2)
