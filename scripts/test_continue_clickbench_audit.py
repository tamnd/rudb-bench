"""An incomplete comparison must replace all cases, not keep good fragments."""
import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('continuation', Path(__file__).with_name('continue-clickbench-audit.py'))
continuation = importlib.util.module_from_spec(spec)
spec.loader.exec_module(continuation)


class CompleteGroupTests(unittest.TestCase):
    def setUp(self):
        self.rows = [dict(engine=e, run=r, status='ok', query_s=.001)
                     for e in continuation.ENGINES for r in range(8)]

    def test_all_cases_and_all_rounds_are_required(self):
        self.assertTrue(continuation.complete_group(self.rows, 7))
        self.assertFalse(continuation.complete_group(self.rows[:-1], 7))
        self.assertFalse(continuation.complete_group(self.rows[:8], 7))

    def test_duplicate_round_cannot_replace_a_missing_round(self):
        rows = copy.deepcopy(self.rows)
        rows[-1]['run'] = 6
        self.assertFalse(continuation.complete_group(rows, 7))

    def test_failure_or_missing_timer_invalidates_the_whole_group(self):
        rows = copy.deepcopy(self.rows)
        rows[-1]['status'] = 'query_error'
        self.assertFalse(continuation.complete_group(rows, 7))
        rows[-1].update(status='ok', query_s=None)
        self.assertFalse(continuation.complete_group(rows, 7))


if __name__ == '__main__':
    unittest.main()
