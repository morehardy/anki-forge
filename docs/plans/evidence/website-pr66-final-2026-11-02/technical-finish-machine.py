"""Seal this review's manually authored answers and scoped artifact observations.

This is not an automated semantic reviewer. It serializes the observations made
by /root/standards_review after reading current saved HTTP bodies and deltas.
It independently rechecks their bytes before writing the two reports.
"""
import hashlib
import json
import pathlib
import re

batch = pathlib.Path('docs/plans/evidence/website-pr66-final-2026-11-02')
digest = lambda data: hashlib.sha256(data).hexdigest()
load = lambda name: json.loads((batch / name).read_text())
def proof(name, description):
    path = batch / name
    return {'path': str(path), 'sha256': digest(path.read_bytes()), 'description': description}

# Fresh answers composed after reading this candidate's six localized entries.
# Their facts are not imported from the previous machine reports.
answers = {
 'en': {
  'python-install': "Use ordinary CPython 3.11+ with a matching platform wheel; the documented release checks cover 3.11–3.14. Create an environment with `python -m venv .venv`, activate `. .venv/bin/activate` (PowerShell: `.venv\\Scripts\\Activate.ps1`), and install `python -m pip install ankiforge==0.3.1`. Save the quickstart's complete program as `main.py`: it creates Project('spanish', default_deck='Spanish'), adds key 'es:hola' with Note.basic('hola', 'hello'), and builds with BuildOptions.to('spanish.apkg'). Run `python main.py`; expect a persistent APKG containing hola → hello in Spanish. Keep the namespace and key for subsequent versions. Export itself needs no Anki; importing and playback require checking the intended Anki client.",
  'node-api': "Start with Node.js 22.13+ and `npm install --include=optional ankiforge@0.3.1`; the matching native dependency is required. The complete main.mjs uses Project, Note and BuildOptions, awaits project.build(BuildOptions.to('spanish.apkg')), reads output.artifact.path, then awaits output.artifact.close(). Run `node main.mjs`. BuildOutput owns the artifact and exposes report/snapshot; BuildReport records observations, not success or file ownership. A persistent file survives close, whereas the final temporary owner deletes its file. Retain an owner while reading a temporary path. Snapshot JSON owns nothing. Prepared publish attempts consume their owner even on policy or I/O rejection, so retry by preparing again and inspect publication facts after errors.",
  'json-update': "Follow the local JSON recipe and put v1.json, v2.json, cell.png and tone.wav in recipe/. The caller parses namespace/deck/notes; preserve namespace biology-json and the three business IDs. Choose a complete SDK program: Python runs `python main.py recipe output/python`, Rust runs `cargo run -- recipe output/rust`, or compile main.mts with the documented NodeNext command and run `node dist/main.mjs recipe output/node`. Each release contains 3 notes, 3 cards and 2 media files. Keep the original v1.apkg as baseline and publish v2 separately after checking comparison.json and review.json; this edit changes only the first answer, records RISK.NOTE_CHANGED and allows publication. Publish the reviewed prepared candidate. Anki re-exports are unsuitable baselines; import does not delete omitted notes or guarantee local review history.",
  'compatibility-constraints': "The compatibility page dates its 0.3.1 package record to 2026-10-13. It records Rust 1.92+, Node 22.13 and the 22/24/26 lines, and ordinary CPython 3.11–3.14. Native packages cover macOS arm64/x64, Linux x64 GNU/glibc and Windows x64; package existence is not proof of every host. Browser/Electron/Bun, musl and ARM64 Linux/Windows are outside recorded Node checks; other Python implementations, free-threaded interpreters and subinterpreters need separate validation. Export needs no Anki, but import/rendering/codecs/settings and update effects require client checks. APKG replacement is atomic per file, not a transaction with a separate report or a crash-durability guarantee. A late error may follow publication; inspect stage and durability before retrying. Inspection limits are finite independent counters, not a process sandbox."
 },
 'zh-CN': {
  'python-install': "使用有匹配平台 wheel 的普通 CPython 3.11+；文档中的发布检查覆盖 3.11–3.14。运行 `python -m venv .venv`，用 `. .venv/bin/activate` 激活（PowerShell 用 `.venv\\Scripts\\Activate.ps1`），再执行 `python -m pip install ankiforge==0.3.1`。将快速开始的完整程序保存为 main.py：创建 namespace 为 spanish、默认牌组为 Spanish 的 Project，添加键 es:hola 与 Note.basic('hola', 'hello')，使用 BuildOptions.to('spanish.apkg') 构建。执行 `python main.py` 后应得到持久 APKG，其中 Spanish 牌组包含 hola → hello。后续版本保持 namespace 和键不变。导出无需安装 Anki；导入和播放仍需在目标客户端验证。",
  'node-api': "使用 Node.js 22.13+，运行 `npm install --include=optional ankiforge@0.3.1` 安装公开包及匹配的原生依赖。快速开始的 main.mjs 导入 Project、Note、BuildOptions，await project.build(BuildOptions.to('spanish.apkg'))，读取 output.artifact.path，最后 await output.artifact.close()；以 `node main.mjs` 运行。BuildOutput 拥有产物并提供 report/snapshot；BuildReport 只有观察结果，没有操作结果或文件所有权。持久文件关闭后保留，临时文件则在最后一个所有者释放时删除，读取期间必须保留所有者；JSON 快照不会延长寿命。预备发布每次 publish 尝试都会消耗所有者，策略或 I/O 拒绝也如此；重试需重新准备，并检查错误中的实际发布状态。",
  'json-update': "从 JSON 示例下载 v1.json、v2.json、cell.png、tone.wav 放入 recipe/。JSON 由调用方解析，保留 biology-json 命名空间和三条业务 ID。选择完整 SDK 程序：Python 执行 `python main.py recipe output/python`；Rust 执行 `cargo run -- recipe output/rust`；Node 按文档 NodeNext 命令编译 main.mts，再运行 `node dist/main.mjs recipe output/node`。每版应有 3 条笔记、3 张卡片、2 个媒体文件。保留原始分发 v1.apkg 作基线，将 v2 写到另一路径；检查 comparison.json、review.json，本例仅改变首条答案，出现 RISK.NOTE_CHANGED 且策略允许发布。发布的必须是已审阅预备候选包。Anki 重新导出的包不能作基线；省略笔记不会删除客户端笔记，客户端本地修改和复习状态需单独验证。",
  'compatibility-constraints': "兼容性页将公开包 0.3.1 的检查时间限定为 2026-10-13。记录覆盖 Rust 1.92+、Node 22.13 及 22/24/26 系列、普通 CPython 3.11–3.14。原生包面向 macOS arm64/x64、Linux x64 GNU/glibc、Windows x64，存在包不代表每台主机都测试过。浏览器、Electron、Bun、musl、ARM64 Linux/Windows 不在记录的 Node 范围；其他 Python 实现、自由线程与子解释器需另验。导出无需 Anki，但导入、渲染、编解码器、设置及更新效果依赖客户端。APKG 是单文件原子替换，不与独立报告构成事务，也不自动保证崩溃持久性。晚期错误可能发生在替换后，重试前检查 stage 和 durability。检查预算不是进程级资源沙箱。"
 },
 'es': {
  'python-install': "Usa CPython convencional 3.11 o posterior con un wheel compatible; las comprobaciones documentadas cubren 3.11–3.14. Crea el entorno con `python -m venv .venv`, actívalo con `. .venv/bin/activate` (PowerShell: `.venv\\Scripts\\Activate.ps1`) e instala `python -m pip install ankiforge==0.3.1`. Guarda el programa completo del inicio rápido como main.py: Project usa el espacio spanish y el mazo Spanish, añade es:hola mediante Note.basic('hola', 'hello') y genera BuildOptions.to('spanish.apkg'). Ejecuta `python main.py`. El APKG persistente debe contener hola → hello. Conserva el espacio de nombres y la clave en futuras versiones. Exportar no necesita Anki; verifica la importación y reproducción en tu cliente.",
  'node-api': "Necesitas Node.js 22.13 o posterior y `npm install --include=optional ankiforge@0.3.1`, incluida su dependencia nativa. El main.mjs completo importa Project, Note y BuildOptions, espera project.build(BuildOptions.to('spanish.apkg')), lee output.artifact.path y ejecuta await output.artifact.close(); ejecútalo con `node main.mjs`. BuildOutput conserva el archivo y expone report/snapshot. BuildReport solo contiene observaciones, sin resultado ni propiedad del archivo. Cerrar un artefacto persistente conserva su archivo; liberar el último propietario temporal lo elimina. Mantén un propietario mientras lees la ruta: el JSON no prolonga su vida. Cada intento de publish consume el candidato preparado, incluso ante rechazo por política o E/S; prepara otro para reintentar y comprueba los datos reales de publicación tras un error.",
  'json-update': "Descarga v1.json, v2.json, cell.png y tone.wav en recipe/. La aplicación analiza el JSON; conserva biology-json y los tres ID de negocio. Elige el programa completo de un SDK: `python main.py recipe output/python`, `cargo run -- recipe output/rust`, o compila main.mts con el comando NodeNext indicado y ejecuta `node dist/main.mjs recipe output/node`. Cada versión debe contener 3 notas, 3 tarjetas y 2 archivos multimedia. Guarda el APKG v1 original de distribución y publica v2 en otra ruta después de revisar comparison.json y review.json. El cambio de la primera respuesta registra RISK.NOTE_CHANGED, sin notas añadidas o eliminadas, y permite publicar el candidato preparado revisado. Una reexportación de Anki no sirve de base; importar no elimina notas omitidas ni garantiza el historial local.",
  'compatibility-constraints': "La página fecha la comprobación de los paquetes 0.3.1 en 2026-10-13. Registra Rust 1.92+, Node 22.13 y las ramas 22/24/26, y CPython convencional 3.11–3.14. Hay paquetes nativos para macOS arm64/x64, Linux x64 GNU/glibc y Windows x64; disponibilidad no significa prueba en todos los equipos. Navegadores, Electron, Bun, musl y Linux/Windows ARM64 quedan fuera de las pruebas de Node; otras implementaciones de Python, intérpretes sin GIL y subintérpretes requieren validación aparte. Exportar no necesita Anki, pero importación, códecs, representación y actualizaciones dependen del cliente. El reemplazo APKG es atómico por archivo, no una transacción junto al informe ni garantía de durabilidad tras un fallo. Antes de reintentar un error tardío, revisa stage y durability. Los límites de inspección no son un aislamiento de recursos del proceso."
 },
 'pt-BR': {
  'python-install': "Use CPython convencional 3.11 ou posterior com wheel compatível; a verificação registrada cobre 3.11–3.14. Crie o ambiente com `python -m venv .venv`, ative com `. .venv/bin/activate` (PowerShell: `.venv\\Scripts\\Activate.ps1`) e instale `python -m pip install ankiforge==0.3.1`. Salve o programa completo do início rápido como main.py: ele cria Project com namespace spanish e baralho Spanish, adiciona es:hola com Note.basic('hola', 'hello') e gera BuildOptions.to('spanish.apkg'). Rode `python main.py`. O arquivo persistente deve conter hola → hello. Preserve namespace e chave nas edições seguintes. Exportar não exige Anki; a importação e a reprodução precisam ser verificadas no cliente de destino.",
  'node-api': "Use Node.js 22.13 ou posterior e instale `npm install --include=optional ankiforge@0.3.1` com a dependência nativa correspondente. O main.mjs completo usa Project, Note e BuildOptions, aguarda project.build(BuildOptions.to('spanish.apkg')), lê output.artifact.path e fecha com await output.artifact.close(); execute `node main.mjs`. BuildOutput mantém o artefato e fornece report/snapshot; BuildReport tem observações, sem resultado da operação nem propriedade de arquivo. Um arquivo persistente permanece após close; o temporário desaparece ao liberar seu último proprietário. Mantenha o proprietário durante a leitura, pois JSON não prolonga essa vida. Toda tentativa de publish consome o candidato preparado, inclusive rejeições de política ou E/S. Prepare novamente para tentar outra vez e confira os fatos de publicação em caso de erro.",
  'json-update': "Coloque v1.json, v2.json, cell.png e tone.wav em recipe/. A aplicação interpreta o JSON; mantenha biology-json e os três IDs de negócio. Escolha um programa completo: `python main.py recipe output/python`, `cargo run -- recipe output/rust`, ou compile main.mts pelo comando NodeNext documentado e rode `node dist/main.mjs recipe output/node`. Cada versão deve ter 3 notas, 3 cartões e 2 arquivos de mídia. Preserve o APKG v1 originalmente distribuído como base e grave v2 em outro caminho. Confira comparison.json e review.json: a mudança só na primeira resposta gera RISK.NOTE_CHANGED, sem notas adicionadas ou removidas, e permite publicar o candidato preparado revisado. Reexportações do Anki não servem como base. Importar não exclui notas omitidas nem garante o histórico local; teste o cliente.",
  'compatibility-constraints': "A página registra os pacotes 0.3.1 conforme a verificação de 2026-10-13: Rust 1.92+, Node 22.13 e séries 22/24/26, CPython convencional 3.11–3.14. Os pacotes nativos cobrem macOS arm64/x64, Linux x64 GNU/glibc e Windows x64; isso não prova execução em toda máquina. Navegadores, Electron, Bun, musl e Linux/Windows ARM64 ficam fora das verificações de Node; outras implementações de Python, interpretadores sem GIL e subinterpretadores exigem teste separado. Exportar dispensa Anki, mas importação, codecs, renderização e atualizações dependem do cliente. A substituição atômica vale por APKG, não pelo conjunto com um relatório nem como garantia contra perda após falha. Um erro tardio pode ocorrer depois da publicação: examine stage e durability antes de repetir. Os limites de inspeção não isolam todos os recursos do processo."
 },
 'ja': {
  'python-install': "対応する wheel がある通常の CPython 3.11 以降を使います。記録されたリリース検査は 3.11–3.14 です。`python -m venv .venv` の後、`. .venv/bin/activate`（PowerShell は `.venv\\Scripts\\Activate.ps1`）で有効にし、`python -m pip install ankiforge==0.3.1` を実行します。クイックスタートの完全なコードを main.py に保存します。名前空間 spanish、デッキ Spanish の Project に es:hola と Note.basic('hola', 'hello') を追加し、BuildOptions.to('spanish.apkg') で生成します。`python main.py` の出力は hola → hello を含む永続 APKG です。次回も名前空間とキーを維持します。エクスポートに Anki は不要ですが、インポートや再生は対象クライアントで確認します。",
  'node-api': "Node.js 22.13 以降で `npm install --include=optional ankiforge@0.3.1` を実行し、対応するネイティブ依存も入れます。完全な main.mjs は Project、Note、BuildOptions を使い、project.build(BuildOptions.to('spanish.apkg')) を await して output.artifact.path を読み、最後に await output.artifact.close() します。`node main.mjs` で実行できます。BuildOutput は成果物を所有し report/snapshot を公開します。BuildReport は観測結果であり、成功・失敗やファイル所有権ではありません。永続ファイルは close 後も残り、一時ファイルは最後の所有者とともに削除されます。読み取り中は所有者を保持し、JSON に寿命延長を期待しないでください。準備済み候補の publish はポリシーや I/O 拒否も含め毎回消費されるため、再試行には再準備が必要です。エラー後は実際の公開状態を確認します。",
  'json-update': "v1.json、v2.json、cell.png、tone.wav を recipe/ に置きます。JSON は呼び出し側で解析し、biology-json と3つの業務 ID を維持します。完全な SDK プログラムを1つ選び、Python なら `python main.py recipe output/python`、Rust なら `cargo run -- recipe output/rust` を実行します。Node は文書の NodeNext コマンドで main.mts をコンパイルし、`node dist/main.mjs recipe output/node` を実行します。各版はノート3件、カード3枚、メディア2個です。配布時のオリジナル v1.apkg を保存して基準にし、v2 は別のパスに出します。comparison.json と review.json で最初の回答だけの変更、RISK.NOTE_CHANGED、追加・削除なし、公開許可を確認して同じ準備済み候補を公開します。Anki 再エクスポートは基準にできず、ノートの省略による削除や各学習者の履歴保持も保証されません。",
  'compatibility-constraints': "互換性ページの 0.3.1 は2026-10-13の確認記録です。Rust 1.92 以降、Node 22.13 と22/24/26、通常の CPython 3.11–3.14 を記録しています。ネイティブパッケージは macOS arm64/x64、Linux x64 GNU/glibc、Windows x64 向けですが、全ホストでの成功を意味しません。ブラウザー、Electron、Bun、musl、ARM64 Linux/Windows は Node の記録範囲外です。他の Python 実装、free-threaded、サブインタープリターも別途検証します。出力には Anki 不要でも、インポート、描画、コーデック、設定、更新効果は対象クライアントに依存します。APKG のアトミック置換はファイル単位で、別レポートとの一括トランザクションやクラッシュ後の耐久性を保証しません。公開後の遅いエラーもあるため、再試行前に stage と durability を確認します。検査上限はプロセス全体のサンドボックスではありません。"
 },
 'de': {
  'python-install': "Verwende reguläres CPython ab 3.11 mit passendem Plattform-Wheel; die dokumentierten Release-Prüfungen decken 3.11–3.14 ab. Erzeuge mit `python -m venv .venv` eine Umgebung, aktiviere sie über `. .venv/bin/activate` (PowerShell: `.venv\\Scripts\\Activate.ps1`) und installiere `python -m pip install ankiforge==0.3.1`. Speichere das vollständige Schnellstartprogramm als main.py: Project verwendet den Namensraum spanish und Stapel Spanish, fügt es:hola mit Note.basic('hola', 'hello') hinzu und baut mit BuildOptions.to('spanish.apkg'). Starte `python main.py`. Das persistente APKG enthält hola → hello. Behalte Namensraum und Schlüssel bei späteren Änderungen bei. Der Export benötigt kein Anki; Import und Wiedergabe sind im Zielclient zu prüfen.",
  'node-api': "Benötigt werden Node.js ab 22.13 und `npm install --include=optional ankiforge@0.3.1` samt passender nativer Abhängigkeit. Das vollständige main.mjs importiert Project, Note und BuildOptions, wartet auf project.build(BuildOptions.to('spanish.apkg')), liest output.artifact.path und ruft await output.artifact.close() auf. Ausführen mit `node main.mjs`. BuildOutput besitzt das Artefakt und bietet report/snapshot; BuildReport enthält Beobachtungen, keinen Operationsausgang oder Dateibesitz. Persistente Dateien bleiben nach close erhalten, temporäre verschwinden mit dem letzten Besitzer. Halte beim Lesen einen Besitzer; JSON verlängert die Lebensdauer nicht. Jeder publish-Versuch verbraucht den vorbereiteten Kandidaten, auch bei Richtlinien- oder E/A-Ablehnung. Bereite ihn für einen neuen Versuch erneut vor und prüfe bei Fehlern die tatsächlichen Veröffentlichungsfakten.",
  'json-update': "Lege v1.json, v2.json, cell.png und tone.wav unter recipe/ ab. Die Anwendung liest das JSON; biology-json und die drei fachlichen IDs bleiben stabil. Wähle ein vollständiges SDK-Programm: `python main.py recipe output/python`, `cargo run -- recipe output/rust` oder kompiliere main.mts mit dem dokumentierten NodeNext-Befehl und starte `node dist/main.mjs recipe output/node`. Jede Version enthält 3 Notizen, 3 Karten und 2 Mediendateien. Bewahre das ursprünglich verteilte v1.apkg als Basis auf und schreibe v2 an einen anderen Pfad. comparison.json und review.json sollen die Änderung nur der ersten Antwort, RISK.NOTE_CHANGED, keine hinzugefügten/entfernten Notizen und die Veröffentlichungsfreigabe zeigen. Veröffentlicht wird derselbe geprüfte Kandidat. Ein Anki-Reexport ist keine geeignete Basis; der Import löscht keine ausgelassenen Notizen und garantiert keinen individuellen Wiederholungsverlauf.",
  'compatibility-constraints': "Die Kompatibilitätsseite datiert den Paketstand 0.3.1 auf die Prüfung vom 2026-10-13. Dokumentiert sind Rust ab 1.92, Node 22.13 sowie 22/24/26 und reguläres CPython 3.11–3.14. Native Pakete decken macOS arm64/x64, Linux x64 GNU/glibc und Windows x64 ab, nicht nachweislich jeden Host. Browser, Electron, Bun, musl und ARM64 Linux/Windows liegen außerhalb der Node-Prüfabdeckung; andere Python-Implementierungen, Interpreter ohne GIL und Subinterpreter benötigen eigene Tests. Exportieren benötigt kein Anki, doch Import, Darstellung, Codecs, Einstellungen und Aktualisierungen hängen vom Client ab. Der atomare APKG-Austausch gilt pro Datei, nicht gemeinsam mit einem Bericht und nicht automatisch als Absturz-Dauerhaftigkeit. Ein später Fehler kann nach der Veröffentlichung auftreten; prüfe stage und durability vor dem Wiederholen. Inspektionslimits sind keine Ressourcen-Sandbox für den Prozess."
 }
}

