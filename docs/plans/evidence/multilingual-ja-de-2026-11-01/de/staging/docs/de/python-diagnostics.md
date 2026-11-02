# Python-Diagnostik und Aktualisierungsrichtlinie

Ein erfolgreicher Build liefert BuildOutput mit einem garantierten Artefakt. Ein fehlgeschlagener Build löst BuildError mit Beobachtungen und Veröffentlichungsfakten aus. Diagnosen allein bestimmen niemals den Erfolg.

```python
import json
from ankiforge import Project, Note, BuildOptions, BuildError

project = Project('course').add('term', Note.basic('Question', 'Answer'))
try:
    output = project.build(BuildOptions.to('course.apkg'))
except BuildError as error:
    print(error.kind, error.code)
    print(json.dumps(error.snapshot(), ensure_ascii=False))
else:
    print(output.artifact.path)
    print(json.dumps(output.report.snapshot(), ensure_ascii=False))
```

Verwende für Entscheidungen die maschinenlesbaren Angaben `kind` und `code`, statt Meldungen zu parsen. Alle öffentlichen fachlichen Ausnahmen erhalten den Text der Ursachenverkettung in `causes` und verketten ihre native Ausnahme. `details` enthält vorgangsspezifische Nachweise: Quellpositionen in Vorlagen, Medienpfade und Budgetmessungen, unvollständige Vergleichsbeobachtungen oder Veröffentlichungsfakten. Eine E/A-Ursache wird, soweit verfügbar, zu einem verketteten OSError. `source_details` erhält typisierte verschachtelte Fehler: Ursachen für Textbudgetfehler in Vorlagenpaketen enthalten `limit` und `observed`; Medienursachen enthalten `code`, `path` und `limit_exceeded`.

BuildReport enthält nur Beobachtungen. Sein Snapshot enthält Anzahlen, Diagnosen, verstrichene Zeit und einen gegebenenfalls abgeschlossenen Vergleich. Output- und BuildError-Snapshots enthalten das tatsächliche Success-/Failure-Ergebnis. Fehler-Snapshots unterscheiden ein nie veröffentlichtes Ziel von einem Ziel, das vor einem Fehler bei der dauerhaften Speicherung veröffentlicht wurde. JSON-Pfade besitzen keine Dateien und verlängern die Lebensdauer temporärer Artefakte nicht.

<!-- section: compare-before-publication -->
## Vor der Veröffentlichung vergleichen

```python
from ankiforge import CompareOptions, UpdatePolicy

comparison = project.compare(CompareOptions.against('course-v1.apkg'))
print(comparison.findings)
print(comparison.policy)
```

Ein abgeschlossener Vergleich kehrt normal zurück, auch wenn er ein High-Risiko entdeckt. Prüfe `comparison.allows_publication`, um die Richtlinienentscheidung zu sehen. Eine unlesbare Basisversion, fehlende vollständige Identitätsnachweise, ein ungültiger Kandidat oder ein überschrittenes Budget lösen CompareError aus; sein `.report` enthält die vor dem Fehler abgeschlossenen Beobachtungen.

Die Standardrichtlinie blockiert Befunde der Stufen High und Critical. Nach der Prüfung einer bestimmten Risikokategorie erfolgt ihre Freigabe ausdrücklich:

```python
policy = UpdatePolicy().allow('RISK.NOTE_REMOVED')
comparison = project.compare(CompareOptions.against('course-v1.apkg').update_policy(policy))
output = project.build(BuildOptions.to('course-v2.apkg')
    .update_from('course-v1.apkg').update_policy(policy))
```

Die Freigabe umfasst jeden Befund der genannten Kategorie. Die Befunde behalten ihr ursprüngliches Risiko und ihre Nachweise. Freigaben ohne passenden Befund erzeugen Warnungen; Warnungen machen aus einer erfolgreichen Veröffentlichung keinen Fehler. Unbekannte Codes, fehlende Nachweise und andere harte Fehler lassen sich nicht akzeptieren. Akzeptiere nicht automatisch jede von einem Vergleich zurückgegebene Kategorie.

`update_from` verlangt das ursprünglich verteilte Paket der Vorgängerversion einschließlich seiner vollständigen eingebetteten Nachweise. Erneute Exporte aus Anki erhalten diese Nachweise nicht. Im APKG weggelassene Notizen oder Karten werden nicht automatisch aus der Sammlung eines Lernenden gelöscht. Prüfe strukturelle Änderungen anhand der vorgesehenen Anki-Importeinstellungen.

<!-- section: budgets-and-artifact-ownership -->
## Budgets und Artefaktbesitz

```python
from ankiforge import InspectLimits

limits = InspectLimits(max_archive_bytes=3 << 30)
options = BuildOptions.to('course-v2.apkg').update_from('course-v1.apkg').inspect_limits(limits)
```

Budgets gelten getrennt für jede inspizierte Basisversion und jeden Kandidaten. Der Medienimport hat sein eigenes vorgelagertes MediaLimits-Budget. Budgetfehler erhalten Ressource, Limit und beobachtete Werte unter `details['limit_exceeded']`; erhöhe das betreffende Limit gezielt und versuche es erneut. Verwende für Vorlagenpakete `NoteType.from_bundle(path, limits=MediaLimits(...))`, um das Lesebudget jeder Mediendatei anzupassen. Manifest- und Vorlagentexte behalten ihre separaten festen Limits. Erforderliche Inspektionen lassen sich nicht abschalten.

Verwende `BuildOptions.temporary()`, wenn kein persistenter Pfad gewünscht ist. Halte die Ausgabe oder ein Artefakthandle, solange du die Datei verwendest; `artifact.persist_to(path)` erzeugt eine persistente Kopie. Wenn die Speicherung fehlschlägt, erfassen die PersistError-Details die Veröffentlichungsphase des Ziels, und das ursprüngliche Handle bleibt nutzbar.

Pfade in Snapshots und Fehlerdetails sind bei gültigem Unicode Zeichenfolgen. Unix-Bytenamen, die Python mit `surrogateescape` darstellt, erscheinen in JSON als `{"encoding": "unix_bytes", "bytes": [...]}`; stelle sie mit `Path(os.fsdecode(bytes(value["bytes"])))` wieder her. Windows-Namen mit ungepaarten UTF-16-Surrogaten verwenden `{"encoding": "windows_wide", "units": [...]}`, wobei jeder Arrayeintrag eine ursprüngliche 16-Bit-Codeeinheit enthält. Kodierte Pfade beschreiben ihre ursprüngliche Plattform. Zur Laufzeit bleibt `artifact.path` ein `pathlib.Path`, und `BuildOptions.to`, `update_from` sowie `CompareOptions.against` erhalten native Pfadwerte durch das Binding hindurch.

Eine fehlende oder unpassende native Erweiterung löst einen Importfehler mit `BINDING.EXTENSION_UNAVAILABLE` oder `BINDING.VERSION_MISMATCH` aus. Installiere ein passendes Wheel oder baue den Quellcode mit maturin neu. Gleichzeitige Vorgänge auf einem Project werden zurückgewiesen; verwende unabhängige Projekte für parallele Arbeit.
