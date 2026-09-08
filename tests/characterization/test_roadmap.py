"""Documentation-control regression; no provider or product behavior claims."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("roadmap", ROOT / "tools/checks/check-roadmap.py")
roadmap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(roadmap)


class RoadmapTests(unittest.TestCase):
    def setUp(self):
        self.text = (ROOT / "ROADMAP.md").read_text()

    def test_real_roadmap_and_read_only_generated_summary(self):
        result = roadmap.validate(self.text)
        proc = subprocess.run(["python3", "tools/checks/check-roadmap.py", "--summary"],
                              cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertEqual(proc.stdout.strip(), result)
        self.assertEqual((ROOT / "ROADMAP.md").read_text(), self.text)

    def test_summary_counts_only_maturity_rows(self):
        expected = roadmap.validate(self.text)
        extra = "\n| Z99 | outside matrix | 🟢 ESTABLISHED | not a maturity row |\n"
        self.assertEqual(roadmap.validate(self.text + extra), expected)

    def test_stale_summary_and_deleted_row_refuse(self):
        old = roadmap.section(self.text, "maturity-summary")
        with self.assertRaisesRegex(ValueError, "stale maturity summary"):
            roadmap.validate(self.text.replace(old, "TOTAL=999"))
        row = next(line for line in self.text.splitlines() if line.startswith("| K01 |"))
        with self.assertRaisesRegex(ValueError, "stale maturity summary"):
            roadmap.validate(self.text.replace(row, ""))

    def test_unknown_state_duplicate_identity_and_malformed_row_refuse(self):
        for before, after, error in [
            ("🟢 ESTABLISHED |", "🟢 COMPLETE |", "invalid maturity state"),
            ("| K02 |", "| K01 |", "duplicate maturity identity"),
            ("| K02 |", "| K02 | extra |", "malformed table row"),
            ("<!-- maturity:end -->", "", "marker pair"),
        ]:
            with self.subTest(error=error), self.assertRaisesRegex(ValueError, error):
                roadmap.validate(self.text.replace(before, after, 1))

    def test_evidence_traceability_is_required_not_invented(self):
        row = next(line for line in self.text.splitlines() if line.startswith("| K01 |"))
        parts = row.split("|")
        parts[-2] = " assertion without evidence "
        with self.assertRaisesRegex(ValueError, "lacks evidence link"):
            roadmap.validate(self.text.replace(row, "|".join(parts)))
        for extra, error in [
            ("\n[missing][undefined]\n", "undefined references"),
            ("\n[missing](docs/nonexistent-roadmap-evidence.md)\n", "missing or escaping"),
            ("\n[escape](../private.md)\n", "missing or escaping"),
            ("\n[fragment](#absent-heading)\n", "missing heading"),
            ("\n[kernel]: ROADMAP.md\n", "duplicate reference"),
        ]:
            with self.subTest(error=error), self.assertRaisesRegex(ValueError, error):
                roadmap.validate(self.text + extra)

    def test_exactly_one_temporal_boundary_and_matching_snapshot(self):
        body = roadmap.section(self.text, "execution")
        row = body.splitlines()[-1]
        state = row.split("|")[2].strip()
        other = next(s for s in sorted(roadmap.TEMPORAL) if s != state)
        for replacement, error in [
            (body + "\n" + row, "exactly one selected"),
            (body.replace(state, "ESTABLISHED"), "valid temporal"),
            (body.replace(state, other), "snapshot"),
        ]:
            with self.subTest(error=error), self.assertRaisesRegex(ValueError, error):
                roadmap.validate(self.text.replace(body, replacement))

    def test_no_competing_live_control_file_or_authority_declaration(self):
        with tempfile.TemporaryDirectory(prefix="yai-roadmap-") as temp:
            root = Path(temp)
            (root / "docs").mkdir()
            competitor = root / "docs/PLAN.md"
            competitor.write_text("# A second queue\n")
            with self.assertRaisesRegex(ValueError, "competing live roadmap"):
                roadmap.validate(self.text, root)
            competitor.unlink()
            (root / "docs/other.md").write_text(roadmap.AUTHORITY)
            with self.assertRaisesRegex(ValueError, "competing live roadmap"):
                roadmap.validate(self.text, root)

    def test_control_section_cannot_disappear(self):
        with self.assertRaisesRegex(ValueError, "missing/duplicate control section"):
            roadmap.validate(self.text.replace("## Strategic Programs", "## Old wave log"))


if __name__ == "__main__":
    unittest.main()