source = load('technical-delta-observations.json')
source_observations = source['observations']
llms_observations = {
 '/llms.txt': 'Read the rewritten introductory constraint sentence in the actual llms response: it still distinguishes public packages/source builds, requires the original preceding APKG, and retains Anki import/local-edit caveats. The version/runtime commands and four task destinations are unchanged.',
 '/de/llms.txt': 'Read Aufgabenanleitungen → Anleitungen nach Aufgabe in the response. This changes the task-group label only; localized task destinations, runtime commands and caveats are unchanged.',
 '/ja/llms.txt': 'Read the updated Rust quickstart and Python API summaries. They describe saving an APKG for later use and pre-generating/saving a candidate; the linked current pages retain persistence and same-candidate publication conditions. Task URLs and commands are unchanged.',
 '/pt-br/llms.txt': 'Read the changed introduction/template/troubleshooting summaries and stable-ID article title. Card templates remain distinct from note types, programming language is explicit, and the article still requires stable IDs. All linked task destinations and package/runtime commands are unchanged.',
 '/zh-cn/llms.txt': 'Read the new stable-ID article link title: changing note content while retaining ID matches the current article. The target, historical date, original-distribution baseline constraint and four task links are unchanged.',
}
metadata_only = {
 '/docs/installation/index.md': 'The captured H1 now explicitly says Rust application. Cargo commands, runtime prerequisites and the complete body are unchanged.',
 '/de/docs/installation/index.md': 'The captured H1 now explicitly says Rust-Anwendung. Cargo commands, minimum runtime and native toolchain requirements are unchanged.',
 '/pt-br/docs/installation/index.md': 'The captured H1 now explicitly identifies aplicação Rust. The installation commands, version and runtime requirements are unchanged.',
 '/zh-cn/docs/installation/index.md': 'The captured H1 now says 集成到 Rust 应用. Rust installation commands, version and toolchain limits are unchanged.',
 '/ja/docs/quickstart/index.md': 'The captured description explains persistence as saving an APKG for later use. The complete Rust program and persistent-artifact explanation remain unchanged.',
 '/pt-br/docs/custom-notetypes/index.md': 'Read the captured title/description changes: the page is about tipos de nota personalizados, containing templates de cartão. This preserves the note-type versus card-template distinction; executable content is unchanged.',
 '/pt-br/docs/templates/index.md': 'Read the captured title/description replacing ambiguous modelos with templates de cartão. The same bundle API, note-type ownership and explicit media dependency conditions remain in the body.',
}

