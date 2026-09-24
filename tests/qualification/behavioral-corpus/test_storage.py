#!/usr/bin/env python3
"""Storage accounting is bounded metadata, never a database-content reader."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "tools/validation"))
from profile_storage import measure_profile_storage


class StorageAccounting(unittest.TestCase):
    def test_hardlinks_count_once_and_external_symlinks_are_not_followed(self):
        with tempfile.TemporaryDirectory() as profile, tempfile.TemporaryDirectory() as outside:
            root = Path(profile)
            (root / "material").write_bytes(b"retained")
            os.link(root / "material", root / "same-backing")
            (Path(outside) / "external").write_bytes(b"outside" * 100)
            (root / "external").symlink_to(outside, target_is_directory=True)
            (root / "loop").symlink_to(root, target_is_directory=True)
            with patch.object(Path, "read_bytes", side_effect=AssertionError("content read")), patch.object(Path, "read_text", side_effect=AssertionError("content read")):
                result = measure_profile_storage(root)
            self.assertEqual(result["posture"], "observed")
            self.assertEqual(result["unique_files"], 1)
            self.assertEqual(result["logical_bytes"], len(b"retained"))
            self.assertEqual(result["duplicate_inodes_not_recounted"], 1)
            self.assertEqual(result["symlinks_not_followed"], 2)
            self.assertEqual((root / "material").read_bytes(), b"retained")

    def test_bounded_partial_observation_does_not_claim_complete_totals(self):
        with tempfile.TemporaryDirectory() as profile:
            root = Path(profile)
            for name in ["first", "second"]:
                (root / name).write_bytes(b"x")
            result = measure_profile_storage(root, 1)
            self.assertEqual(result["posture"], "partial")
            self.assertTrue(result["entry_limit_reached"])
            self.assertEqual(result["unique_files"], 1)
            for limit in [0, 1000001]:
                with self.assertRaises(ValueError):
                    measure_profile_storage(root, limit)

    def test_replaced_directory_cannot_redirect_scan(self):
        with tempfile.TemporaryDirectory() as profile, tempfile.TemporaryDirectory() as outside:
            root = Path(profile)
            (root / "child").mkdir()
            (Path(outside) / "external").write_bytes(b"external")
            original = os.open
            def replace_before_open(path, flags, **kwargs):
                if path == "child":
                    (root / "child").rmdir()
                    (root / "child").symlink_to(outside, target_is_directory=True)
                return original(path, flags, **kwargs)
            with patch("os.open", side_effect=replace_before_open):
                result = measure_profile_storage(root)
            self.assertEqual(result["posture"], "partial")
            self.assertEqual(result["logical_bytes"], 0)
            self.assertTrue(result["errors"])


if __name__ == "__main__":
    unittest.main()
