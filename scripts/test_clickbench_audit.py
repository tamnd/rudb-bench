"""Checks that measurement failures and per-child resource accounting stay honest."""
import importlib.util
import os
import re
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('audit', Path(__file__).with_name('clickbench-audit.py'))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


@unittest.skipUnless(sys.platform in ('linux', 'darwin'), 'wait4 resource units')
class MeasurementTests(unittest.TestCase):
    def test_a_measurement_cannot_reenable_the_parquet_mirror(self):
        with tempfile.TemporaryDirectory() as d, mock.patch.dict(os.environ, {'RUDB_PARQUET_MIRROR': '1'}):
            prefix = Path(d) / 'mirror'
            result = audit.measure([sys.executable, '-c',
                                    "import os; print(os.environ['RUDB_PARQUET_MIRROR'])"],
                                   prefix, 10)
            self.assertEqual(result['status'], 'ok')
            self.assertEqual(prefix.with_suffix('.stdout').read_text().strip(), '0')

    @unittest.skipUnless(sys.platform == 'darwin', 'BSD time units')
    def test_wait4_agrees_with_bsd_time_rss_for_the_same_child(self):
        with tempfile.TemporaryDirectory() as d:
            prefix = Path(d) / 'bsd-time'
            result = audit.measure(['/usr/bin/time', '-l', sys.executable,
                                    '-c', 'x=bytearray(64*1024*1024)'], prefix, 10)
            said = prefix.with_suffix('.stderr').read_text()
            peak = int(re.search(r'(\d+)\s+maximum resident set size', said).group(1))
            self.assertEqual(result['peak_rss_bytes'], peak)
            self.assertIsNone(result['read_bytes'])
            self.assertIsNone(result['write_bytes'])

    def test_peak_is_per_child_and_not_the_previous_high_water_mark(self):
        with tempfile.TemporaryDirectory() as d:
            big = audit.measure([sys.executable, '-c', 'x=bytearray(64*1024*1024)'], Path(d)/'big', 10)
            small = audit.measure(['/usr/bin/true'], Path(d)/'small', 10)
            self.assertEqual(big['status'], 'ok')
            self.assertGreater(big['peak_rss_bytes'], 64*1024*1024)
            self.assertLess(small['peak_rss_bytes'], 8*1024*1024)
            self.assertGreater(big['cpu_s'], 0)

    def test_failure_and_timeout_are_not_query_timings(self):
        with tempfile.TemporaryDirectory() as d:
            failed = audit.measure(['/usr/bin/false'], Path(d)/'fail', 10)
            timeout = audit.measure([sys.executable, '-c', 'import time; time.sleep(10)'], Path(d)/'timeout', .1)
            self.assertEqual(failed['status'], 'error')
            self.assertIsNone(failed['query_s'])
            self.assertEqual(timeout['status'], 'timeout')
            self.assertLess(timeout['orchestrator_wall_s'], 2)
            self.assertIsNone(timeout['peak_rss_bytes'])

    @unittest.skipUnless(sys.platform == 'linux', 'GNU time units')
    def test_wait4_agrees_with_gnu_time_for_the_same_child(self):
        with tempfile.TemporaryDirectory() as d:
            report = Path(d)/'gnu-time.txt'
            command = ['/usr/bin/time', '-v', '-o', str(report), sys.executable,
                       '-c', 'x=bytearray(64*1024*1024); sum(i*i for i in range(2000000))']
            result = audit.measure(command, Path(d)/'cross-check', 10)
            fields = dict(line.strip().split(': ', 1) for line in report.read_text().splitlines() if ': ' in line)
            peak = int(fields['Maximum resident set size (kbytes)'])*1024
            cpu = float(fields['User time (seconds)']) + float(fields['System time (seconds)'])
            self.assertEqual(result['peak_rss_bytes'], peak)
            # GNU truncates each CPU component to hundredths; wait4 includes the
            # tiny GNU time parent as well. This compares one run, not two runs.
            self.assertLess(abs(result['cpu_s']-cpu), .025)

    def test_large_integer_answers_are_not_rounded_to_floats(self):
        self.assertFalse(audit.equal([['9223372036854775807']], [['9223372036854775806']]))
        self.assertTrue(audit.equal([['1.00000000001']], [['1.0']]))
        self.assertFalse(audit.equal([['a']], [['b']]))


if __name__ == '__main__':
    unittest.main()