def source_key(route):
    segments = route.strip('/').split('/')
    language = ''
    if segments[0] in ['de', 'es', 'ja', 'pt-br', 'zh-cn']:
        language = segments.pop(0)
    if segments[0] == 'blog':
        return f"website/src/content/blog/{language}/{segments[1]}.mdx"
    name = segments[1] if len(segments) > 2 else 'index'
    if not language and name == 'python-api':
        return 'docs/python/api.md'
    if not language and name == 'templates':
        return 'docs/template-bundles.md'
    return 'docs/' + (language+'/' if language else '') + name + '.md'

comparison = load('technical-machine-dual-configuration-comparison.json')
assert len(comparison['records']) == 192
prefix_audit = load('technical-machine-prefix-input-audit.json')
changed_observations = []
for record in prefix_audit['records']:
    if not record['changed']:
        continue
    route = record['route'].removeprefix('/anki-forge')
    key = source_key(route) if not route.endswith('llms.txt') else None
    if route in llms_observations:
        observation = llms_observations[route]
    elif route in metadata_only:
        observation = metadata_only[route]
    else:
        assert key in source_observations, (route, key)
        observation = 'Read the actual captured Markdown delta and surrounding task context. The generated change preserves the following independently checked source meaning: ' + source_observations[key]
    changed_observations.append({
        'routeWithoutDeploymentBase': route,
        'prefixRoute': record['route'], 'rootRoute': route,
        'prefixSha256': record['sha256'],
        'rootSha256': next(x['rootSha256'] for x in comparison['records'] if x['rootRoute'] == route),
        'status': 'reviewed', 'observation': observation,
        'scope': 'All added/removed text and immediate unchanged context in the actual machine resource were manually read in this run. Unchanged long-form remainder is inherited only through the explicit old/current byte audit; it is not claimed as newly reread in full.',
        'sourceObservationKey': key,
        'actualCapturedDelta': record['delta'],
        'openIssues': [],
    })
