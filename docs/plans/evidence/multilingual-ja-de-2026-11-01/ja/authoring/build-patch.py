import pathlib,json
p=pathlib.Path(__file__).parent
brief=json.loads(pathlib.Path('docs/plans/evidence/remaining-locales-2026-11-01/samples/ja/language-brief.json').read_text());brief['status']='draft/full-candidate'
prompt='''{language} アプリケーションで Anki Forge {version} を使う作業を手伝ってください。
実行環境の要件: {runtime}

公開パッケージ `ankiforge` と、このバージョンの文書に記載された公開 API だけを使ってください。
git/path 依存関係、内部ツール、ソース専用 API、未公開のメソッド、推測したメソッドは使わないでください。

作りたいもの:
{request}

次の要件に従ってください。
1. ファイルを変更する前に、既存プロジェクトを確認します。
2. 必要な設定と正確なインストールコマンドを示します: {install}
3. 最小限かつ完全に実行できるプログラムを作り、追加・編集するすべてのファイル名を示します。
4. 永続的な `.apkg` ファイルを生成し、安定した名前空間とノートキーを使います。
5. 実行コマンド、想定される出力、検証手順を含めます。
6. メディア、独自テンプレート、画像穴埋め、更新が必要な場合は API を推測せず、文書化された手順に従います。
   更新では、前回配布時のオリジナル APKG を基準として保持します。Anki から再エクスポートしたファイルは使いません。
7. 言語、入力形式、出力パス、識別情報のルールが不明な場合に、その解消に必要な質問だけをします。

クイックスタート: {quickstart}'''
patch={'schemaVersion':1,'locale':'ja','documents':json.loads((p/'metadata.json').read_text()),'docGroups':['はじめに','作業別ガイド','言語別ガイド','リファレンス','ヘルプ'],'navigation':{'docs':'ガイド','examples':'サンプル','blog':'ブログ'},'languageBrief':brief,'prompts':{'ja':prompt},'requestPlaceholders':{'ja':'<作りたいものを記入してください>'},'promptContracts':{'ja':{'required':['公開パッケージ','公開 API','永続的','安定した名前空間とノートキー','実行コマンド','検証手順','配布時のオリジナル APKG'],'maxCharacters':1000}},'searchTerms':json.loads((p/'search-terms.json').read_text()),'searchConcepts':json.loads((p/'search-concepts.json').read_text())}
(p/'ja-data-patch.json').write_text(json.dumps(patch,ensure_ascii=False,indent=2)+'\n')
print('documents',len(patch['documents']),'searchTerms',len(patch['searchTerms']),'promptCharacters',len(prompt))
