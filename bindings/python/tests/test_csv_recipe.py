"""Run against either an installed public wheel or the source-built Python SDK."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

from ankiforge import CompareOptions, Note, Project


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / 'bindings/python/examples/csv_to_anki.py'
spec = importlib.util.spec_from_file_location('csv_recipe', SCRIPT)
recipe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recipe)


class CsvRecipeTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.source = self.directory / 'cards.csv'
        self.output = self.directory / 'deck.apkg'

    def assert_package(self, rows, namespace='csv-vocabulary', deck='Vocabulary'):
        with zipfile.ZipFile(self.output) as package:
            self.assertIsNone(package.testzip())
        expected = Project(namespace, default_deck=deck)
        for key, front, back in rows:
            expected.add(key, Note.basic(front, back))
        comparison = expected.compare(CompareOptions.against(self.output)).snapshot()
        self.assertEqual(comparison['baseline_counts'], {'notes': len(rows), 'cards': len(rows), 'media': 0})
        self.assertEqual(comparison['findings'], [])

    def test_documented_command_builds_expected_cards(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(ROOT / 'docs/recipes/csv-to-anki/cards.csv'),
             str(self.output), '--namespace', 'spanish-vocabulary', '--deck', 'Spanish'],
            check=True, capture_output=True, text=True,
        )
        self.assertIn('3 notes, 3 Basic cards', result.stdout)
        self.assert_package([('es:hola', 'hola', 'hello'), ('es:gracias', 'gracias', 'thank you'),
                             ('es:adios', 'adiós', 'goodbye')], 'spanish-vocabulary', 'Spanish')

    def test_spreadsheet_bom_quotes_newlines_and_unicode_survive_export(self):
        self.source.write_text('\ufeffid,front,back\r\nzh:one,"你好, 世界","a ""quoted"" answer"\r\n'
                               'zh:two,"two\nlines",谢谢\r\n', encoding='utf-8')
        recipe.build_csv(self.source, self.output, 'csv-vocabulary', 'Vocabulary')
        self.assert_package([('zh:one', '你好, 世界', 'a "quoted" answer'), ('zh:two', 'two\nlines', '谢谢')])

    def test_row_order_does_not_change_note_identity(self):
        rows = [('es:hola', 'hola', 'hello'), ('es:gracias', 'gracias', 'thank you')]
        self.source.write_text('id,front,back\nes:gracias,gracias,thank you\nes:hola,hola,hello\n')
        recipe.build_csv(self.source, self.output, 'csv-vocabulary', 'Vocabulary')
        self.assert_package(rows)

    def test_invalid_input_never_publishes_a_partial_deck(self):
        cases = [
            '', 'id,front,back\n', 'front,back\na,b\n',
            'id,front,back\nx,a,b\nx,c,d\n', 'id,front,back\nx,a,b\n x ,c,d\n',
            'id,front,back\nx,a\n', 'id,front,back\nx,a,b,extra\n',
            'id,front,back\nx, ,b\n', 'id,front,back\nx,"unterminated,b\n',
        ]
        for source in cases:
            with self.subTest(source=source):
                self.source.write_text(source)
                with self.assertRaises((ValueError, recipe.csv.Error)):
                    recipe.build_csv(self.source, self.output, 'csv-vocabulary', 'Vocabulary')
                self.assertFalse(self.output.exists())

    def test_existing_distribution_is_preserved(self):
        self.output.write_bytes(b'previously distributed file')
        with self.assertRaises(ValueError):
            recipe.build_csv(self.source, self.output, 'csv-vocabulary', 'Vocabulary')
        self.assertEqual(self.output.read_bytes(), b'previously distributed file')


if __name__ == '__main__':
    unittest.main()
