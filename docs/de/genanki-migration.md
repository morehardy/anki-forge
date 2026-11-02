# Von genanki-Konzepten umsteigen

Du wählst noch eine Bibliothek aus? Lies [Anki Forge im Vergleich zu genanki](/docs/anki-forge-vs-genanki/).

Anki Forge verwendet stabile Datenschlüssel, validierte unveränderliche Modelle, Mediensnapshots mit eigenem Datenbesitz und ausdrückliche Build- und Aktualisierungsvorgänge. Das ist ein neues Modell zur Inhaltserstellung; es bildet weder genanki-Objekte noch die entfernten Deck-/Lockfile-Schnittstellen von Anki Forge nach.

| genanki-Konzept | Entsprechung in Anki Forge |
| --- | --- |
| Modell mit numerischer ID | `NoteType.builder(stable_key)...build()` |
| Feldnamen als Bezeichner | Stabile Field-Schlüssel und separate Anzeigenamen |
| Notiz mit Feldzeichenfolgen | `model.note().field(key, content)` mit eigenem Modellbesitz |
| Stapel als Notizcontainer | Project mit Standardstapel; optional `.deck(name)` pro Notiz |
| Dateipfade für Paketmedien | `Media.file(path)`-Werte mit eigenem Datenbesitz, automatisch über typisierte Inhalte gesammelt |
| Paket schreiben | `project.build(BuildOptions.to(path))` |
| Aus Feldern abgeleitete Standard-GUID oder eigene GUID-Regel | Ausdrücklicher stabiler Namensraum und Notizschlüssel |

```python
from ankiforge import Project, NoteType, Field, Template, BuildOptions

model = (NoteType.builder('vocabulary').name('Vocabulary')
    .field(Field('word', name='Word', required=True))
    .field(Field('meaning', name='Meaning'))
    .template(Template('recognition', '{{word}}', '{{FrontSide}}<hr>{{meaning}}'))
    .build())
project = Project('vocabulary-course', default_deck='Languages::Vocabulary')
project.add('dictionary:cell', model.note().field('word', 'cell').field('meaning', '细胞'))
project.build(BuildOptions.to('vocabulary.apkg'))
```

Wähle Schlüssel aus dauerhaften fachlichen Bezeichnern. Das Umbenennen eines Anzeigenamens oder das Ändern eines Definitionstextes darf diese Schlüssel nicht ändern. Bilde keinen Hash aus den aktuellen Feldinhalten, es sei denn, eine inhaltsadressierte Identität ist eine ausdrückliche Anforderung deines Importers. Bei Quelldaten ohne Schlüssel musst du diese außerhalb des SDK zuweisen und dauerhaft speichern.

Vorlagenausdrücke verwenden Field-Schlüssel. Anzeigenamen können unabhängig davon übersetzt werden; die Bibliothek übersetzt Schlüsselreferenzen in Anki-Feldnamen. Beim Abschluss eines Modells werden fehlende oder widersprüchliche Deklarationen zurückgewiesen, bevor eine Notiz hinzugefügt werden kann. Das Hinzufügen von Notizen sammelt Modellabhängigkeiten automatisch und weist widersprüchliche Definitionen unter demselben Modellschlüssel zurück.

Zeichenfolgen sind Klartext; umschließe ausdrücklich gewünschtes HTML mit `Content.html(...)`. Verwende `media.image()` / `media.sound()` und `Content.sequence(...)`, um Inhalte zu kombinieren. Dateisnapshots bleiben erhalten, wenn der ursprüngliche Pfad gelöscht oder verändert wird. Deklariere Abhängigkeiten aus direkt eingebundenem HTML, CSS oder Skripten ausdrücklich mit `.asset(media)` am Modell-Builder oder `project.add_asset(media)`.

Erzeuge für spätere Veröffentlichungen ein neues Project mit demselben Namensraum und denselben Datenschlüsseln und rufe `BuildOptions.to(next_path).update_from(previous_original_apkg)` auf. Das ursprüngliche APKG liefert die vollständigen Identitätsnachweise. Dieser Ablauf stellt keine Identität aus beliebigen älteren Paketen wieder her und verspricht keine automatische Umwandlung vorhandener numerischer genanki-IDs oder GUIDs. Behandle die erste Verteilung mit dem neuen Modell als bewussten Identitätswechsel, sofern nicht eine separat validierte Datenmigration die Zuordnung nachweist.

Siehe [Schnellstart](/docs/python-quickstart/), [API-Referenz](/docs/python-api/) und [Aktualisierungsrichtlinie](/docs/python-diagnostics/).
