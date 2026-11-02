from pathlib import Path
import json
BASE=Path(__file__).resolve().parent.parent
ROOT=BASE.parents[4]
meta={
'docs':('Anki-Stapel aus Code erzeugen','Wähle eine Programmiersprache, erzeuge deinen ersten Stapel und ergänze Medien, Vorlagen und wiederholbare Aktualisierungen.'),
'languages':('Deine Programmiersprache wählen','Installiere das veröffentlichte Paket für Rust, Node oder Python und exportiere deinen ersten Anki-Stapel.'),
'quickstart':('Anki-Karten mit Rust erzeugen','Installiere das veröffentlichte Rust-Paket, erstelle eine spanische Vokabelkarte und exportiere mit einem vollständigen Programm ein dauerhaft gespeichertes APKG.'),
'json-to-anki':('Anki-Stapel aus JSON erzeugen und aktualisieren','Lies JSON mit Rust, Python oder TypeScript ein, bündle Bilder und Audio und prüfe eine APKG-Aktualisierung, die nur eine Antwort ändert.'),
'csv-to-anki':('CSV zu Anki: Karten mit Python erzeugen','Wandle eine UTF-8-Vokabeltabelle mit einem vollständigen Python-Skript in ein Anki-APKG um – mit CSV-Beispiel, stabilen Notiz-IDs und Importprüfungen.'),
'ai-anki-cards':('Anki-Karten mit KI-Unterstützung: von Aufzeichnungen zum APKG','Lass einen KI-Assistenten Karten entwerfen, prüfe die Antworten und exportiere mit Anki Forge ein APKG. Nutze CSV oder eine Aufgabenbeschreibung für Programmierassistenten.'),
'anki-forge-vs-genanki':('Anki Forge oder genanki: passend zu deinem Ablauf wählen','Vergleiche die Erstellung von Anki-APKGs, Laufzeitanforderungen, Aktualisierungsprüfungen und Migrationsaufwand.'),
'development':('SDKs aus dem Quellcode bauen','Baue SDKs und Beispiele aus dem Repository und unterscheide Quellcodefunktionen von veröffentlichten Paketen.'),
'rust-guide':('Inhalte mit Rust erstellen','Erstelle Projekte, validiere Notizen und verwalte Medien und Aktualisierungen über die unterstützte Rust-API.'),
 'templates':('Anki-Kartenvorlagen, CSS und Medien wiederverwenden','Lade einen wiederverwendbaren Anki-Notiztyp aus einem Paket mit Kartenvorlagen, CSS und Medien – für normale oder Cloze-Karten mit Rust, Python oder Node.js.'),
'api':('Überblick über die öffentliche API','Finde die Rust-, Python- und Node-APIs für Inhaltserstellung, Medien, Diagnostik und Aktualisierungen.'),
'installation':('Anki Forge in deine Anwendung integrieren','Installiere das veröffentlichte Rust-Crate, konfiguriere Cargo-Abhängigkeiten und wähle die nächste Anleitung.'),
'concepts':('Grundbegriffe','Verstehe Projekte, Notizen und Medien mit eigenem Datenbesitz, stabile Schlüssel, Text und Identität bei Aktualisierungen.'),
'cards':('Basic- und Cloze-Karten für Anki erzeugen','Erzeuge Basic-Karten mit Frage und Antwort sowie Cloze-Lückentextkarten aus Code. Ein geprüftes Rust-Beispiel erstellt zwei Notizen und drei Karten.'),
'custom-notetypes':('Eigene Notiztypen','Definiere Felder, Kartenvorlagen und Erzeugungsregeln für deine Inhalte mit stabilen Schlüsseln und wiederverwendbaren unveränderlichen Modellen.'),
'media':('Anki-Karten um Bilder und Audio ergänzen','Importiere mit Rust, Python oder Node.js Dateien, kombiniere Karteninhalte und verwalte Exportnamen, MIME-Typen und Medienbudgets.'),
'image-occlusion':('Anki-Karten mit Bildverdeckung erzeugen','Erzeuge Bildverdeckungskarten per Code mit Rust, Python oder Node.js: Gib Bild und Rechteckmasken vor und wähle einen der beiden Verdeckungsmodi.'),
'updates':('Anki-Stapel mit stabilen Notiz-IDs aktualisieren','Veröffentliche eine neue Stapelversion: Behalte stabile Notiz-IDs bei, verwende das ursprüngliche APKG der Vorgängerversion als Basis und prüfe den Vergleich vor der Verteilung.'),
'build-guarantees':('Build- und Ausgabegarantien','Verstehe temporäre Artefakte, Pfadschutz, Inspektionsgrenzen und die Veröffentlichung geprüfter Kandidaten.'),
'troubleshooting':('Fehlersuche','Behebe Fehler bei Installation, Notizen, Medien, Vorlagen, Identität und der Veröffentlichung von Ausgabedateien.'),
'compatibility':('Kompatibilität und geprüfte Umgebungen','Prüfe veröffentlichte Pakete, verifizierte Laufzeitumgebungen und Bedingungen für Import und Wiedergabe in Anki-Clients.'),
'rust-api':('Rust-API-Referenz','Finde öffentliche Methoden zur Inhaltserstellung, Build-Vorgaben, Berichte und Diagnoseverhalten in Rust.'),
'node-quickstart':('Anki-Karten mit Node.js erzeugen','Installiere ankiforge über npm, erzeuge Karten mit Node.js oder TypeScript und exportiere einen APKG-Stapel zum Import in Anki.'),
'node-api':('Node- und TypeScript-API','Öffentliche Methoden, asynchrones Verhalten, Build-Optionen, Ausgabedateien und Berichte des Node-SDK.'),
'python-quickstart':('Anki-Karten mit Python erzeugen','Installiere ankiforge über PyPI, erzeuge Karten mit einem kurzen Python-Skript und exportiere einen APKG-Stapel zum Import in Anki.'),
'python-api':('Python-API','Project, Medien, Build-Optionen, Berichte und die Kontrolle über die Lebensdauer temporärer Artefakte in Python.'),
'python-diagnostics':('Python-Diagnostik','Verstehe native Fehler bei der Inhaltserstellung, Build-Berichte, Aktualisierungsnachweise und Risikorichtlinien in Python.'),
'python-migration':('Umstieg auf die neue Python-API','Passe deinen Code an das aktuelle Python-SDK mit eigenem Datenbesitz an und verstehe den Bruch mit der früheren API.'),
'genanki-migration':('Von genanki zu Anki Forge wechseln','Verstehe, wie du Notizen, Modelle, Vorlagen und Medien aus genanki an die stabilen Schlüssel und das Inhaltsmodell von Anki Forge anpasst.')
}
terms={
'docs':['Dokumentationsübersicht','Anki-Stapel aus Code','Notizen erstellen','Anleitungen finden','strukturierte Lerninhalte','wiederholbare Stapelveröffentlichung','Notizfelder und Kartenvorlagen'],
'languages':['Programmiersprache wählen','SDK auswählen','Rust Python Node.js TypeScript','veröffentlichte Pakete installieren','native Laufzeitvoraussetzungen','APKG ohne Anki exportieren'],
'quickstart':['Rust-Schnellstart','erster Anki-Stapel mit Rust','Rust Einstieg','cargo new anki-deck','cargo run','src/main.rs','spanish.apkg','stabiler Schlüssel es:hola'],
'json-to-anki':['JSON zu Anki','JSON in APKG umwandeln','strukturierte Daten','Notizen stapelweise erstellen','JSON.parse','json.loads','serde_json','TypeScript JSON','Bilder und Audio aus JSON','nur die Antwort aktualisieren','comparison.json review.json','dauerhafte fachliche Schlüssel'],
'csv-to-anki':['CSV zu Anki','CSV in APKG umwandeln','Vokabeltabelle','Tabellenimport','Excel als UTF-8-CSV speichern','Basic-Karten mit Python','Spalten id front back','csv_to_anki.py','eindeutige Notiz-IDs','CSV-Kopfzeile und Anführungszeichen','vocabulary-v1.apkg'],
'ai-anki-cards':['Anki-Karten mit KI','künstliche Intelligenz','Kartenentwürfe aus Aufzeichnungen','Fragen und Antworten entwerfen','KI-Antworten prüfen','Programmierassistent','Aufgabenbeschreibung','Quellmaterial prüfen','CSV entwerfen','PDF-Extraktion und OCR außerhalb der Bibliothek'],
'anki-forge-vs-genanki':['Anki Forge im Vergleich zu genanki','Bibliotheksvergleich','genanki Alternative','Bibliothek auswählen','Vor- und Nachteile','Laufzeitaufwand','Migrationsaufwand','historische Exportmessung','Rust und genanki Benchmark'],
'development':['SDK aus dem Quellcode bauen','Repository entwickeln','Quellcode kompilieren','Cargo Pfadabhängigkeit','Node-Addon lokal bauen','maturin develop','Quellcode und veröffentlichte Pakete','lokale Tarballs'],
'rust-guide':['Rust-Anleitung','Project erstellen','Rust Datenbesitz','Modelle und Medien','Notizen atomar hinzufügen','stabile Notizschlüssel','AddError context','Tool-Rezepte ankiforge-project-v2','v1 zurückgewiesen'],
'templates':['Vorlagenpaket','Vorlagen-Bundle','Kartendesign wiederverwenden','CSS und Schriftarten','anki-template.yaml','template-bundle-v2','Cloze-Vorlage','Notiztyp laden','Paketpfade','UTF-8-Bytebudget'],
'api':['öffentliche API','API-Übersicht','Rust Python Node Schnittstellen','Methoden zur Inhaltserstellung','Medien-API','Diagnose-API','Vergleich und Veröffentlichung'],
'installation':['Anki Forge installieren','Rust-Anwendung integrieren','Cargo-Abhängigkeiten','cargo add ankiforge','Rust 1.92','serde serde_json','veröffentlichtes Crate','Linker und Build-Werkzeuge'],
'concepts':['Grundbegriffe','Notiz Karte Stapel','Notiz und Karte unterscheiden','Projekt Namensraum','Felder und Vorlagen','mehrere Karten pro Notiz','APKG-Bestandteile','stabile Identität','Text und HTML','Datenbesitz','Snapshot'],
'cards':['Basic-Karten','Frage-Antwort-Karten','Cloze-Karten','Lückentext','Lueckentext','Lückennummern','zwei Notizen drei Karten','back_extra','Text statt HTML','Kartennummer ändern'],
'custom-notetypes':['eigene Notiztypen','benutzerdefinierte Felder','unveränderliches Modell','Anzeigenamen und Schlüssel','Feldschlüssel','Vorlagenschlüssel','Pflichtfelder','Sortierfeld','bedingte Kartenerzeugung','GenerationRule'],
'media':['Bilder und Audio','Mediendateien importieren','Medienimport im Stapel','Stapelimport','Media.files','Aussprache','Audiodateien','Exportnamen','MIME-Typ','Dateiendung','Medienbudget','Schriftarten','keine Sprachsynthese'],
'image-occlusion':['Bildverdeckung','Bildmasken','Image Occlusion','Rechteckmasken','Diagrammbeschriftung verdecken','Maskenkoordinaten','Maskenschlüssel','HideAllGuessOne','HideOneGuessOne','EXIF-Ausrichtung','Pixelkoordinaten','keine automatische Beschriftungserkennung'],
'updates':['Stapel aktualisieren','Stapelaktualisierung','Notizaktualisierung','Antwort ändern','ursprünglich verteiltes APKG','Basisversion','Aktualisierungsbasis','erneuter Anki-Export','Risikovergleich','Veröffentlichungsrichtlinie','vorbereiten prüfen veröffentlichen','Lernplanung und lokale Änderungen'],
'build-guarantees':['Build-Garantien','Ausgabegarantien','temporäre Artefakte','persistente Dateien','Dateilebensdauer','letzter Besitzer','atomarer Dateiaustausch','Dauerhaftigkeit','Veröffentlichungsphase','native Pfade','Inspektionslimits','vorbereitete Veröffentlichung verbraucht Besitzer'],
'troubleshooting':['Fehlersuche','Fehlerdiagnose','Installation fehlgeschlagen','native Erweiterung fehlt','NOTE.KEY_DUPLICATE','NOTE.FIELD_UNKNOWN','NOTE.MEDIA_USAGE_INVALID','MEDIA.DECLARED_MIME_MISMATCH','Vorlagenfehler','Ausgabepfad verschwunden','Richtlinie blockiert'],
'compatibility':['Kompatibilität','unterstützte Plattformen','geprüfte Umgebungen','Mindestversion','Betriebssystem','CPython Wheels','Node native Pakete','macOS Linux Windows','Alpine musl','Electron Bun','Anki-Import','Codec-Wiedergabe'],
'rust-api':['Rust-API','Methodenreferenz Rust','BuildOptions','InspectLimits','ApkgArtifact','BuildReport','BuildError','ComparisonReport','AddContext','prepare_publication','PathSnapshot'],
'node-quickstart':['Node.js-Schnellstart','JavaScript Einstieg','erster Stapel mit Node','npm install ankiforge','main.mjs','node main.mjs','CommonJS ESM','TypeScript installieren'],
'node-api':['Node-API','TypeScript-API','JavaScript Methoden','asynchroner Build','Promise','Ereignisschleife','Medienarbeiter','artifact.close','BuildOptions.updateFrom','tsconfig.json','snake_case Snapshot','camelCase Bericht'],
'python-quickstart':['Python-Schnellstart','erster Stapel mit Python','pip install ankiforge','virtuelle Umgebung','venv','main.py','Python-Skript ausführen','PyPI-Wheel'],
'python-api':['Python-API','Python Methodenreferenz','Project Note Media','BuildOptions','BuildOutput','temporärer Dateibesitz','Python Kontextmanager','GIL','Fork-Grenzen','prepare_publication'],
'python-diagnostics':['Python-Diagnostik','Python Fehlerbehandlung','BuildError','CompareError','PersistError','kind code causes','source_details','Risikokategorie freigeben','allows_publication','Inspektionsbudget','native Erweiterung fehlerhaft'],
'python-migration':['Python-API umstellen','alte API ersetzen','Clean-slate-API','anki-forge anki_forge ankiforge','Deck Registry Runtime entfernt','neues Datenbesitzmodell','PyO3 im selben Prozess'],
'genanki-migration':['von genanki umsteigen','genanki migrieren','Migration zu Anki Forge','numerische IDs und GUIDs','Identitätszuordnung','genanki Modell übertragen','Feldnamen und stabile Schlüssel','HTML ausdrücklich markieren','keine automatische ID-Konvertierung'],
'blog:your-first-anki-deck':['erster Anki-Stapel','Rust-Einstiegsartikel','hola hello','stabile Notiz-ID','ein vollständiges Programm','von einer Notiz zu mehreren Karten'],
'blog:stable-ids-and-deck-updates':['Notizidentität und Inhalt','stabile IDs','Formulierung ändern','ursprüngliches Paket aufbewahren','Basisversion für nächsten Build','Import prüfen','zwei Versionen vergleichen']
}
def concept(terms,**kw):return {'terms':terms,**kw}
concepts={
'__language':{'stopWords':'der die das den dem des ein eine einen einem einer eines und oder mit ohne von vom zu zum zur für fuer in im an am auf aus bei als ist sind sein wird werden ich du wir ihr sie es mein meine dein deine kann können koennen wie was wo welche welcher welches bitte möchte moechte will soll sollte muss müssen muessen nutzen verwenden benutze sdk über ueber zwischen auch noch diese dieser dieses'.split()},
'python':concept(['Python','CPython','pip'],sdk='python'),
'node':concept(['Node.js','Node','JavaScript','TypeScript','npm'],sdk='node'),
'rust':concept(['Rust','Cargo','Crate'],sdk='rust'),
'quickstart':concept(['Schnellstart','Schnelleinstieg','Einstieg','erste Schritte','beginnen','anfangen','Anfänger','Anfaenger','Tutorial','erster Stapel','ersten Stapel','erste Karte'],roles=['quickstart'],intent=True),
'reference':concept(['API','Referenz','API-Referenz','Methodenreferenz','Methoden','Funktionen','Typdeklarationen','Signaturen'],roles=['api'],intent=True),
'installation':concept(['Installation','installieren','einrichten','Einrichtung','Abhängigkeit','Abhängigkeiten','Abhaengigkeiten','integrieren'],roles=['installation','quickstart'],intent=True),
'development':concept(['aus dem Quellcode bauen','Quellcode kompilieren','Quellcode-Build','Repository entwickeln','lokales Addon','Wrapper neu bauen','maturin develop'],roles=['development'],intent=True),
'comparison':concept(['versus','vs','Alternative','Alternativen','Bibliotheksvergleich','Bibliotheken vergleichen','Bibliothek wählen','Bibliothek auswählen','Vor- und Nachteile'],roles=['comparison'],intent=True),
'migration':concept(['Migration','migrieren','umsteigen','Umstieg','wechseln','genanki ersetzen','alte API ersetzen'],roles=['migration'],intent=True),
'updates':concept(['aktualisieren','Aktualisierung','Aktualisierungen','Stapelaktualisierung','Notizaktualisierung','verteilten Stapel','vorhandene Notizen','Antwort ändern','erneut veröffentlichen','Lernplanung','Wiederholungsverlauf'],roles=['updates'],intent=True,requiresAny=['aktualisieren','Aktualisierung','Aktualisierungen','Stapelaktualisierung','Notizaktualisierung','veröffentlichen','Veröffentlichung','verteilt','verteilten','Verteilung','Basisversion','Version','vorhandene Notizen','Antwort ändern']),
'baseline':concept(['Aktualisierungsbasis','Basisversion','ursprünglich verteiltes APKG','ursprüngliches APKG','Original-APKG','urspruengliches APKG','erneuter Anki-Export','Anki-Reexport','Vorgängerpaket','Identitätsnachweise'],roles=['updates'],intent=True),
'identity':concept(['stabile ID','stabile IDs','stabiler Schlüssel','stabile Schlüssel','Notizschlüssel','Notiz-ID','GUID','Namensraum','namespace','fachlicher Schlüssel','Notizidentität','Notizidentitaet']),
'diagnostics':concept(['Fehlersuche','Fehlerdiagnose','Diagnose','Diagnostik','Fehler','fehlgeschlagen','scheitert','funktioniert nicht','fehlt','verschwunden','lädt nicht','laedt nicht','kann nicht laden'],roles=['diagnostics'],intent=True),
'media':concept(['Medien','Mediendatei','Mediendateien','Mediendaten','Medienimport','Stapelimport','Anhänge','Ressourcen'],roles=['media']),
'images':concept(['Bild','Bilder','Bilddatei','Bilddateien','Foto','Fotos','Illustration','Abbildung'],roles=['media']),
'audio':concept(['Audio','Audiodatei','Audiodateien','Ton','Töne','Toene','Klang','Aussprache','Stimme','mp3','wav'],roles=['media']),
'occlusion':concept(['Bildverdeckung','Image Occlusion','Bildmaske','Bildmasken','Rechteckmaske','Rechteckmasken','Maskenkoordinaten','Maskenschlüssel','Diagrammbeschriftung verdecken'],roles=['occlusion']),
'cloze':concept(['Cloze','Lückentext','Lueckentext','Lückentextkarte','Lückentextkarten','Lückennummer','Lückennummern','Lücken ausfüllen'],roles=['cards']),
 'templates':concept(['Vorlage','Vorlagen','Kartenvorlage','Kartenvorlagen','Vorlagenpaket','Vorlagenpakete','Vorlagen-Bundle','Template','Templates','CSS','Schriftart','Schriftarten'],roles=['templates']),
'note-types':concept(['Notiztyp','Notiztypen','eigener Notiztyp','eigene Notiztypen','eigenes Modell','benutzerdefinierte Felder','bedingte Kartenerzeugung'],roles=['note-types']),
'compatibility':concept(['Kompatibilität','Kompatibilitaet','unterstützte Plattformen','unterstuetzte Plattformen','geprüfte Umgebungen','Mindestversion','Betriebssystem','Alpine','musl','Electron','Bun'],roles=['compatibility']),
'csv':concept(['CSV','Tabelle','Tabellen','Tabellenkalkulation','Vokabeltabelle','Excel','Tabellenimport']),
'json':concept(['JSON','strukturierte Daten','Notizen stapelweise erstellen']),
'ai':concept(['KI','künstliche Intelligenz','kuenstliche Intelligenz','KI-Assistent','Programmierassistent','KI-Unterstützung','Kartenentwürfe','Aufzeichnungen in Karten']),
 'text':concept(['Klartext','Text','HTML','maskieren','Maskierung','HTML-Escaping']),
'fundamentals':concept(['Grundbegriff','Grundbegriffe','Konzept','Konzepte','was ist','was sind','Bedeutung','Zusammenhang','Unterschied zwischen','unterscheiden','gruppieren'],requiresAny=['Notiz','Notizen','Karte','Karten','Stapel','Projekt','Project','APKG','Namensraum','Feld','Felder','Vorlage','Vorlagen','Modell'],roles=['concepts'],intent=True,consumeContext=True),
'package-structure':concept(['Inhalt','enthält','enthalten','Bestandteile','Aufbau','Struktur','zusammengesetzt'],requiresAny=['APKG','Paket','Pakete','Export','exportiert'],roles=['concepts'],intent=True,consumeContext=True),
'learning':concept(['lernen','üben','ueben','wiederholen','Lernen','Fragen','Quiz'],weight=0.25),
'customization':concept(['anpassen','gestalten','Gestaltung','Design','definieren','eigene'],requiresAny=['Feld','Felder','Vorderseite','Rückseite','Rueckseite','Kartenvorlage','Kartenvorlagen','Modell','Notiztyp'],roles=['note-types'],intent=True),
'cardinality':concept(['mehrere','viele','wie viele','Anzahl','Kardinalität','Kartenanzahl','eine zu mehreren'],requiresAll=[['Notiz','Notizen'],['Karte','Karten']],roles=['concepts'],intent=True,consumeContext=True),
'occlusion-action':concept(['verdecken','verbergen','abdecken','verstecken','maskieren'],requiresAny=['Bild','Bilder','Foto','Fotos','Diagramm','Diagramme','Abbildung','Beschriftung'],roles=['occlusion'],consumeContext=True)
}
prompt='''Hilf mir, Anki Forge {version} in einer {language}-Anwendung zu verwenden.
Laufzeitanforderung: {runtime}

Verwende das veröffentlichte Paket `ankiforge` und ausschließlich die für
 diese Version dokumentierte öffentliche API. Verwende keine git-/path-Abhängigkeiten,
 internen Werkzeuge, nur im Quellcode verfügbaren APIs, unveröffentlichten oder
 erfundenen Methoden.

Meine Aufgabe:
{request}

Bitte:
1. Prüfe das bestehende Projekt, bevor du Dateien änderst.
2. Nenne die erforderliche Einrichtung und den genauen Installationsbefehl: {install}
3. Erstelle das kleinste vollständige, ausführbare Programm und nenne jede anzulegende oder zu bearbeitende Datei.
4. Erzeuge eine dauerhaft gespeicherte `.apkg`-Datei und verwende einen stabilen Namensraum und stabile Notizschlüssel.
5. Gib den Ausführungsbefehl, die erwartete Ausgabe und einen Prüfschritt an.
6. Wenn die Aufgabe Medien, eigene Vorlagen, Bildverdeckung oder Aktualisierungen
   benötigt, folge dem dokumentierten Ablauf, statt die API zu erraten. Bewahre für
   Aktualisierungen das ursprünglich verteilte APKG der Vorgängerversion als Basis
   auf; verwende keinen erneuten Export aus Anki.
7. Stelle nur die Fragen, die zur Klärung von Programmiersprache, Eingabeformat,
   Ausgabepfad oder Identitätsregel nötig sind.

Schnellstart-Referenz: {quickstart}'''
brief=json.loads((ROOT/'docs/plans/evidence/remaining-locales-2026-11-01/samples/de/language-brief.json').read_text());brief.pop('status',None)
p={'schemaVersion':1,'locale':'de','documents':{k:{'title':v[0],'description':v[1]} for k,v in meta.items()},'docGroups':['Hier beginnen','Anleitungen','Anleitungen nach Programmiersprache','Referenz','Hilfe'],'navigation':{'docs':'Anleitung','examples':'Beispiele','blog':'Blog'},'languageBrief':brief,'prompts':{'de':prompt},'requestPlaceholders':{'de':'<beschreibe deine Aufgabe>'},'promptContracts':{'de':{'required':['veröffentlichte Paket','öffentliche API','dauerhaft gespeicherte','stabilen Namensraum und stabile Notizschlüssel','Ausführungsbefehl','Prüfschritt','ursprünglich verteilte APKG'],'maxWords':500}},'searchTerms':terms,'searchConcepts':concepts}
assert len(meta)==29 and len(terms)==31
(BASE/'authoring/de-data-patch.json').write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n')
print('documents',len(meta),'searchTerms',len(terms),'searchConcepts',len(concepts))
