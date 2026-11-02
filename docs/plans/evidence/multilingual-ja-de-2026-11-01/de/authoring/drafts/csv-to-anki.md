# CSV mit Python in Anki-Karten umwandeln

Wandle mit Python und Anki Forge eine Vokabeltabelle in eine Anki-Datei im Format `.apkg` um. Dieses Beispiel liest eine UTF-8-CSV-Datei mit den Spalten `id`, `front` und `back`, erstellt Basic-Karten mit Frage und Antwort und exportiert einen teilbaren Stapel. Der CSV-Parser aus Pythons Standardbibliothek liest die Tabelle; Anki Forge erstellt das Paket. Für den Export muss Anki nicht installiert sein.

Für den einmaligen Import einer Tabelle unterstützt Anki bereits [Textdateien mit Trennzeichen](https://docs.ankiweb.net/manual/importing/text-files). Verwende diese Anleitung, wenn du ein wiederholbar ausführbares Skript, eigene Notiz-IDs und ein verteilbares APKG benötigst. Eine Excel-Datei im Format `.xlsx` musst du zunächst als UTF-8-CSV speichern.

<!-- section: prepare-csv -->
## Die CSV-Datei vorbereiten

Speichere Folgendes als `cards.csv` oder [lade das Beispiel herunter](/generated/recipes/csv-to-anki/cards.csv):

<!-- source: docs/recipes/csv-to-anki/cards.csv -->
[[CODE0]]
<!-- /source -->

Übernimm die Kopfzeile unverändert und verwende Kommas als Trennzeichen. Setze Felder mit Kommas, doppelten Anführungszeichen oder Zeilenumbrüchen gemäß den [CSV-Regeln](https://docs.python.org/3/library/csv.html) in Anführungszeichen. Das Skript akzeptiert eine UTF-8-Byte-Order-Mark, wie sie manche Tabellenexporte hinzufügen. Es weist doppelte IDs, fehlende Werte, zusätzliche Spalten und leere Eingaben zurück.

Vergib eine ID einmal und behalte sie bei, wenn sich die Frage oder Antwort ändert. Leite IDs nicht aus der Zeilenposition oder dem veränderlichen Kartentext ab. Verwende für unabhängige Datensätze unterschiedliche Namensräume; behalte den Namensraum für aufeinanderfolgende Versionen dieses Stapels bei.

<!-- section: install -->
## Anki Forge installieren

Verwende gewöhnliches CPython ab Version 3.11 auf einer [unterstützten Plattform](/docs/compatibility/). Erstelle und aktiviere eine virtuelle Umgebung:

[[CODE1]]

Unter Windows aktivierst du sie mit `.venv\Scripts\Activate.ps1`. Für diese Umwandlung benötigst du weder eine Tabellenbibliothek noch einen KI-Dienst.

<!-- section: convert -->
## Das APKG erzeugen

Speichere Folgendes als `csv_to_anki.py` neben `cards.csv` oder [lade das Skript herunter](/generated/recipes/csv-to-anki/csv_to_anki.py):

<!-- source: bindings/python/examples/csv_to_anki.py -->
[[CODE2]]
<!-- /source -->

Führe es in diesem Verzeichnis aus:

[[CODE3]]

Das Beispiel erzeugt `vocabulary-v1.apkg` mit **drei Notizen und drei Basic-Karten** im Stapel **Spanish**, ohne Medien. Es gibt Folgendes aus:

[[CODE4]]

Wähle einen Ausgabepfad, der noch nicht existiert. Das Skript ersetzt keine vorhandene Datei, damit du das ursprünglich verteilte Paket aufbewahren kannst.

<!-- section: verify-and-extend -->
## Importieren, prüfen und erweitern

Öffne `vocabulary-v1.apkg` in Anki. Prüfe, ob der Stapel `hola → hello`, `gracias → thank you` und `adiós → goodbye` enthält. Sieh dir die Kartenvorschau an und prüfe die Akzente. Prüfe den Inhalt vor der Verteilung, besonders wenn eine KI die CSV-Daten entworfen hat.

Diese Anleitung erstellt Basic-Karten mit reinem Text. Für andere Abläufe:

- [Anki-Karten mit KI-Unterstützung erstellen](/docs/ai-anki-cards/): Entwirf und prüfe CSV-Zeilen aus Aufzeichnungen.
- [JSON zu Anki](/docs/json-to-anki/): Verwende Python, Node / TypeScript oder Rust, bündle Bilder und Audio und prüfe eine zweite Veröffentlichung.
- [Basic- und Cloze-Karten](/docs/cards/): Wähle zwischen Frage-Antwort- und Lückentextkarten.
- [Erzeugte Anki-Stapel aktualisieren](/docs/updates/): Verwende das ursprüngliche APKG der Vorgängerversion als Aktualisierungsbasis und prüfe den Vergleich.

Behalte für spätere Veröffentlichungen den Namensraum und die Notiz-IDs bei und durchlaufe vor der Veröffentlichung den Aktualisierungsablauf. Stabile IDs helfen bei der Zuordnung von Notizen; Ankis Importeinstellungen und neuere lokale Änderungen beeinflussen Aktualisierungen weiterhin.
