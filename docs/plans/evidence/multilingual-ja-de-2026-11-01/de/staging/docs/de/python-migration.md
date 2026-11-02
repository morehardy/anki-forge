# Die neu aufgebaute Python-API

Verwende `python -m pip install ankiforge` und `from ankiforge import ...`. Die frühere Distribution `anki-forge` und der Importname `anki_forge` sind keine Kompatibilitätsaliase. Wenn du aus dem Quellcode arbeitest, baue die native Erweiterung nach Namensänderungen mit Maturin neu. Siehe die [Einrichtungsanleitung](https://github.com/morehardy/anki-forge/blob/main/bindings/python/README.md).

Es bleibt keine Kompatibilitätsschicht für die alte API erhalten. Ersetze die alten Deck-, Registry- und Runtime-APIs durch das öffentliche Modell mit eigenem Datenbesitz:

| Bisheriger Ablauf | Aktueller Ablauf |
| --- | --- |
| Deck oder Project mit optionaler stabiler ID | `Project(namespace, default_deck=...)` |
| `add_note(note)` und inhaltsbasierte Identität | `add(stable_key, note)` |
| Veränderlicher NoteType und add_notetype | `NoteType.builder(key)...build()`, danach `model.note()` |
| Modellreferenzen als Zeichenfolgen | Notizen besitzen ihr validiertes Modell |
| Medienregistrierung im Project | Unabhängige Snapshots mit `Media.file` / `Media.bytes` |
| Direkt gerenderte Medien und feldspezifische Setter | `Content.sequence`, `media.image()`, `media.sound()`, `Note.field` |
| Implizites HTML | Zeichenfolgen sind Text; verwende `Content.html` |
| Masken ohne stabile Identität | `Mask.rect(key, x, y, width, height)` |
| Optionales Artefakt und ensure_success | `BuildOutput` mit garantiertem Artefakt; Fehler lösen strukturierte Ausnahmen aus |
| Aus Diagnosen erratenes Berichtsergebnis | Output-/Error-Snapshots liefern das tatsächliche Ergebnis; Report enthält nur Beobachtungen |
| Diff, Lockfile und Schalter zur Aktualisierungssicherheit | `compare(CompareOptions.against(...))`, `BuildOptions.update_from(...)` |
| Laufzeitpaket mit Unterprozess | PyO3-Bindung an die standardmäßige Rust-API im selben Prozess |

Nur vollständige, ursprünglich verteilte APKGs mit aktuellen Identitätsnachweisen eignen sich als Aktualisierungsbasis. Alte Bundles und alte APKG-Nachweise werden nicht automatisch wiederhergestellt. Den neuen Ablauf zeigen die README und die ausführbaren Beispiele.
