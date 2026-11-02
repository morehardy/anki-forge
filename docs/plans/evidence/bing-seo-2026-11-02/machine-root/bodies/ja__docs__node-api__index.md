# Node と TypeScript API

Canonical page: https://ankiforge.dev/ja/docs/node-api/
Language: ja
Description: Node SDK の作成メソッド、非同期処理、メディア、所有権、型付きレポート、TypeScript の設定を確認できます。

`ankiforge` の公開 API では、`Project` を中心にノートやメディアを組み立てます。型は[公開ソース](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/index.ts)、[レポートクラス](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/report.ts)、[JSON スナップショットの型宣言](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/snapshots.ts)にあります。[クイックスタート](https://ankiforge.dev/ja/docs/node-quickstart/)から始めてください。

## プロジェクトとノート

| API | 動作 |
| --- | --- |
| `new Project(namespace)` | 配布物の安定した名前空間を検証する |
| `.defaultDeck(name)` | 既定の配置先を設定し、プロジェクトを返す |
| `.add(key, note)` | ノート・モデル・メディアをアトミックに収集する。キーの重複は失敗する |
| `.addAsset(media)` | 直接記述した HTML・CSS・スクリプト用の明示的リソースを追加する |
| `.clone()` | 不変の値を共有する、独立して編集可能なプロジェクトを返す |
| `.length` | 登録済みノート数 |
| `Note.basic(front, back)` / `Note.cloze(text)` | 組み込みモデルを保持する不変のノート |
| `model.note().field(key, content)` | モデルを保持する不変の独自ノート |
| `note.deck(name).tag(tag).tags(iterable)` | 新たに設定したノートを返す |

Cloze の文字列を含め、文字列は常に Text になります。マークアップを明示するには `Content.html` を使います。`Content.sequence(iterable)` はテキスト、HTML、型付きメディアを組み合わせます。明示的な業務キーを使ってください。内容や表示名から識別情報を導くことはありません。

## 不変のモデル

```js
import { NoteType, Field, Template, GenerationRule } from 'ankiforge';
const model = NoteType.builder('vocabulary')
  .name('词汇')
  .field(new Field('front', { name: '正面', required: true, sort: true }))
  .field(new Field('back', { name: '背面' }))
  .template(new Template('recognition', {
    front: '{{front}}',
    back: '{{FrontSide}}<hr>{{back}}',
    generation: GenerationRule.all(['front']),
  }))
  .build();
const note = model.note().field('front', 'cell').field('back', '细胞');
```

ビルダーは新しい値を返します。完成済みモデルにはフィールド・テンプレートを変更するメソッドがありません。`builder.clozeField(key)` は独自モデルに Cloze の動作を指定します。`builder.css` と `builder.asset` はスタイルと明示的な依存関係を設定します。テンプレートには `name`、`browserFront`、`browserBack`、`targetDeck` も指定できます。テンプレート参照と生成条件はフィールドキーを使い、表示名は Anki 向けにコンパイルされます。`GenerationRule.ankiDefault()` は Anki の既定動作、`all` と `any` は明示的な条件を選択します。

`await NoteType.fromBundle(path, mediaLimits?)` は `template-bundle-v2` から同じ不変モデルを返します。成功後、読み込んだテキスト・アセットは元のディレクトリに依存しません。モデルはプロジェクト間で再使用できます。同じモデルキーで定義が両立しない場合は、部分的に変更せず失敗します。

## メディアと画像穴埋め

`await Media.file(path, limits?)` と `await Media.bytes(Uint8Array, mime, limits?)` は、データを所有するスナップショットを返します。await が成功した後は元ファイルを変更・削除できます。`withExportName(name)` は、それ以前のクローンやコンテンツを変更せず、改名した値を返します。`filename`、`mediaType`、`byteLength` はプロパティとして読み取ります。

`media.image()` と `.sound()` は、プロジェクトへのアセット追加を自動で行う型付き Content を返します。直接記述した HTML・CSS・スクリプトの参照には、明示的な `project.addAsset` とモデルの `builder.asset` を使います。メディア上限は `{ maxBytes }` を受け付けます。既定値は256 MiBで、ネイティブバイト列に64 KiBのインライン上限はありません。

```js
import { Media, Note, Mask } from 'ankiforge';
const image = await Media.file('diagram.png');
const note = Note.imageOcclusion(image)
  .mask(Mask.rect('nucleus', 10, 10, 20, 20))
  .mask(Mask.rect('wall', 50, 50, 20, 20))
  .mode('hide_one_guess_one')
  .build().field('header', 'Cell');
```

既定のモードは `hide_all_guess_one` です。どちらのモードもキー付きマスクごとに別のカードを生成します。この画像穴埋めビルダーの `.build()` は、デコード後のサイズ、有限の矩形、一意のキーを検証します。`image` と `occlusion` は予約済みの自動生成フィールドです。デコードと更新の制約は[画像穴埋め](https://ankiforge.dev/ja/docs/image-occlusion/)を参照してください。

## ビルド、比較、出力

| API | 結果 |
| --- | --- |
| `await project.build(BuildOptions.to(path))` | 永続成果物を所有する BuildOutput |
| `await project.build(BuildOptions.temporary())` | 一時成果物を所有する BuildOutput |
| `await project.compare(CompareOptions.against(path))` | ポリシーによる公開拒否も含む、完了した ComparisonReport |
| `options.updateFrom(path)` | 配布時のオリジナルパッケージの完全な証拠を使った更新を設定 |
| `options.inspectLimits(limits)` | 独立した検査カウンターを上書き |
| `options.updatePolicy(policy)` | 更新・比較のリスクポリシーを設定 |

成功した出力は `.artifact`、`.report`、`.snapshot()` を公開します。`BuildReport` は `.counts`、`.baselineCounts`、`.durationMs`、`.diagnostics`、`.comparison`、`.snapshot()` を公開します。成功・失敗の結果やファイルの所有権は持ちません。`.comparison` は `ComparisonReport` または `null` で、`project.compare()` と同じクラスを使います。比較は `.findings`、`.highestRisk`、`.policy`、`.diagnostics`、`.snapshot()` を公開します。スナップショットのフィールド名は、`allows_publication` や `schema_version` を含めて Rust の JSON と一致します。ネイティブレポートを camelCase にしたコピーではありません。`null` でないことを確認した後、比較オブジェクトでは `report.comparison.highestRisk`、JSON スナップショットでは `report.snapshot().comparison.highest_risk` を使います。

`new UpdatePolicy()` は High/Critical を拒否します。`.failOn('medium')` はしきい値を変更します。`.allow('RISK.NOTE_REMOVED')` は証拠と重大度を保持したまま、その分類全体を明示的に許可します。不明なコードとハードエラーは許可できません。新規作成だけのビルドにポリシーを指定するとエラーになります。比較とビルドの公開判断を一致させるには、同じポリシーと上限を使ってください。

検査オプションには `maxArchiveBytes`、`maxEntries`、`maxCentralDirectoryBytes`、`maxZipEntryBytes`、`maxZipTotalBytes`、`maxMetaBytes`、`maxMediaMapBytes`、`maxIdentityBytes`、`maxCollectionBytes`、`maxMediaBytes`、`maxDecodedTotalBytes`、`maxZstdWindowBytes` があります。値は非負の安全な整数、または u64 の最大値までの `bigint` です。0は実際に上限0を意味します。カウンターは基準パッケージと候補に独立して適用します。不明なオプションはエラーになります。

## 成果物の生存期間と並行処理

JSON スナップショットのパスは、エクスポートされた `PathSnapshot` 型を使います。Unicode パスは文字列です。Unicode でないネイティブパスは `{ encoding: "unix_bytes", bytes: number[] }` または `{ encoding: "windows_wide", units: number[] }` になります。配列は正確な Unix のバイト列または Windows の UTF-16 コード単位を保持します。ビルドスナップショットの成果物パス、失敗時の公開パス、構造化エラーのパス詳細が対象です。エンコード済みのパスは元のプラットフォームでのみデコードしてください。Unix では `Buffer.from(value.bytes)` が、Node のファイルシステム関数で使えるバイト列のパスを保持します。Node の作成メソッドは引き続き文字列のパスを受け付けます。

相対的なメディア・バンドルのパスは読み取りの呼び出し時に、ビルド先と基準パッケージはオプション作成時に基準ディレクトリが確定します。POSIX では、シンボリックリンクの後の `..` はファイルシステムでのパス走査の意味を保持します。`persistTo` は、クローン済み・永続化済みハンドルも含め、元のビルドが記録した作業ディレクトリを相対保存先の基準にします。後から作業ディレクトリを変えても、これらのパスは変わりません。

`.path` を利用している間は成果物の所有者を保持します。`.clone()` は独立した所有者を作り、`await .close()` はその所有者を解放します。最後の一時所有者が解放されるとファイルが削除されます。`await .persistTo(path)` は永続成果物を返し、失敗しても元の成果物は使用できます。所有者を保持していれば、標準の Node ファイルストリームで成果物を読み取れます。

スナップショットは凍結済みの JSON データで、ネイティブファイルを保持しません。永続ファイルは所有者の解放後も残ります。`BuildOutput` は呼び出し側で構築できず、成功したビルドによってのみ作成されます。

ビルドと比較は、呼び出し時に取得した所有付きのプロジェクトスナップショット上で非同期実行します。後の編集でその要求が変わることはありません。メディアの読み取りとバンドルの読み込みもワーカーで実行します。Node の ESM と CJS は同じネイティブモジュールとクラス識別子を共有します。

## エラー

`SchemaError`、`AddError`、`MediaError`、`TemplateBundleError`、`ImageOcclusionError`、`PolicyError`、`ConfigurationError`、`CompareError`、`BuildError`、`PersistError` は `ForgeError` を継承します。`kind`、`code`、`domain`、ネイティブ原因チェーンのテキスト `causes`、認識可能な I/O・スキーマ・メディア・上限の原因を構造化した `sourceDetails`、操作の `details` を公開します。元のネイティブ例外は JS の `cause` に保持されます。

`BuildError.snapshot()` は失敗と公開状況を保持します。`BuildError.report` と `CompareError.report` は `BuildOutput.report` と同じ BuildReport を返します。これらのレポートでは `.baselineCounts` と `.comparison` を使い、snake\_case の JSON フィールドには `.snapshot()` を使います。失敗した比較では、一部の観測結果と `null` の比較が含まれる場合があります。`PersistError.publication` は、保存先がすでに公開されたか、耐久性が確認されたかを記録します。メディア・検査エラーは上限の詳細を保持します。エラーメッセージの解析ではなく、これらの機械処理用フィールドを使ってください。`ArtifactClosedError` は閉じた所有者の使用、`NativeLoadError` はネイティブパッケージの欠落・非互換・古いバージョンを報告します。

## 追加時のコンテキスト

追加に失敗してもプロジェクトは変わりません。構造化コンテキストには元のノート・モデルのキーと対象を記録し、必要に応じて競合やメディア使用の詳細を付けます。Rust では `AddError::context()` / `detail()` を通じて `note::{AddContext, AddTarget, AddDetail}` を取得できます。SDK では `AddError.details` が型付きの context/detail を提供します。Node のフィールドは camelCase で深く凍結されています。Python は snake\_case の TypedDict を使います。タグは挿入時のインデックスを保持します。フィールドの位置は、元のシーケンスインデックス（フィールド自体は null/None、ルートは \[\]）と任意の UTF-8 バイト範囲を保持します。ビルド時の既定デッキのエラーはノート・モデルのキーを持たず、原因チェーンから確認できます。

型付き画像を追加する際は image/\* MIME が必要です。音声参照には audio/\* または video/\* を受け付けます。他の種類は `NOTE.MEDIA_USAGE_INVALID` になります。コンストラクター自体は失敗せず、明示的アセットと直接記述した HTML には影響しません。検査はインポート時に保持した MIME を使い、再生を保証しません。ビルド時の独立した MIME・拡張子検証も引き続き適用されます。対象と詳細の完全な対応表は[設計文書](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md)を参照してください。

## メディアの一括処理と候補ファイルの事前生成

メディアの一括処理と候補ファイルの事前生成は、公開 `0.3.1` パッケージに含まれています。

### 準備、確認、公開

```js
import { Project, Note, BuildOptions, BuildError } from 'ankiforge';
const project = new Project('review-once').add('one', Note.basic('Question', 'Answer'));
const previous = await project.build(BuildOptions.to('previous.apkg'));
await previous.artifact.close();
const prepared = await project.preparePublication(
  BuildOptions.to('next.apkg').updateFrom('previous.apkg'),
);
try {
  const report = prepared.report; // immutable observations, no file ownership
  console.log(report.comparison?.snapshot());
  if (report.comparison?.policy.allows_publication !== false) {
    const output = await prepared.publish();
    await output.artifact.close(); // persistent next.apkg remains
  }
} catch (error) {
  if (error instanceof BuildError) console.log(error.snapshot().result);
  // Inspect publication/durability facts: a late failure can follow replacement.
  throw error;
} finally {
  await prepared.close(); // idempotent; cleans an unpublished candidate
}
```

`publish()` はネイティブの所有者を即座に取得し、1つのワーカータスクを実行します。ポリシーや I/O による拒否を含め、すべての試行で所有者を消費します。再呼び出しは `PreparedPublicationStateError` で拒否され、コードは `BUILD.PREPARED_UNAVAILABLE`、`reason` は `consumed` または `closed` です。公開開始後に閉じてもキャンセルされません。閉じた後もレポートは読めます。オプションは準備の呼び出し時に確定します。ポリシーや保存先を変えるには、再び準備する必要があります。所要時間には確認待ち時間を含みません。レポートや JSON を所有者へ戻すことはできません。

ビルド、比較、準備、クローンは、不変のネイティブ Project のバージョンを共有します。作成時の変更は引き続き同期的で、互いに独立しています。別のスナップショットが生存している間の最初の変更では、Project がコピーされる場合があります。その後に排他的に行う変更ではコピーしません。これによりコピー処理は Promise の送信時から移りますが、最初の編集の高速化、最悪タイマー遅延の改善、RSS の削減を保証するものではありません。JavaScript のバイト列入力は、引き続きワーカー送信前にコピーします。これらのラッパーにはプロトコル6のバイナリーを使ってください。

## TypeScript

アプリケーションに `typescript` と `@types/node` をインストールします。`main.mts` または `"type": "module"` を使い、次の `tsconfig.json` を設定します。

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "outDir": "dist"
  }
}
```

`npx tsc` でコンパイルします。`.mts` のエントリーなら `node dist/main.mjs` で実行します。
