"""Check positional and pairwise balance in fresh-process comparisons."""
import importlib.util
from pathlib import Path
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('fresh', Path(__file__).with_name('fresh-query-audit.py'))
fresh = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fresh)


class OrderingTests(unittest.TestCase):
    def test_every_case_occupies_every_position_equally(self):
        for count in [2, 3, 4, 6]:
            cases = list(range(count))
            orders = [fresh.run_order(cases, run) for run in range(2*count)]
            for case in cases:
                self.assertEqual([sum(order[position] == case for order in orders)
                                  for position in range(count)], [2]*count)

    def test_each_pair_runs_first_equally_often(self):
        for count in [2, 3, 4, 6]:
            cases = list(range(count))
            orders = [fresh.run_order(cases, run) for run in range(2*count)]
            for left in cases:
                for right in cases[left+1:]:
                    self.assertEqual(sum(order.index(left) < order.index(right) for order in orders), count)


class CapacityTests(unittest.TestCase):
    def test_disabled_limit_never_sleeps(self):
        with mock.patch.object(fresh.audit.time, 'sleep') as sleep:
            fresh.audit.wait_for_capacity(None, 60)
            sleep.assert_not_called()

    def test_busy_host_waits_until_load_is_below_limit(self):
        with mock.patch.object(fresh.audit.os, 'getloadavg', side_effect=[(9, 0, 0), (9, 0, 0), (7, 0, 0)]), \
             mock.patch.object(fresh.audit.time, 'sleep') as sleep, \
             mock.patch('builtins.print'):
            fresh.audit.wait_for_capacity(8, 60)
            sleep.assert_called_once_with(1)

    def test_busy_host_timeout_stops_the_run(self):
        with mock.patch.object(fresh.audit.os, 'getloadavg', return_value=(9, 0, 0)), \
             mock.patch.object(fresh.audit.time, 'monotonic', side_effect=[0, 61]):
            with self.assertRaisesRegex(RuntimeError, 'incomplete'):
                fresh.audit.wait_for_capacity(8, 60)


if __name__ == '__main__':
    unittest.main()