assert len(changed_observations) == 65
scope = {
    'schemaVersion': 1, 'reviewerRun': '/root/standards_review/pr66-machine-semantic-reading-2026-11-02',
    'method': 'Independent AI reading of coordinator-captured current HTTP response bodies. Read six locale llms entries, all six complete Python quickstarts, all six Node quickstart programs and Node API artifact/report/lifetime/consume passages, all six JSON recipe prose/result/limitations and unchanged shell commands, all six complete compatibility and build-guarantee prose, and update/media baseline/client/budget passages. All 65 changed resource deltas were separately read, including metadata, headings and articles beyond the four tasks. Fresh localized answers were authored from this reading. Exact executable JSON-recipe blocks were compared to the prior reviewed actual archive, not rerun in this machine review. SDK execution belongs to the separate usage reviewer.',
    'rootScope': 'Separately read actual root-capture llms introductions and all 24 primary task title/canonical/language headers. Full semantic reading of current prefix responses is shared with root only after proving all 192 paired bytes differ solely by the one complete configured site+base substitution. This is not a claim of a second full manual reread.',
    'captureProvenance': 'HTTP GETs were performed by the coordinator capture-machine.mjs on the two archive directories, at the localhost origins recorded in each observation. This reviewer read those saved actual response bytes and independently matched them to archive and manifest. No deployed-site request or hosting/MIME acceptance is claimed.',
    'bindingScope': 'Exactly 192 Markdown/llms resources per configuration. There are 96 saved HTTP responses per configuration, including all 65 changed resources. The remaining resources retain explicit verified old-archive review lineage. No unread showcase metadata or application HTML is bound by this report.',
    'previousReviewRule': 'Prior reports are retained only as traceable evidence for unchanged bytes/sections. Current answers, changed-resource observations, manifest/configuration bindings and status are newly reviewed. Previous machine reports had no open issues; no issue was silently removed.',
    'sourceReview': proof('technical-source-review.json', 'Frozen independent source review underlying the precise source comparisons cited in the changed machine resources; not rewritten by this artifact review.'),
    'sourceObservations': proof('technical-delta-observations.json', 'Prior source-level API and prose observations, used to compare the newly read generated changes to their actual reviewed meaning.'),
    'configurationComparison': proof('technical-machine-dual-configuration-comparison.json', 'Actual 192 resource pairs with one exact configuration URL substitution; no prose normalization.'),
    'answers': answers,
    'changedResources': changed_observations,
    'openIssues': [],
}
scope_path = batch / 'technical-machine-semantic-reading.json'
scope_path.write_text(json.dumps(scope, ensure_ascii=False, indent=2)+'\n')

