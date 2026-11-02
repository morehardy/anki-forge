# Python API の刷新

`python -m pip install ankiforge` と `from ankiforge import ...` を使います。以前の配布名 `anki-forge` とインポート名 `anki_forge` は、互換エイリアスではありません。ソースをチェックアウトして使う場合、名前を変更した後は Maturin でネイティブ拡張を再ビルドしてください。[セットアップガイド](https://github.com/morehardy/anki-forge/blob/main/bindings/python/README.md)を参照してください。

旧 API の互換レイヤーは残していません。従来の Deck・レジストリ・ランタイムの API を、データを所有する公開モデルに置き換えます。

| 従来の手順 | 現在の手順 |
| --- | --- |
| 安定 ID が任意の Deck または Project | `Project(namespace, default_deck=...)` |
| `add_note(note)` と内容由来の識別情報 | `add(stable_key, note)` |
| 変更可能な NoteType と add_notetype | `NoteType.builder(key)...build()` の後に `model.note()` |
| 文字列によるモデル参照 | ノートが検証済みモデルを所有 |
| プロジェクトのメディアレジストリ | 独立した `Media.file` / `Media.bytes` スナップショット |
| インラインで描画したメディアとフィールド専用 setter | `Content.sequence`、`media.image()`、`media.sound()`、`Note.field` |
| 暗黙の HTML | 文字列は Text。HTML には `Content.html` を使う |
| 安定した識別情報のないマスク | `Mask.rect(key, x, y, width, height)` |
| 成果物がない場合もある戻り値と ensure_success | 成果物が保証された `BuildOutput`。失敗時は構造化エラーを送出 |
| 診断からレポートの成功・失敗を推測 | Output/Error のスナップショットが実際の結果を示す。Report は観測結果のみ |
| Diff、lockfile、更新安全性の切り替え | `compare(CompareOptions.against(...))`、`BuildOptions.update_from(...)` |
| 実行時のサブプロセス用パッケージ | デフォルト Rust API へのプロセス内 PyO3 バインディング |

更新の基準に使えるのは、現在の識別情報の証拠を持つ完全な配布時のオリジナル APKG だけです。旧バンドルや古い APKG の証拠を自動復元することはありません。新しい手順は README と実行可能なサンプルを参照してください。
