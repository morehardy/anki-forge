import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

// Evidence assembly after /root/standards_review independently read the captured
// task passages. This script verifies bindings; it does not perform AI review.
const directory = 'docs/plans/evidence/multilingual-review-fixes-2026-11-01';
const sha = value => createHash('sha256').update(value).digest('hex');
const parse = async filename => JSON.parse(await readFile(filename));
const save = async (filename, value) => writeFile(filename, JSON.stringify(value, null, 2) + '\n');
const expectedInput = 'a2618b25656a1b4adfafb3add60ad85b5699b32a3217e01c8e5b125c53a8f833';
const scope = 'Only the isolated 6d4625639fc1477222b41880d29724c162baa3be plus seven multilingual review-fix implementation inputs and this batch of source/post-build review records. Concurrent edits in the original shared workspace are excluded. This is an undeployed release candidate.';
const answers = {
  en: {
    'python-install': 'The documented 0.3.1 quickstart uses ordinary CPython 3.11+ with a matching native wheel; recorded checks cover 3.11–3.14. Create a virtual environment with python -m venv .venv, activate it using . .venv/bin/activate (Windows PowerShell: .venv\\Scripts\\Activate.ps1), then run python -m pip install ankiforge==0.3.1. Save the complete example as main.py and run python main.py. Project namespace spanish and note key es:hola produce persistent spanish.apkg with the Spanish deck and hola → hello Basic card. Keep those identifiers stable for updates. Exporting does not require Anki; importing and inspecting the card is a separate client step.',
    'node-api': 'Use Node 22.13+ and npm install --include=optional ankiforge@0.3.1 so the matching native dependency is installed. Create Project(namespace), add a note under a stable key, and await project.build(BuildOptions.to(path)). BuildOutput owns the artifact and exposes the report/snapshot; BuildReport contains observations, not ownership or the final operation outcome. Keep an owner alive while reading a temporary path; closing the last temporary owner deletes the file, whereas a persistent output remains. Node build/compare capture project state at invocation. Every prepared-publication attempt consumes its owner, including policy or I/O rejection; retry requires preparing again. TypeScript uses the documented NodeNext configuration before running the emitted .mjs.',
    'json-update': 'Download v1.json, v2.json, cell.png and tone.wav into recipe/, then use one complete Rust, Python or Node example to parse JSON and build both releases. The namespace biology-json and the three business note IDs remain stable; each release has three notes/cards and two media files, and v2 changes the first answer. Keep the originally distributed v1.apkg as the immutable comparison baseline and publish to a different v2 path; an Anki re-export is unsuitable. Inspect comparison.json/review.json: the expected finding is RISK.NOTE_CHANGED, with no added or removed notes, and policy approval before publishing that same prepared candidate. Missing notes are not deletion synchronization; client update and review-history behavior still need client-specific validation.',
    'compatibility-constraints': 'The compatibility page records package 0.3.1 as checked on 2026-10-13; this is a recorded verification snapshot. It lists Rust 1.92, Node 22.13+ with checks on 22/24/26, and ordinary CPython 3.11–3.14. Native packages target macOS arm64/x64, Linux x64 GNU/glibc and Windows x64; package availability alone does not prove every host was tested. Other Python implementations, free-threaded/subinterpreters, and Node browsers/Electron/Bun/musl/ARM64 Linux or Windows need separate validation. Anki import, rendering and playback depend on client/settings/codecs. Preserve original APKG baselines. Atomic replacement applies per file, not to a separate JSON report; late failures can occur after publication, so inspect publication stage and durability before retrying.'
  },
  'zh-CN': {
    'python-install': '文档的 0.3.1 示例要求普通 CPython 3.11+ 和匹配平台的原生 wheel，记录的检查覆盖 3.11–3.14。先执行 python -m venv .venv，以 . .venv/bin/activate 激活；Windows PowerShell 使用 .venv\\Scripts\\Activate.ps1。再执行 python -m pip install ankiforge==0.3.1，把完整示例存成 main.py 并运行 python main.py。namespace 为 spanish、业务键为 es:hola，生成持久化的 spanish.apkg，其中 Spanish 牌组包含 hola → hello 的 Basic 卡片。更新时保持这两个标识稳定。导出不依赖安装 Anki，导入检查属于另一个客户端步骤。',
    'node-api': '使用 Node 22.13+，通过 npm install --include=optional ankiforge@0.3.1 安装匹配平台的原生依赖。创建 Project(namespace)，以稳定键添加笔记，再 await project.build(BuildOptions.to(path))。BuildOutput 持有产物并提供报告与快照；BuildReport 只记录观察结果，既不持有文件，也不能单靠诊断严重性推断操作成败。读取临时路径时必须保留所有者，关闭最后一个临时所有者会删除文件，持久化文件则保留。Node 的 build/compare 在调用时捕获项目快照。准备好的发布每次尝试都会消费所有者，包括策略或 I/O 拒绝，重试需要重新准备；TypeScript 按文档的 NodeNext 配置编译后运行 .mjs。',
    'json-update': '将 v1.json、v2.json、cell.png、tone.wav 下载到 recipe/，任选文档给出的完整 Rust、Python 或 Node 程序，由调用者解析 JSON 并构建两版。biology-json namespace 和三个业务笔记 ID 保持稳定；每版有三条笔记、三张卡片、两个媒体，v2 修改第一条答案。保留最初分发的 v1.apkg 为不可变基线，v2 使用不同输出路径，不能改用 Anki 重新导出的包。检查 comparison.json/review.json，预期出现 RISK.NOTE_CHANGED，没有新增或删除笔记，并在策略允许后发布同一个已准备候选。省略笔记不会同步删除学习者的数据，也不能据此保证所有客户端的复习历史行为。',
    'compatibility-constraints': '兼容性页记录的是 2026-10-13 核验的 0.3.1：Rust 1.92、Node 22.13+（记录检查 22/24/26）、普通 CPython 3.11–3.14。原生包覆盖 macOS arm64/x64、Linux x64 GNU/glibc、Windows x64；有包不等于每个宿主都实际跑过。其他 Python 实现、free-threaded 与子解释器，以及浏览器、Electron、Bun、musl、ARM64 Linux/Windows 的 Node 环境需要额外验证。Anki 导入、渲染和媒体播放依赖客户端版本、设置及编解码器。更新应保留原始 APKG；原子替换仅按文件成立，不与外部 JSON 报告构成事务。晚期错误可能发生在发布之后，重试前须查看 publication stage 和 durability。'
  },
  es: {
    'python-install': 'El inicio rápido documenta 0.3.1 para CPython normal 3.11+ con una wheel nativa compatible; las comprobaciones registradas cubren 3.11–3.14. Ejecute python -m venv .venv, active con . .venv/bin/activate (PowerShell: .venv\\Scripts\\Activate.ps1) e instale con python -m pip install ankiforge==0.3.1. Guarde el ejemplo completo como main.py y ejecute python main.py. El namespace spanish y la clave es:hola generan spanish.apkg persistente, con la tarjeta Basic hola → hello en Spanish. Mantenga estables ambos identificadores al actualizar. Exportar no requiere Anki; la importación debe comprobarse después en el cliente.',
    'node-api': 'Use Node 22.13+ e instale npm install --include=optional ankiforge@0.3.1 para incluir la dependencia nativa correspondiente. Cree Project(namespace), añada notas con claves estables y espere project.build(BuildOptions.to(path)). BuildOutput conserva el artefacto y ofrece informe/snapshot; BuildReport contiene observaciones, no mantiene el archivo ni determina por sí solo el resultado. Conserve un propietario mientras use una ruta temporal: cerrar el último elimina el archivo, pero el persistente permanece. build/compare capturan el proyecto al invocarse. Cada intento de publicación preparada consume el propietario, incluso si falla por política o E/S; prepare de nuevo para reintentar. Compile TypeScript con la configuración NodeNext documentada antes de ejecutar el .mjs.',
    'json-update': 'Descargue v1.json, v2.json, cell.png y tone.wav en recipe/ y use uno de los programas completos en Rust, Python o Node. El programa analiza JSON y conserva el namespace biology-json y los tres identificadores de negocio. Cada versión contiene tres notas/tarjetas y dos medios; v2 cambia la primera respuesta. Conserve el v1.apkg distribuido originalmente como base inmutable y escriba v2 en otra ruta: una reexportación desde Anki no sirve como sustituto. Revise comparison.json/review.json: se espera RISK.NOTE_CHANGED, sin notas añadidas o eliminadas, y aprobación de la política antes de publicar ese mismo candidato preparado. Omitir notas no elimina datos del alumno; el comportamiento de importación e historial depende del cliente.',
    'compatibility-constraints': 'La tabla registra 0.3.1 comprobado el 2026-10-13: Rust 1.92, Node 22.13+ con comprobaciones en 22/24/26 y CPython normal 3.11–3.14. Hay paquetes nativos para macOS arm64/x64, Linux x64 GNU/glibc y Windows x64; disponer de paquete no demuestra pruebas en todos los hosts. Otras implementaciones de Python, free-threaded/subintérpretes y Node en navegador/Electron/Bun/musl/ARM64 Linux o Windows necesitan validación aparte. La importación, el renderizado y la reproducción dependen de Anki, sus ajustes y los códecs. Conserve la base APKG original. La sustitución atómica es por archivo y no incluye un informe JSON separado; un error tardío puede ocurrir después de publicar, por lo que hay que revisar stage y durability antes de reintentar.'
  },
  'pt-BR': {
    'python-install': 'O início rápido documenta 0.3.1 com CPython comum 3.11+ e uma wheel nativa compatível; as verificações registradas cobrem 3.11–3.14. Execute python -m venv .venv, ative com . .venv/bin/activate (no PowerShell: .venv\\Scripts\\Activate.ps1) e instale com python -m pip install ankiforge==0.3.1. Salve o exemplo completo como main.py e execute python main.py. O namespace spanish e a chave es:hola produzem spanish.apkg persistente, com o cartão Basic hola → hello no baralho Spanish. Preserve esses identificadores nas atualizações. Exportar dispensa Anki; a importação deve ser conferida separadamente no cliente.',
    'node-api': 'Use Node 22.13+ e npm install --include=optional ankiforge@0.3.1 para instalar a dependência nativa correta. Crie Project(namespace), adicione notas com chaves estáveis e aguarde project.build(BuildOptions.to(path)). BuildOutput mantém o artefato e fornece relatório/snapshot; BuildReport reúne observações, sem manter o arquivo nem definir sozinho o resultado da operação. Mantenha um proprietário enquanto usa um caminho temporário: fechar o último exclui o arquivo, enquanto o persistente permanece. build/compare capturam o projeto no momento da chamada. Toda tentativa de publicação preparada consome o proprietário, inclusive rejeições por política ou E/S; uma nova tentativa exige nova preparação. Compile TypeScript com o NodeNext documentado e execute o .mjs gerado.',
    'json-update': 'Baixe v1.json, v2.json, cell.png e tone.wav para recipe/ e use um dos programas completos em Rust, Python ou Node. O programa interpreta o JSON e mantém o namespace biology-json e os três IDs de negócio. Cada versão tem três notas/cartões e duas mídias; v2 muda a primeira resposta. Guarde o v1.apkg originalmente distribuído como base imutável e publique v2 em outro caminho; um arquivo reexportado pelo Anki não é uma base equivalente. Confira comparison.json/review.json: espera-se RISK.NOTE_CHANGED, sem notas adicionadas ou removidas, e autorização da política antes de publicar o mesmo candidato preparado. Omitir notas não sincroniza exclusões; importação e histórico de revisão ainda exigem validação no cliente.',
    'compatibility-constraints': 'A página registra 0.3.1 verificado em 2026-10-13: Rust 1.92, Node 22.13+ com verificações em 22/24/26 e CPython comum 3.11–3.14. Os pacotes nativos cobrem macOS arm64/x64, Linux x64 GNU/glibc e Windows x64; disponibilidade não comprova execução em todos os hosts. Outras implementações de Python, free-threaded/subinterpretadores e Node em navegador/Electron/Bun/musl/ARM64 Linux ou Windows precisam de validação própria. Importação, renderização e reprodução dependem do Anki, das configurações e dos codecs. Preserve a base APKG original. A substituição atômica vale por arquivo, sem incluir um relatório JSON separado; falhas tardias podem ocorrer depois da publicação. Confira stage e durability antes de tentar novamente.'
  },
  ja: {
    'python-install': 'クイックスタートの対象は 0.3.1、通常の CPython 3.11 以降と対応するネイティブ wheel です。記録された検証は 3.11〜3.14 を対象にしています。python -m venv .venv で環境を作り、. .venv/bin/activate（PowerShell では .venv\\Scripts\\Activate.ps1）で有効化して、python -m pip install ankiforge==0.3.1 を実行します。完全な例を main.py に保存し python main.py で実行すると、namespace spanish とキー es:hola により、Spanish デッキの hola → hello Basic カードを含む永続的な spanish.apkg が生成されます。更新時も識別子を維持してください。エクスポート自体に Anki は不要で、インポート確認は別途必要です。',
    'node-api': 'Node 22.13 以降を使い、npm install --include=optional ankiforge@0.3.1 で対応するネイティブ依存関係も導入します。Project(namespace) に安定したキーでノートを追加し、project.build(BuildOptions.to(path)) を await します。BuildOutput は成果物を所有しレポートとスナップショットを提供しますが、BuildReport は観測情報だけで、ファイルを保持せず操作結果そのものでもありません。一時パスの使用中は所有者を保持し、最後の所有者を閉じると一時ファイルは削除されます。永続ファイルは残ります。build/compare は呼び出し時の状態を固定します。準備済み公開はポリシー拒否や I/O 失敗でも試行ごとに所有者を消費するため、再試行には再準備が必要です。TypeScript は文書の NodeNext 設定でコンパイルし、生成した .mjs を実行します。',
    'json-update': 'v1.json、v2.json、cell.png、tone.wav を recipe/ に置き、Rust・Python・Node のいずれかの完全な例で JSON を読み込みます。namespace biology-json と3個の業務キーを維持し、各版は3ノート・3カード・2メディア、v2 は最初の回答を変更します。最初に配布した v1.apkg を変更しない比較基準として保存し、v2 は別の出力パスへ公開します。Anki からの再エクスポートで代用できません。comparison.json/review.json の想定結果は RISK.NOTE_CHANGED で、追加・削除ノートはなく、ポリシーが許可した同じ準備済み候補を公開します。ノートの省略は利用者のコレクションからの削除ではなく、更新や復習履歴の挙動は対象クライアントでの検証が必要です。',
    'compatibility-constraints': '記載は 2026-10-13 に確認された 0.3.1 の検証記録です。Rust 1.92、Node 22.13 以降（22/24/26 で確認）、通常の CPython 3.11〜3.14 を扱います。ネイティブパッケージは macOS arm64/x64、Linux x64 GNU/glibc、Windows x64 向けですが、存在するだけですべてのホストを検証済みとはいえません。他の Python 実装、free-threaded・サブインタープリター、ブラウザー・Electron・Bun・musl・ARM64 Linux/Windows の Node は別途検証が必要です。Anki のインポート・描画・再生はクライアントや設定・コーデックに依存します。元の APKG を保持してください。原子的な置換はファイル単位で、別の JSON レポートとの一括トランザクションではありません。公開後の遅い失敗もあるため、再試行前に stage と durability を確認します。'
  },
  de: {
    'python-install': 'Der Schnellstart beschreibt 0.3.1 mit regulärem CPython ab 3.11 und passendem nativen Wheel; dokumentierte Prüfungen decken 3.11–3.14 ab. Erstellen Sie die Umgebung mit python -m venv .venv, aktivieren Sie sie mit . .venv/bin/activate (PowerShell: .venv\\Scripts\\Activate.ps1) und installieren Sie python -m pip install ankiforge==0.3.1. Speichern Sie das vollständige Beispiel als main.py und führen Sie python main.py aus. Namespace spanish und Notizschlüssel es:hola erzeugen die persistente spanish.apkg mit einer Basic-Karte hola → hello im Stapel Spanish. Behalten Sie beide Kennungen bei Aktualisierungen bei. Der Export benötigt kein Anki; den Import prüfen Sie separat im Client.',
    'node-api': 'Verwenden Sie Node ab 22.13 und npm install --include=optional ankiforge@0.3.1, damit die passende native Abhängigkeit installiert wird. Erzeugen Sie Project(namespace), fügen Sie Notizen mit stabilen Schlüsseln hinzu und warten Sie auf project.build(BuildOptions.to(path)). BuildOutput hält das Artefakt und bietet Bericht/Snapshot; BuildReport enthält Beobachtungen, hält keine Datei und ist allein kein Operationsergebnis. Solange Sie einen temporären Pfad verwenden, muss ein Besitzer bestehen: Das Schließen des letzten löscht die temporäre Datei; persistente Dateien bleiben erhalten. build/compare erfassen den Projektzustand beim Aufruf. Jeder vorbereitete Veröffentlichungsversuch verbraucht den Besitzer, auch bei Richtlinien- oder E/A-Fehlern; ein weiterer Versuch benötigt neue Vorbereitung. TypeScript wird mit der beschriebenen NodeNext-Konfiguration kompiliert, anschließend läuft die erzeugte .mjs.',
    'json-update': 'Laden Sie v1.json, v2.json, cell.png und tone.wav nach recipe/ und verwenden Sie eines der vollständigen Rust-, Python- oder Node-Programme. Das Programm liest JSON ein und behält Namespace biology-json sowie die drei fachlichen Notiz-IDs bei. Jede Version enthält drei Notizen/Karten und zwei Medien; v2 ändert die erste Antwort. Bewahren Sie die ursprünglich verteilte v1.apkg als unveränderte Vergleichsbasis auf und wählen Sie für v2 einen anderen Pfad. Ein Anki-Reexport ist kein geeigneter Ersatz. Prüfen Sie comparison.json/review.json: Erwartet wird RISK.NOTE_CHANGED ohne hinzugefügte oder entfernte Notizen; erst nach Richtlinienfreigabe wird derselbe vorbereitete Kandidat veröffentlicht. Weggelassene Notizen löschen keine Nutzerdaten; Aktualisierungen und Wiederholungsverlauf müssen im Zielclient geprüft werden.',
    'compatibility-constraints': 'Die Seite dokumentiert 0.3.1 zum Prüfstand 2026-10-13: Rust 1.92, Node ab 22.13 mit Prüfungen unter 22/24/26 und reguläres CPython 3.11–3.14. Native Pakete gibt es für macOS arm64/x64, Linux x64 GNU/glibc und Windows x64; Verfügbarkeit belegt nicht die Ausführung auf jedem Host. Andere Python-Implementierungen, free-threaded/Subinterpreter sowie Node in Browsern/Electron/Bun/musl/ARM64 Linux oder Windows erfordern eigene Prüfung. Anki-Import, Darstellung und Wiedergabe hängen von Client, Einstellungen und Codecs ab. Behalten Sie die originale APKG als Basis. Atomarer Austausch gilt je Datei, nicht gemeinsam mit einem separaten JSON-Bericht. Ein später Fehler kann nach der Veröffentlichung auftreten; prüfen Sie stage und durability vor einem erneuten Versuch.'
  }
};

