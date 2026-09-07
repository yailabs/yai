"""Independent fixed behavioral oracle; the model may change src/, not tests/."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("retry", Path(__file__).resolve().parents[1] / "src/retry.py")
retry = importlib.util.module_from_spec(spec)
spec.loader.exec_module(retry)


class ReleaseContract(unittest.TestCase):
    def test_active_stable_canary_contract(self):
        self.assertEqual([retry.delay_ms(n) for n in range(1, 5)], [100, 200, 250, 250])

    def test_later_attempts_remain_bounded(self):
        self.assertTrue(all(retry.delay_ms(n) == 250 for n in range(3, 21)))

    def test_attempt_domain(self):
        for value in [0, -1, 21]:
            with self.assertRaises(ValueError):
                retry.delay_ms(value)


if __name__ == "__main__":
    unittest.main()
