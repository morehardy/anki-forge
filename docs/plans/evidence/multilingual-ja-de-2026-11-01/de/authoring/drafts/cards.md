# Basic- und Cloze-Karten

Erzeuge Basic-Karten mit Frage und Antwort sowie Cloze-Karten mit Lückentext aus Code, statt sie einzeln in Anki anzulegen. Eine Basic-Notiz hat eine Vorder- und eine Rückseite. Eine Cloze-Notiz erzeugt für jede unterstützte Lückennummer eine Karte. Dieses Rust-Beispiel erstellt zwei Notizen und drei Karten. Python und Node.js bieten dieselben Konstruktoren `Note.basic` und `Note.cloze`; siehe [Python-API](/docs/python-api/#project-and-note-values) und [Node-API](/docs/node-api/#project-and-notes).

Führe im Stammverzeichnis des Repositorys Folgendes aus:

[[CODE0]]

Öffne `target/docs-examples/cards.apkg` in Anki. Die Basic-Frage lautet **hola**; die Cloze-Karten verbergen getrennt **Madrid** und **Spain**.

<!-- section: author-the-notes -->
## Notizen erstellen

Diese Funktion stammt aus dem [vollständigen ausführbaren Programm](https://github.com/morehardy/anki-forge/blob/main/anki_forge/examples/docs_workflow.rs). Die Hilfsfunktion `verify` prüft die Anzahl der Notizen, Karten und Medien in der erfolgreichen Ausgabe.

<!-- source: anki_forge/examples/docs_workflow.rs#cards -->
[[CODE1]]
<!-- /source -->

Das erste Argument von `Project::new` ist ein stabiler Namensraum. Jeder Aufruf von `add` benötigt einen stabilen Schlüssel des Quelldatensatzes. Wird derselbe Schlüssel für eine andere Notiz verwendet, schlägt der Vorgang fehl; es wird kein automatischer, inhaltsbasierter Schlüssel zugewiesen. Eine geänderte Antwort unter demselben Schlüssel bleibt dieselbe logische Notiz, wenn du eine Aktualisierung auf Basis ihrer vorherigen Verteilung erstellst.

<!-- section: text-fields-and-card-identity -->
## Text, Felder und Kartenidentität

Zeichenfolgen für Basic und Cloze sind Text. Verwende `Content::html`, um HTML bewusst einzubinden; Cloze-Lückensyntax wird weiterhin innerhalb von Text erkannt. `back_extra` ist der eingebaute Schlüssel des zusätzlichen Cloze-Feldes. Verwende dieselbe Methode `Note::field` für Text, HTML, Bilder und Audio.

Verwende `{{c1::answer}}` oder `{{c1::answer::hint}}`. Ein einfacher Satz enthält keine Cloze-Frage. Unterstützte Lückennummern reichen von 1 bis 500; Änderungen der Nummern können die Kartenidentität verändern. Prüfe solche Änderungen mit [Vergleich und Aktualisierungsrichtlinie](/docs/updates/).

Ergänze als Nächstes [Medien](/docs/media/), ein [eigenes Modell](/docs/custom-notetypes/) oder [Bildverdeckung](/docs/image-occlusion/).
