Anki Forge erzeugt aus Notizen, Vorlagen und Medien Anki-Dateien im Format `.apkg`. Die Bibliothek eignet sich, wenn deine Lerninhalte in Code oder strukturierten Daten vorliegen und du Stapel wiederholt erzeugen und verteilen möchtest.

<!-- section: start-with-your-language -->
## Mit deiner Programmiersprache beginnen

Installiere ein veröffentlichtes Paket und exportiere deinen ersten Stapel. [Wähle den passenden Einstieg](/docs/languages/).

| Sprache | Erstes Ergebnis | Nächster Schritt |
| --- | --- | --- |
| Rust | [Das Beispiel für den ersten Stapel ausführen](/docs/quickstart/) | [In deine Anwendung integrieren](/docs/installation/) |
| Node / TypeScript | [Installieren und den ersten Stapel erzeugen](/docs/node-quickstart/) | [API und asynchrones Verhalten](/docs/node-api/) |
| Python | [Installieren und einen Stapel exportieren](/docs/python-quickstart/) | [API und Lebensdauer der Ausgabedateien](/docs/python-api/) |

Möchtest du zuerst das Ergebnis sehen? [Sieh dir die Kartenbeispiele an](/examples/) und lade die erzeugten Pakete herunter.

<!-- section: understand-the-pieces -->
## Die Bestandteile verstehen

Eine Notiz enthält Felder. Ein Notiztyp erzeugt mit seinen Vorlagen Karten aus diesen Feldern. Stapel gruppieren die entstandenen Karten in Anki. Die [Grundbegriffe](/docs/concepts/) erklären diese Zusammenhänge, den Datenbesitz von `Project`-Werten und die stabile Identität.

<!-- section: build-your-next-feature -->
## Die nächste Funktion umsetzen

- [CSV zu Anki](/docs/csv-to-anki/): Erzeuge mit Python ein APKG aus einer Vokabeltabelle.
- [JSON zu Anki](/docs/json-to-anki/): Erzeuge Karten mit Medien und prüfe eine Aktualisierung in drei Programmiersprachen.
- [Karten mit KI-Unterstützung erstellen](/docs/ai-anki-cards/): Erstelle Kartenentwürfe aus deinen Aufzeichnungen, prüfe die Antworten und exportiere ein APKG.
- [Basic und Cloze](/docs/cards/): Erstelle Frage-Antwort-Karten und Lückentextkarten.
- [Bilder und Audio](/docs/media/): Registriere echte Mediendateien und füge sie den Karten hinzu.
- [Eigene Notiztypen](/docs/custom-notetypes/) und [Vorlagenpakete](/docs/templates/): Definiere Felder, Layouts und CSS.
- [Bildverdeckung](/docs/image-occlusion/): Erzeuge Fragen zu Bereichen eines Bildes.
- [Aktualisieren und verteilen](/docs/updates/): Übernimm Identitäts- und Revisionsnachweise von einer Veröffentlichung zur nächsten.

Die Anleitungen zu Kartentypen verwenden ausführbare Rust-Beispiele. Die CSV-Anleitung verwendet Python; die JSON-Anleitung deckt alle drei Sprachen ab. Die API-Seiten für Node und Python erklären die jeweiligen Schnittstellen und Unterschiede.

<!-- section: find-answers-and-contribute -->
## Antworten finden und mitarbeiten

Nutze die [Fehlersuche](/docs/troubleshooting/), wenn ein Vorgang fehlschlägt. Unter [geprüfte Umgebungen](/docs/compatibility/) findest du Angaben zu unterstützten Laufzeitumgebungen und den Importbedingungen der Anki-Clients.

Anki Forge steht unter der [MIT-Lizenz](https://github.com/morehardy/anki-forge/blob/main/LICENSE). Melde reproduzierbare Fehler auf [GitHub](https://github.com/morehardy/anki-forge/issues) oder lies die [Anleitung für Mitwirkende](https://github.com/morehardy/anki-forge/blob/main/docs/development.md).