task_observation = {
    'python-install': 'The localized llms entry leads to a complete virtualenv/pinned-pip/main.py path, ordinary CPython and native-wheel prerequisites, persistent Spanish/hola→hello result, stable identity and bounded client compatibility. The answer uses the current captured localized instructions.',
    'node-api': 'Read current localized complete quickstart and API passages for awaited build, persistent/temporary ownership, BuildOutput versus BuildReport, report snapshots and consumed prepared publication. The fresh answer retains native installation prerequisites and lifetime/error constraints.',
    'json-update': 'Read the current recipe prose, shared-input links, SDK run commands, counts, review report/policy outcome, original baseline and client limits. All ten fenced blocks per locale match the prior actual reviewed archive; no new SDK execution is claimed. Read supporting updates/media constraints as documented.',
    'compatibility-constraints': 'Read the current complete localized compatibility and build-guarantee prose. The answer preserves the dated package record, verified-versus-available distinction, excluded platforms, client conditions, per-file atomicity, late publication errors and non-sandbox inspection limits.',
}

for environment in ['prefix', 'root']:
    observation = load(f'machine-{environment}/observations.json')
    audit = load(f'technical-machine-{environment}-input-audit.json')
    manifest_name = f'{environment}-site-manifest.json'
    manifest = load(manifest_name)
    archive = pathlib.Path(f'target/website-pr66-final/{environment}-dist')
    old_archive = pathlib.Path(f'/private/tmp/anki-forge-multilingual-fixes-20261101/target/multilingual-review-fixes/{environment}-dist')
    assert (batch / manifest_name).read_bytes() == (archive / 'generated/site-manifest.json').read_bytes()
    assert digest((batch / manifest_name).read_bytes()) == observation['manifest']['sha256'] == audit['manifest']['sha256']
    assert digest((batch / f'machine-{environment}/observations.json').read_bytes()) == audit['capture']['sha256']
    assert manifest['inputSha256'] == 'fd6716207f6f5557895553e95f710a27cc66cee6d4bbd1cacf0ddc56b7aceebf'
    previous = audit['previousReview']
    assert digest(pathlib.Path(previous['path']).read_bytes()) == previous['sha256']
    assert not json.loads(pathlib.Path(previous['path']).read_text()).get('openIssues', [])
    by_route = {x['route']: x for x in audit['records']}
    responses = {x['route']: x for x in observation['responses']}
    bindings = []
    for item in observation['inventory']:
        route = item['route']
        relative = route.removeprefix('/anki-forge') if environment == 'prefix' else route
        current = (archive / relative.lstrip('/')).read_bytes()
        old = (old_archive / relative.lstrip('/')).read_bytes()
        assert digest(current) == item['sha256'] == by_route[route]['sha256']
        assert digest(old) == item['previousSha256'] == by_route[route]['previousSha256']
        if route in responses:
            response = responses[route]
            assert response['status'] == 200 and response['sha256'] == digest(current)
            assert pathlib.Path(response['bodyPath']).read_bytes() == current
        if item['changed']:
            assert route in responses
        if '/docs/json-to-anki/index.md' in route:
            assert re.findall(rb'```[^\n]*\n.*?```', old, re.S) == re.findall(rb'```[^\n]*\n.*?```', current, re.S)
        bindings.append({'route': route, 'sha256': digest(current)})
    assert len(bindings) == 192 and len(responses) == 96
    assert sum(x['changed'] for x in observation['inventory']) == 65
    assert set(answers) == set(manifest['releaseLocales'])
    entries = {}
    for locale in manifest['releaseLocales']:
        results = []
        for task, fresh_answer in answers[locale].items():
            reads = observation['readings'][locale][task]
            # Python API is captured for delta review; the installation answer
            # is directly based on the complete quickstart and compatibility.
            if task == 'python-install':
                reads = [x for x in reads if '/docs/python-api/' not in x['route']]
            for item in reads:
                assert item['route'] in responses
            primary = reads[1]
            evidence = [
                {'route': primary['route'], 'sha256': by_route[primary['route']]['sha256'], 'observation': task_observation[task]},
                proof('technical-machine-semantic-reading.json', 'Fresh localized answer, exact manual reading scope and all 65 separately reviewed changed-resource observations; explicit unchanged-section and root/prefix lineage.'),
                proof(f'technical-machine-{environment}-input-audit.json', 'Independent old/current archive hash comparison for all 192 machine resources and exact match of every saved HTTP body; retains the real previous manifest/report lineage.'),
                proof(f'machine-{environment}/observations.json', 'Coordinator actual localhost HTTP GET records and saved-body paths, independently verified by this reviewer; not claimed as deployed hosting validation.'),
            ]
            if environment == 'root':
                evidence.append(proof('technical-machine-dual-configuration-comparison.json', 'Current root and prefix body equality after only complete configured URL replacement; root entry/header checks separately performed.'))
            results.append({'id': task, 'status': 'passed', 'readUrls': [x['url'] for x in reads], 'answer': fresh_answer + '\n\nSources: ' + ', '.join(x['url'] for x in reads), 'evidence': evidence, 'openIssues': []})
        entries[locale] = {'aiMachineReadReview': results, 'openIssues': []}
    report = {
        'schemaVersion': 1, 'kind': 'machine-read', 'status': 'passed', 'mode': 'release-candidate',
        'reviewerType': 'ai', 'reviewerRun': f'/root/standards_review/pr66-current-machine-{environment}-2026-11-02',
        'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'], 'releaseLocales': manifest['releaseLocales'],
        'readOrigin': observation['readOrigin'],
        'originManifest': {'path': str(batch / manifest_name), 'sha256': digest((batch / manifest_name).read_bytes()), 'commit': manifest['commit'], 'description': 'Actual new PR #66 archive manifest whose saved HTTP responses and current machine resources were independently read and verified in this run.'},
        'method': scope['method'], 'captureProvenance': scope['captureProvenance'],
        'scope': scope['bindingScope'], 'rootScope': scope['rootScope'] if environment == 'root' else 'Direct current prefix captured-body semantic reading, with unchanged sections explicitly inherited as described in semantic-reading evidence.',
        'previousReview': previous, 'resourceBindings': bindings, 'entries': entries, 'openIssues': [],
        'limitations': ['No deployment has occurred or been validated by this review.', 'This is a machine-readable documentation review; SDK execution and Anki client behavior require their distinct usage/client evidence.', 'HTTP retrieval was performed by the coordinator; independent reviewer read saved exact HTTP response bodies, not an independently fetched second set.'],
    }
    target = batch / f'machine-read-{environment}.json'
    target.write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({'path': str(target), 'sha256': digest(target.read_bytes()), 'tasks': sum(len(x['aiMachineReadReview']) for x in entries.values()), 'bindings': len(bindings), 'changedResourcesRead': len(changed_observations)}))
