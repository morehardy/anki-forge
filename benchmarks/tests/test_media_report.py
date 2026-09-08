import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import media_report


def sessions():
    result = []
    for session in (1, 2, 3):
        rows = []
        for profile, _ in media_report.PROFILES:
            for size in media_report.SIZES:
                for adapter in media_report.ADAPTERS:
                    for role, repeats in media_report.ROLES.items():
                        for repeat in range(repeats):
                            measured_ms = session * (10 if adapter == 'rust' else 20)
                            rows.append({
                                'id': f'{profile}-{size}-{adapter}-{role}-{repeat}',
                                'profile': profile, 'size': size, 'adapter': adapter,
                                'role': role, 'status': 'success',
                                'artifact_bytes': 1000 if adapter == 'rust' else 2000,
                                'measurement': {'elapsed_ns': (10000 if 'warmup' in role else measured_ms)*10**6,
                                                'peak_rss_bytes': (20 if adapter == 'rust' else 10)*2**20,
                                                'exit_code': 0, 'signal': 0, 'leftover_descendants': False},
                            })
        result.append(rows)
    return result


class MediaReportTests(unittest.TestCase):
    def test_power_checks_cover_every_attempt_and_reject_source_changes(self):
        expected = "Now drawing from 'Battery Power'"
        attempts = [{'id': 'first'}, {'id': 'second'}]
        records = [{'id': row['id'], 'before': expected + '\n75%', 'after': expected + '\n74%'} for row in attempts]
        media_report.validate_power(records, attempts, expected)
        for changed in (records[:1], records + records[:1],
                        [records[0], {**records[1], 'after': "Now drawing from 'AC Power'"}],
                        [records[0], {'id': 'second', 'before': expected}]):
            with self.subTest(records=changed), self.assertRaises(ValueError):
                media_report.validate_power(changed, attempts, expected)

    def test_dirty_sources_require_an_exact_predeclared_snapshot(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            plan = {'source_commit': 'base', 'git_status': ' M runtime.rs'}
            manifest = {**plan, 'identity_before': {'source_files': {'runtime.rs': 'digest'}}}
            with self.assertRaisesRegex(ValueError, 'clean predeclared'):
                media_report.validate_source(root, plan, manifest)
            snapshot = {**plan, **manifest['identity_before']}
            (root/'source-snapshot.json').write_text(json.dumps(snapshot))
            (root/'source.patch').write_text('the exact patch')
            plan['source_snapshot'] = {name: hashlib.sha256((root/name).read_bytes()).hexdigest()
                                       for name in ('source-snapshot.json', 'source.patch')}
            media_report.validate_source(root, plan, manifest)
            changed = copy.deepcopy(manifest)
            changed['identity_before']['source_files']['runtime.rs'] = 'changed'
            with self.assertRaisesRegex(ValueError, 'worktree snapshot'):
                media_report.validate_source(root, plan, changed)
            (root/'source.patch').write_text('changed patch')
            with self.assertRaisesRegex(ValueError, 'frozen source changed'):
                media_report.validate_source(root, plan, manifest)

    def test_pools_equal_sessions_and_excludes_warmups(self):
        cells = media_report.aggregate(sessions())
        self.assertEqual(len(cells), 20)
        cell = cells[0]
        self.assertEqual(cell['rust']['time_ms']['n'], 30)
        self.assertEqual(cell['rust']['rss_mib']['n'], 15)
        self.assertEqual(cell['rust']['time_ms']['median'], 20)
        self.assertEqual(cell['genanki']['time_ms']['median'], 40)
        self.assertEqual(cell['time_saved_pct'], 50)
        self.assertEqual(cell['round_time_saved_range_pct'], [50, 50])
        self.assertEqual(cell['rss_mib_saved_pct'], -100)
        self.assertEqual(cell['apkg_bytes_saved_pct'], 50)

    def test_slower_rust_and_session_variation_remain_visible(self):
        rows = sessions()
        for row in rows[2]:
            if row['adapter'] == 'rust' and row['role'] == 'timing':
                row['measurement']['elapsed_ns'] *= 3
        cell = media_report.aggregate(rows)[0]
        self.assertEqual(cell['rounds'][2]['time_saved_pct'], -50)
        self.assertEqual(cell['round_spread_pp'], 100)
        self.assertTrue(cell['variable_across_rounds'])

    def test_incomplete_failed_or_duplicate_attempts_reject_entire_aggregate(self):
        original = sessions()
        for kind in ('missing', 'failed', 'duplicate'):
            rows = copy.deepcopy(original)
            if kind == 'missing':
                rows[0].pop()
            elif kind == 'failed':
                rows[0][-1]['status'] = 'failed'
            else:
                rows[0][-1] = rows[0][-2]
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                media_report.aggregate(rows)
        with self.assertRaises(ValueError):
            media_report.aggregate(original[:2])

    def test_changed_archived_evidence_is_rejected_before_reporting(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original = b'original plan'
            (root/'plan.json').write_bytes(b'changed plan')
            (root/'evidence-sha256.json').write_text(json.dumps({'plan.json': hashlib.sha256(original).hexdigest()}))
            with self.assertRaisesRegex(ValueError, 'evidence changed: plan.json'):
                media_report.load_evidence(root)


if __name__ == '__main__':
    unittest.main()
