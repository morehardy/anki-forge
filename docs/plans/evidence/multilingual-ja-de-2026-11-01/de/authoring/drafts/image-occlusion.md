# Bildverdeckung

Erzeuge Anki-Karten mit Bildverdeckung per Code: Du lieferst ein Bild und Rechteckkoordinaten, und Anki Forge erstellt eine importfertige Notiz mit einer Karte pro Maske. So kannst du Diagrammfragen stapelweise erzeugen, ohne Masken in Ankis Editor zu zeichnen. Die Bibliothek erkennt keine Beschriftungen im Bild; die Koordinaten gibst du vor.

Erstelle eine strukturierte Notiz zur Bildverdeckung aus einem Bild mit eigenem Datenbesitz und Rechtecken. Jede Maske benötigt einen stabilen Schlüssel, der auch in späteren Versionen denselben Bereich bezeichnet. Die Notiz wird wie gewöhnlich mit `add(key, note)` einem Projekt hinzugefügt. Diese Beispiele verwenden Rust; Python und Node.js bieten denselben Builder in der [Python-API](/docs/python-api/#media-and-occlusion) und [Node-API](/docs/node-api/#media-and-image-occlusion).

<!-- section: run-the-example -->
## Das Beispiel ausführen

[[CODE0]]

Öffne `target/docs-examples/diagram.apkg` in Anki. Das vollständige Programm lädt das Bild für die Bildverdeckung aus dem Repository und prüft eine Notiz, eine Karte und eine Mediendatei.

<!-- source: anki_forge/examples/docs_workflow.rs#occlusion -->
[[CODE1]]
<!-- /source -->

<!-- section: multiple-masks-and-modes -->
## Mehrere Masken und Modi

Dieses eigenständige Beispiel erwartet `fixtures/occlusion.png`, ein dekodiertes Bild mit mindestens 80 × 80 Pixeln:

[[CODE2]]

`HideAllGuessOne` ist die Standardeinstellung; `HideOneGuessOne` wird ebenfalls unterstützt. Jede Maske mit Schlüssel erzeugt ihre eigene Cloze-Karte. Koordinaten sind endliche Pixelwerte im angezeigten Bild nach Anwendung der EXIF-Ausrichtung. Die Bibliothek dekodiert PNG, JPEG, GIF, WebP und BMP vollständig. Das Budget des dekodierten Puffers beträgt 256 MiB und ist vom Importbudget für komprimierte Medien unabhängig. Rechtecke müssen positive Abmessungen haben, innerhalb des Bildes liegen und eindeutige, nicht leere Schlüssel verwenden.

Der Builder validiert Bild und Masken, bevor er eine `Note` zurückgibt. Verwende gewöhnliche `field`-Zuweisungen für `header`, `back_extra` und `comments`; die erzeugten Felder `image` und `occlusion` sind reserviert und dürfen nicht überschrieben werden.

Bei Aktualisierungen auf Basis einer früheren Verteilung erhalten Maskenschlüssel die Kartennummern auch bei geänderter Reihenfolge. Entfernte Masken reservieren ihre Nummern; wird derselbe Schlüssel wiederhergestellt, wird seine Nummer erneut verwendet. Neue Schlüssel übernehmen nie die Identität einer anderen Maske. Der unterstützte Nummernbereich ist 1–500; ist er ausgeschöpft, schlägt der Vorgang ausdrücklich fehl. Entfernte oder hinzugefügte Masken erscheinen in der [Risikoanalyse für Aktualisierungen](/docs/updates/). Das Weglassen einer Karte im APKG löscht die vorhandene Karte eines Lernenden nicht.