const ids = ['python-install', 'node-api', 'json-update', 'compatibility-constraints'];
const selectedTopics = {
  'python-install': ['python-quickstart', 'compatibility'],
  'node-api': ['node-api', 'node-quickstart'],
  'json-update': ['json-to-anki'],
  'compatibility-constraints': ['compatibility', 'build-guarantees']
};
const passageScope = {
  'python-install': 'Current same-locale llms discovery link; quickstart installation commands, complete basic program and expected output; compatibility runtime/native-wheel restrictions.',
  'node-api': 'Current same-locale llms discovery link; Node API project/build/report/ownership/concurrency/prepared-publication/TypeScript passages and quickstart runtime/npm command. Not a line-by-line re-audit of every API member.',
  'json-update': 'Current same-locale llms discovery link; recipe inputs/schema, invocation commands and result/baseline/policy/client-limit prose. Complete three-language program execution and line-by-line code auditing are not claimed here.',
  'compatibility-constraints': 'Current same-locale llms discovery link; complete compatibility page plus build-guarantees artifact lifetime, publication/late-error/atomicity and prepared-publication passages.'
};
const comparisonPath = `${directory}/machine-dual-configuration-comparison.json`;
const comparison = await parse(comparisonPath);
assert.equal(comparison.inputSha256, expectedInput);
assert.equal(comparison.records.length, 192);
const conclusionsPath = `${directory}/machine-task-conclusions.json`;
const conclusions = {
  schemaVersion: 1,
  reviewerType: 'ai',
  reviewerRun: '/root/standards_review/independent-machine-semantic-review-2026-11-01',
  scope,
  inputSha256: expectedInput,
  method: 'Fresh answers authored after reading current captured prefix task passages in all six locales. All current root llms task links and primary Markdown headers/canonical URLs/language declarations were separately read; all 192 root/prefix machine bodies were independently compared with only the exact configured origin/base replaced. Root answers reuse this current prefix semantic reading under that proven byte equivalence, not an alleged second complete prose reading. Historical same-configuration machine bytes were also compared, but old answers were not copied. HTTP capture is distinct from semantic review: 60 bodies/configuration were fetched, 192 files/configuration were integrity-checked, and semantic reading is limited to the stated task passages. No SDK commands were executed by this machine-read review, and no production hosting behavior is asserted.',
  passageScope,
  configurationReading: {
    prefix: 'Read actual captured same-locale task passages; commands, requirements, outputs and limits independently assessed for the four tasks.',
    root: 'Actually fetched the same task resources in a separate loopback session. Separately re-read all six llms task links and all 24 primary task headers, canonical URLs and language declarations. Applied current prefix semantic conclusions only after proving complete machine-body equality under exact origin/base substitution.'
  },
  answers,
  openIssues: []
};
await save(conclusionsPath, conclusions);
const conclusionsHash = sha(await readFile(conclusionsPath));
const comparisonHash = sha(await readFile(comparisonPath));
const results = [];
for (const environment of ['prefix', 'root']) {
  const manifestPath = `${directory}/${environment}-site-manifest.json`;
  const manifestBytes = await readFile(manifestPath);
  const manifest = JSON.parse(manifestBytes);
  assert.equal(manifest.inputSha256, expectedInput);
  assert.equal(manifest.commit, '6d4625639fc1477222b41880d29724c162baa3be');
  const archive = `target/multilingual-review-fixes/${environment}-dist`;
  assert.equal(sha(await readFile(`${archive}/generated/site-manifest.json`)), sha(manifestBytes));
  const observationsPath = `${directory}/machine-${environment}/observations.json`;
  const observationsBytes = await readFile(observationsPath);
  const observations = JSON.parse(observationsBytes);
  assert.equal(observations.manifest.sha256, sha(manifestBytes));
  assert.equal(observations.manifest.inputSha256, expectedInput);
  assert.equal(observations.responses.length, 60);
  assert.equal(observations.inventory.length, 192);
  const machine = manifest.resources.filter(resource => /\/(?:index\.md|llms\.txt)$/.test(resource.route));
  assert.equal(machine.length, 192);
  const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
  for (const resource of machine) {
    assert.ok(resource.route.startsWith(base + '/'));
    const bytes = await readFile(path.join(archive, resource.route.slice(base.length + 1)));
    assert.equal(bytes.length, resource.bytes);
    assert.equal(sha(bytes), resource.sha256);
    new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    const observed = observations.inventory.find(value => value.route === resource.route);
    assert.equal(observed.sha256, resource.sha256);
    assert.equal(observed.changed, false);
  }
  const bindings = new Map(machine.map(resource => [resource.route, resource.sha256]));
  for (const response of observations.responses) {
    assert.equal(response.status, 200);
    assert.equal(response.url, observations.readOrigin + response.route);
    assert.equal(response.sha256, bindings.get(response.route));
    const body = await readFile(response.bodyPath);
    assert.equal(sha(body), response.sha256);
    assert.equal(body.length, response.bytes);
  }
  const entries = {};
  for (const locale of manifest.releaseLocales) {
    assert.deepEqual(Object.keys(answers[locale]).sort(), [...ids].sort());
    const home = manifest.pages.find(page => page.id === 'home' && page.locale === locale);
    const llmsRoute = `${home.route}llms.txt`;
    const llmsResponse = observations.responses.find(response => response.route === llmsRoute);
    const llms = await readFile(llmsResponse.bodyPath, 'utf8');
    const links = [...llms.matchAll(/\]\((https?:\/\/[^\s)]+)\)/g)].map(match => new URL(match[1]));
    const taskResults = [];
    for (const id of ids) {
      const selectedRoutes = [llmsRoute, ...selectedTopics[id].map(topic => `${home.route}docs/${topic}/index.md`)];
      const actual = observations.readings[locale][id].filter(record => selectedRoutes.includes(record.route));
      assert.deepEqual(actual.map(record => record.route), selectedRoutes);
      for (const record of actual.slice(1)) {
        assert.ok(links.some(url => url.origin === manifest.configuration.site && url.pathname === record.route));
        const markdown = await readFile(record.bodyPath, 'utf8');
        assert.ok(markdown.includes(`Language: ${locale}\n`), record.route);
        assert.ok(markdown.includes(`Canonical page: ${manifest.configuration.site}${record.route.slice(0, -'index.md'.length)}\n`), record.route);
      }
      const citationLabel = { en: 'Sources', 'zh-CN': '来源', es: 'Fuentes', 'pt-BR': 'Fontes', ja: '参照', de: 'Quellen' }[locale];
      const evidence = [
        { path: observationsPath, sha256: sha(observationsBytes), description: 'Actual loopback HTTP responses and saved body bytes from this frozen archive; complete 192-resource inventory and exact same-configuration historical-byte comparison. Capture alone does not assert semantic approval.' },
        { path: conclusionsPath, sha256: conclusionsHash, description: 'Fresh independent task answers, exact passage reading scope, root/prefix semantic-review reuse method and isolated-candidate limitation.' },
        { path: comparisonPath, sha256: comparisonHash, description: 'All 192 current root/prefix machine resources compared after only exact configured origin/base substitution; no other prose/code/whitespace normalization.' },
        ...actual.map(record => {
          const response = observations.responses.find(item => item.url === record.url);
          assert.ok(response);
          return { path: record.bodyPath, sha256: response.sha256, description: `Actual UTF-8 HTTP 200 response from ${record.url}; digest equals the current archive manifest. Task passage reading scope is recorded in machine-task-conclusions.json.` };
        })
      ];
      taskResults.push({
        id, status: 'passed',
        readUrls: actual.map(record => record.url),
        answer: `${answers[locale][id]} ${citationLabel}: ${actual.slice(1).map(record => record.url).join(', ')}`,
        evidence,
        openIssues: []
      });
    }
    entries[locale] = { aiMachineReadReview: taskResults, openIssues: [] };
  }
  const report = {
    schemaVersion: 1, kind: 'machine-read', status: 'passed', mode: 'release-candidate', reviewerType: 'ai',
    reviewerRun: `/root/standards_review/independent-machine-${environment}-2026-11-01`,
    inputSha256: manifest.inputSha256, configuration: manifest.configuration, releaseLocales: manifest.releaseLocales,
    readOrigin: observations.readOrigin,
    originManifest: { path: manifestPath, sha256: sha(manifestBytes), commit: manifest.commit, description: 'Actual frozen isolated-candidate manifest whose archive files were served and read in this run.' },
    resourceBindings: machine.map(({ route, sha256 }) => ({ route, sha256 })),
    entries, openIssues: [], scope,
    method: `${conclusions.method} ${conclusions.configurationReading[environment]} The readUrls record URLs actually fetched during the completed local session; that temporary server has been stopped. Local response MIME does not certify deployed hosting MIME. No cross-locale fallback was used.`
  };
  const filename = `${directory}/machine-read-${environment}.json`;
  await save(filename, report);
  const bytes = await readFile(filename);
  results.push({ environment, path: filename, sha256: sha(bytes), bytes: bytes.length, tasks: Object.values(entries).reduce((total, entry) => total + entry.aiMachineReadReview.length, 0), verifiedMachineResources: machine.length, actualHttpResponses: observations.responses.length, readOrigin: observations.readOrigin });
}
await save(`${directory}/machine-report-binding-check.json`, {
  schemaVersion: 1, reviewerRun: '/root/standards_review/machine-final-binding-check-2026-11-01', scope,
  inputSha256: expectedInput, status: 'passed',
  checks: ['Both archived manifest bytes match the reviewed evidence manifests.', 'All 384 resource bindings match actual frozen archive byte digests and sizes; UTF-8 decoding succeeds.', 'All 120 actual saved HTTP response bodies match their captured and manifest digests.', 'Each of the 48 task entries starts at its actual locale llms URL, follows explicit same-locale links, and uses matching canonical/language declarations.', 'Answers and declared reading scope were authored after current task-passage review; capture status was not promoted as a substitute for semantic review.'],
  reports: results
});
console.log(JSON.stringify(results, null, 2));
