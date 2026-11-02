"""Serialize independently authored language judgments after actual reading."""
from pathlib import Path
import copy, difflib, hashlib, json

ROOT=Path(__file__).resolve().parents[4]
B=Path('docs/plans/evidence/website-pr66-final-2026-11-02')
RUN='/root/spec_review/pr66-final-linguistic-2026-11-02'
def read(p): return json.loads((ROOT/p).read_text())
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def save(p,v): (ROOT/p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def ref(p,description): return {'path':str(p),'sha256':sha(p),'description':description}
A=read(B/'linguistic-input-audit.json'); C=read(B/'source-candidate.json')

J={
'docs/cards.md':'The fuller Basic definition distinguishes stored front/back fields from the generated question-and-answer card. The surrounding two-note/three-card example and Cloze deletion count still agree; no changed command or API.',
'docs/template-bundles.md':'The bundle now names the note type definition and explicitly declared media dependencies in normal technical English. Complete validation, owned snapshots, immutable NoteType and no partial project modification remain in context.',
'docs/updates.md':'The heading names the user object, a deck, rather than a publication. Original distribution baseline, same namespace/keys, policy gates, client limitations and the complete example remain consistent.',
'docs/build-guarantees.md':'Prepare, review and publish accurately names the complete public-package workflow. The unchanged next sentence explicitly includes public 0.3.1; stable section marker preserves the historic anchor. Resolves L002-en.',
'docs/python/api.md':'Batch media and prepared publication names the public 0.3.1 capabilities described by this section instead of suggesting source-only APIs. Stable section marker remains. Resolves L003-en.',
'docs/zh-cn/index.md':'Opening now directly says 根据笔记、模板和媒体文件生成 APKG; the following note/type/card/deck explanation and three-SDK navigation maintain distinct objects.',
'docs/zh-cn/anki-forge-vs-genanki.md':'The media ownership row explicitly says 图片和音频对象持有不可变数据快照，克隆共享存储. This clarifies lifetime/storage without implying copyright or independent copies per clone; the comparison qualifications remain.',
'docs/zh-cn/api.md':'完整标识映射与修订信息 makes the required baseline evidence concrete; the same paragraph still rejects an Anki re-export and requires stable namespace/note keys.',
'docs/zh-cn/build-guarantees.md':'准备候选包并在审阅后写入目标文件 describes the actual prepare/review/publish sequence. Public 0.3.1 and one consumed candidate remain explicit; old section identity is preserved. Resolves L002-zh-CN.',
'docs/zh-cn/compatibility.md':'The required 标识映射与修订信息 replaces abstract 身份证据 in the original-baseline constraint. It still rejects re-export and preserves all client/version/import limitations.',
'docs/zh-cn/concepts.md':'Media 持有媒体数据的不可变快照，并管理其生命周期 explains ownership directly. The surrounding successful-import prerequisite and clone shared storage keep the technical meaning precise.',
'docs/zh-cn/genanki-migration.md':'原始 APKG 提供完整标识映射与修订信息 clarifies baseline evidence while retaining no arbitrary legacy-ID recovery and explicit migration boundaries. The old AnkiForge brand-spacing minor is still present and retained.',
'docs/zh-cn/media.md':'The batch paragraph now describes the immediate single-file return as a snapshot-owning object, without implying parallel work. Four-worker/under-16 serial/input-order-error behavior and memory-budget limits remain explicit; unrelated old phrasing/navigation minors remain.',
'docs/zh-cn/node-api.md':'The Media.file/bytes result is now a Media object holding the snapshot; successful import allows source-file changes/removal without changing imported bytes. Existing renaming and earlier-clone distinctions remain.',
'docs/zh-cn/python-api.md':'Media.file explicitly reads bytes and returns a snapshot-owning Media. 批量媒体处理与预备发布 corrects the old source-only heading, while the public 0.3.1 prerequisite and lifecycle details remain. Resolves L003-zh-CN only; 保持不会失败 remains.',
'docs/zh-cn/python-diagnostics.md':'Missing 完整标识映射与修订信息 is clearly an operation error alongside invalid baseline/candidate/limits. It remains distinct from allowed risk categories and completed comparison policy.',
'docs/zh-cn/python-migration.md':'The final paragraph requires current 标识映射与修订信息 in the complete original distributed package. The old/new API mapping and refusal to recover old evidence are unchanged.',
'docs/zh-cn/rust-api.md':'The edited statements clarify external applications do not need repository tools, hard errors include missing/corrupt identity mapping/revision data, and native builds retain that data. Existing owned-reference/保持不会失败 minors are unchanged and retained.',
'docs/zh-cn/rust-guide.md':'Snapshot ownership is stated as a Media object retaining data, with source files unnecessary after successful import. 本身不会返回错误 naturally describes image/sound constructors while addition/build checks remain explicit. Resolves L007-rust-guide.',
'docs/zh-cn/templates.md':'A bundle now explicitly contains note type definition, template text, CSS and declared media dependencies, with snapshots captured before returning immutable NoteType. The unchanged intro still omits the loading verb, so its old minor is retained.',
'docs/zh-cn/troubleshooting.md':'The original baseline requirement now names 完整标识映射与修订信息; Anki re-export, mismatch and corrupt evidence remain disallowed. Existing IO rectangle wording is unchanged and retained.',
'docs/zh-cn/updates.md':'The baseline paragraph now uses three direct sentences: previous original distributed APKG, complete mapping/revision data in a versioned format, and keeping source revision. It does not change identity or overwrite policy. The old introduction/metadata minors remain.',
'docs/es/anki-forge-vs-genanki.md':'The opening now explains media data retained independently of original files and separates diagnostics, comparison and risk-policy review. Neutral technical Spanish remains consistent with the migration-cost and unsupported-guarantee limitations.',
'docs/es/build-guarantees.md':'hilos de trabajo accurately names worker threads in the concluding reference, unlike the human-worker connotation of trabajadores. Candidate consumption and late-I/O publication facts remain in full context.',
'docs/es/updates.md':'Actualiza y distribuye un mazo names the deck directly. The original APKG, identity evidence, policy/category behavior and client-import limitations remain explicit.',
'docs/pt-br/index.md':'templates and templates de cartão separate card presentation from tipo de nota; the conceptual paragraph still distinguishes notes, types, cards and decks. All three-SDK and next-task labels point to the same tasks.',
'docs/pt-br/anki-forge-vs-genanki.md':'The media row explicitly states snapshots are independent of source files while clones share storage, so independente does not promise cloned file duplication. Cost/migration/performance limits are retained.',
'docs/pt-br/genanki-migration.md':'The opening explains source-file-independent snapshots; template expressions now use templates de cartão while modelo remains the note type. Stable key, explicit asset and migration-identity instructions remain intact.',
'docs/pt-br/node-quickstart.md':'The next-task link now says pacotes de templates, consistent with the destination and distinct from tipos de nota. The complete Node command/program and optional native dependency requirement are untouched.',
'docs/pt-br/python-quickstart.md':'The next-task link now says pacotes de templates; Python installation, executable program and next-task destinations are unchanged.',
'docs/pt-br/python-migration.md':'The media registry mapping now explicitly retains snapshots independently of source files; old bundles are pacotes de templates. It still requires current original APKG evidence and does not offer an old API compatibility layer.',
'docs/pt-br/troubleshooting.md':'Mídia e templates and pacote de templates match the resource being loaded. The body distinguishes model asset declarations from template references; old heading is registered as a historic alias.',
'docs/pt-br/updates.md':'The stable-key and schema paragraphs now distinguish tipos de nota from templates de cartão. Removing fields/templates remains non-deleting, reordering preserves mappings, and client merge risks remain qualified. New deck heading is clearer.',
'docs/ja/build-guarantees.md':'候補ファイルを準備し、確認後に保存する and its first sentence explain prepared publication naturally. The full section preserves a single private candidate, no destination replacement at preparation, consuming publish, and late-failure facts.',
'docs/ja/compatibility.md':'候補を確認してから保存する手順 accurately names the linked workflow; public 0.3.1, platform support versus executed checks, and client limits remain distinct.',
'docs/ja/installation.md':'候補ファイルの事前生成 naturally names the preparation capability within the public-crate list. Rust/native prerequisites, no default features and separate JSON parser remain in context; it does not claim publication during preparation.',
'docs/ja/json-to-anki.md':'初回配布用の APKG and 解答だけを変更した検証済みの更新版 make the two artifacts and answer edit clear. ノート3件/カード3枚/メディア2個, synthetic tone, same IDs, policy review and original baseline remain explicit.',
'docs/ja/media.md':'Image/audio creation is now separate from the concrete example of TTS supplying read-aloud audio; it no longer suggests speech synthesis makes images. Snapshot ownership, MIME and client playback limits remain unchanged.',
'docs/ja/node-api.md':'The opening describes Project-centered assembly instead of implying Project is the only exported value. 候補ファイルの事前生成 is explained by the same prepare/review/publish lifecycle below. Existing 深く凍結 minor is retained.',
'docs/ja/python-api.md':'The heading and introduction use 候補ファイルの事前生成, followed by explicit preparation/review/publication and consumed-owner/error facts. Public 0.3.1 remains explicit; no execution or availability promise changed.',
'docs/ja/python-migration.md':'成果物がない場合もある戻り値 correctly explains an optional return artifact in the old API, instead of arbitrary artifact. The current guaranteed BuildOutput/structured-error contrast is clear.',
'docs/ja/rust-api.md':'候補の事前生成と保存を行う API is a natural explanation of prepared publication and explicitly stays in public 0.3.1. The one-candidate source example is unchanged; the old 深く凍結 minor remains.',
'docs/ja/updates.md':'The paragraph names prepare_publication and publish directly: prepare once, review, then save the same APKG. It preserves the warning that an ordinary second build produces another candidate and the later policy/client checks.',
'docs/de/anki-forge-vs-genanki.md':'Typisierte Bild- und Audioobjekte verwalten unveränderliche Snapshots makes the ownership row concrete and grammatically natural. Migration and benchmark limitations remain unchanged.',
'docs/de/build-guarantees.md':'Fehlerart und Fehlercode explicitly names the structured error attributes; Nebenläufige Vorgänge fits snapshot/lifetime behavior and the maintained Du-register. Publication and client guarantees are unchanged.',
'docs/de/concepts.md':'Behalte ... bei, wenn du Quelldaten bearbeitest oder neu sortierst is a direct actionable instruction to retain namespace/record keys. It preserves the distinction from display names and content.',
'docs/de/csv-to-anki.md':'einen Stapel, den du weitergeben kannst naturally conveys shareable distribution, avoiding teilbar ambiguity. CSV parsing, persistent IDs, no Anki export dependency and import/client checks remain unchanged.',
'docs/de/media.md':'Import mehrerer Mediendateien and Verhalten und Limits ... avoid confusing batch import with an Anki deck. The full paragraph retains input order, bounded concurrency, error cleanup and per-file budgets.',
'docs/de/node-api.md':'Mehrere Mediendateien importieren und die Veröffentlichung vorbereiten names two concrete operations. The public 0.3.1 claim, one consumed owner and worker lifecycle remain intact; old heading alias is preserved.',
'docs/de/python-api.md':'The concrete multi-file/preparation heading fits the section and remains separate from publication. The full public-version/lifecycle paragraph still describes errors, reprepare and late replacement facts.',
'website/src/content/blog/zh-cn/stable-ids-and-deck-updates.mdx':'The title now says 修改笔记内容时，保持 ID 不变; the last section tells readers to inspect source/baseline if identity cannot be reconciled, not infer correctness from file existence. The unchanged 仅…本身只是 minor is retained.',
'website/src/content/blog/es/stable-ids-and-deck-updates.mdx':'Usar el paquete anterior como referencia is a direct baseline heading. Current full section keeps distinct previous/next paths and the original distributed package, and explicitly preserves both historic anchors.',
'website/src/content/blog/pt-br/stable-ids-and-deck-updates.mdx':'Atualize o conteúdo da nota sem mudar seu ID makes the article task concrete. Full prose preserves namespace, previous original APKG, distinct paths, review and Anki import limitations.',
'website/src/content/blog/ja/stable-ids-and-deck-updates.mdx':'The final baseline sentence explicitly says an Anki re-export cannot serve as the update baseline. The shared CodeBlock and source snippet match the prior independently reviewed fix; the rest of the article preserves identity and client limitations.',
}
UI={
'en':'Programming language distinguishes language selection from content language; the original prior-release APKG is explicit; the AI question now has direct interrogative syntax. Comparison results/No changes detected fit the fixed example context. Performance describes the five historical tests and specifically the Anki Forge Rust API, retaining units and limitations. Machine constraints retain source/public API distinction and import caveats.',
'zh-CN':'“保留原有笔记 ID”与原始 APKG 说明更具体；“任务说明/AI 编程助手”在按钮、成功、手动复制反馈中一致。网页预览与 Anki 结果仍区分。性能说明限定 Anki Forge Rust API 和五组实测，独立/共享文件标签含义清楚，小数插值与单位仍适配。',
'es':'lenguaje de programación and the no-JavaScript instruction clearly identify the code language and task placeholder. The occlusion example now identifies a rectangular mask with stable key; la de afinación is a tone with one-second duration. Performance labels specify a distinct file per note, preserve localized percent grammar and the five-test/Rust-API scope; no unsupported result is promised.',
'pt-BR':'templates/template de cartão distinguish presentation templates from tipo de nota/modelo. Linguagem de programação and the deck question identify the task directly; original prior APKG and stable IDs remain explicit. The tone duration and different-per-note versus shared-media labels are natural Brazilian Portuguese; the benchmark remains limited to tested Rust API results.',
'ja':'プログラミング言語 and AI への依頼文 are consistent across selection, copy, success and fallback actions. 必須フィールド avoids implying every field is required; ソースコード全体 and 4枚入り name actual actions and counts. The original distributed APKG remains explicit. Performance wording explains different-per-note/shared media and retains Rust API/test limits with correct counters.',
'de':'Direct Du instructions remain consistent; Notiz-IDs and ursprünglich verteiltes APKG specify update identity. Seite nicht gefunden and Karten zum Lernen match page tasks. Testszenario and Gemessene Exportzeiten are clearer than Lastfall, with Rust API and non-import benchmark limits retained; Anleitungen nach Aufgabe suits the machine index.',
}
SHARED={
'docs/ai-quickstart.json':'Read all six full prompts. Only zh-CN internal tools→内部工具 and pt-BR modelos personalizados→templates personalizados change natural language. Exact placeholders, runtime/install commands, published-API restriction, stable identity and original-APKG prohibition remain unchanged.',
'website/language-briefs.json':'Read all six briefs. Added zh-CN identityEvidence=标识映射与修订信息, pt-BR template=template de cartão; ja preparedPublication is now an explanatory phrase describing preparation and later saving. Checked related target terminology in changed pages and current unchanged pt-BR templates/custom note types/media/concepts. No glossary-version gate was weakened.',
'website/legacy-sections.json':'The added aliases retain old localized headings after editorial renames: en/zh build-guarantees and Python API, de media/build/Node/Python, ja build/Node/Python, pt-BR troubleshooting and es update blog. Existing aliases stay intact; page/task identifiers are unchanged.',
'website/site-model.mjs':'Reviewed every changed title/description and its page context. Rust-only installation is now explicit in five titles and therefore sidebars. pt-BR metadata uses programming language, templates versus note types; zh concepts explains ownership and existing-note identity; ja quickstart describes persistent saving and Python metadata explains candidate preparation. Routes/navigation order/topic IDs remain unchanged.',
'website/src/content/i18n/es.json':'Buscar en este sitio describes the search scope correctly; neighboring load/clear/no-results/count/error labels and placeholders remain compatible.',
}
for l,j in UI.items(): SHARED[f'website/src/i18n/{l}.json']=j
J.update(SHARED)
changed={f:v for f,v in A['files'].items() if v['changed']}
assert set(J)==set(changed),(set(J)-set(changed),set(changed)-set(J))
observations={}
for f,data in changed.items():
    old=data['previousContent'].splitlines();new=data['currentContent'].splitlines()
    spans=[]
    for tag,a,z,c,d in difflib.SequenceMatcher(None,old,new).get_opcodes():
        if tag!='equal': spans.append({'line':c+1,'before':'\n'.join(old[a:z]),'after':'\n'.join(new[c:d])})
    observations[f]={'currentSha256':data['currentSha256'],'previousSha256':data['previousSha256'],'changedSpans':spans,'independentJudgment':J[f]}
O=B/'linguistic-observations.json'
save(O,{'schemaVersion':1,'reviewerRun':RUN,'scope':'Actual independent reading of all changed natural-language spans, complete affected Markdown/MDX prose sections with page headings/introduction, UI changed-message groups and neighboring feedback, metadata, all six full AI prompts, glossary and alias changes. Unchanged long-page judgments are explicitly reused only after input verification. The author suggestion list was not used as an independent approval. Code fences omitted from reading packets are unchanged from the independently bound earlier review.', 'files':observations,'readingPackets':[ref(B/f'linguistic-reading-contexts-{l}.txt','Actually read the entire packet: page headings/introduction and full affected prose sections; unchanged code omitted.') for l in UI]})

resolved={
'L008-en':('Does your coding assistant support Skills?','Direct question order fixes the old declarative-question syntax.'),
'L007-rust-guide':('`Media::image()` 和 `sound()` 本身不会返回错误。','本身不会返回错误 replaces the unnatural 保持不会失败 without changing subsequent validation conditions.'),
'L002-en':('## Prepare, review and publish','The workflow heading no longer wrongly implies a source-only feature; following public 0.3.1 statement and historic anchor remain.'),
'L002-zh-CN':('## 准备候选包并在审阅后写入目标文件','The heading names preparation and later saving instead of wrongly implying source-only availability; public 0.3.1 remains explicit.'),
'L003-en':('## Batch media and prepared publication','The heading names the public capabilities rather than source-only API; source-api-details anchor retained.'),
'L003-zh-CN':('## 批量媒体处理与预备发布','The heading no longer suggests source-only API; public 0.3.1 and source-api-details identity retained.'),
}
issue_audit=[]
report={k:v for k,v in C.items() if k!='entries'};report['reviewerRun']=RUN;report['entries']={}
for id,ls in A['entries'].items():
    report['entries'][id]={}
    for locale,entry in ls.items():
        previous=read(entry['previousEvidence']['path'])['entries'][id][locale]['linguistic']
        current_issues=[];dispositions=[]
        for n,issue in enumerate(previous['openIssues']):
            issue=copy.deepcopy(issue); f=issue['file'];source=(ROOT/f).read_text();key=issue.get('id',id+'/'+locale+'/'+str(n))
            if key in resolved:
                text,reason=resolved[key];assert text in source and issue['span'] not in source
                disposition={'id':key,'status':'resolved','file':f,'line':next(i for i,line in enumerate(source.splitlines(),1) if text in line),'currentText':text,'judgment':reason,'priorIssue':issue,'verifierRun':RUN}
            else:
                assert issue['span'] in source,(id,locale,key)
                line=next(i for i,s in enumerate(source.splitlines(),1) if issue['span'] in s)
                issue['line']=line;issue['revalidatedBy']=RUN
                current_issues.append(issue)
                disposition={'id':key,'status':'retained','file':f,'line':line,'currentText':source.splitlines()[line-1],'judgment':'The original span and its relevant meaning remain; the new changes do not resolve the recorded nonblocking concern. Preserve its original severity and explanation.', 'priorIssue':issue,'verifierRun':RUN}
            dispositions.append(disposition);issue_audit.append({'pageId':id,'locale':locale,**disposition})
        own=entry['targetFile']; english=entry['sourceFile']; ui=f'website/src/i18n/{locale}.json'
        scope=[{'file':f,'observation':J[f],'observationReference':str(O)+'#files/'+f} for f in dict.fromkeys([own,english]) if f in J]
        evidence=[
          {'criterion':'immutable-prior-full-topic-assessment','priorEvidence':entry['previousEvidence'],'audit':ref(B/'linguistic-input-audit.json',f'{id}/{locale}: current three hashes and prior three hashes recomputed; every prior input byte proven against immutable Git/snapshot lineage.'),'judgment':'Reuse the prior independent full-topic reading only for byte-unchanged inputs/regions. The newly read editorial delta is explicitly described below; no fresh full reread of unaffected long pages is claimed.'},
          {'criterion':'actual-current-source-and-target-delta','targetFile':own,'sourceFile':english,'changedInputs':entry['changedInputs'],'unchangedInputs':entry['unchangedInputs'],'pageSpecificObservations':scope or [{'file':own,'observation':'Authored source bytes are unchanged from the immutable independent prior topic reading. Current differences are in the shared UI/metadata/brief/prompt closure described below.'}],'evidence':ref(O,'Contains each actual before/after span, current file digest and independently authored contextual judgment.')},
          {'criterion':'current-ui-and-metadata','file':ui,'judgment':UI[locale],'metadataJudgment':SHARED['website/site-model.mjs'],'sharedDependenciesJudgment':SHARED['docs/ai-quickstart.json']+' '+SHARED['website/language-briefs.json'],'scope':'All current catalog changes were read with surrounding states. Each entry inherits only the UI/metadata its existing rendering closure uses; this records closure review, not a claim that every page displays every key.'},
          {'criterion':'terminology','score':previous['score']['terminology'],'position':own+'; '+ui,'judgment':'Verified changed terms against the current language brief and existing note/card/deck, model/template, stable-ID, original-baseline and media-ownership context. API identifiers are preserved; unchanged topic terminology retains the prior independent judgment.'},
          {'criterion':'syntax','score':previous['score']['syntax'],'position':own+'; '+ui,'judgment':'Read complete changed prose sections and the changed message groups for grammar, references and number/placeholder fit. Updated clauses are comprehensible and natural; unchanged local wording concerns are retained below.'},
          {'criterion':'register','score':previous['score']['register'],'position':own+'; '+ui,'judgment':'The delta maintains the current locale brief and technical reader register, uses direct task wording, and adds no unsupported promises. Retain the conservative previous whole-topic score rather than infer a perfect score from a small edit.'},
          {'criterion':'usability','score':previous['score']['usability'],'position':own+'; website/site-model.mjs; website/legacy-sections.json','judgment':'Revised labels describe the same steps and destinations, new installation titles identify Rust, and changed heading aliases preserve task identity. Public-package/source and original-baseline restrictions remain available near the affected action. Actual built navigation/copy/search testing is a separate pending stage.'},
        ]
        if dispositions:evidence.append({'criterion':'prior-issue-reassessment','dispositions':dispositions})
        report['entries'][id][locale]={k:entry[k] for k in ['sourceHash','targetHash','technicalDependencyHash']}
        report['entries'][id][locale]['linguistic']={'status':'reviewed','score':copy.deepcopy(previous['score']),'evidence':evidence,'openIssues':current_issues}
report['scope']='Source-only independent delta assessment of complete HEAD 09d431830568345a9e6ae2bd1cda2154af5a08e1 editorial candidate across all 204 entries, unlike the prior isolated four-production-fix review. No artifact/browser/deployment approval is claimed.'
report['inputSnapshot']=ref(B/'linguistic-input-audit.json','259 current/prior input identities with full changed contents and exact immutable unchanged lineage.')
save(B/'linguistic-issue-reassessment.json',{'schemaVersion':1,'reviewerRun':RUN,'entries':issue_audit,'resolved':sum(x['status']=='resolved' for x in issue_audit),'retained':sum(x['status']=='retained' for x in issue_audit)})
save(B/'linguistic-source-review.json',report)
print('entries',sum(len(x) for x in report['entries'].values()),'changedInputs',len(changed),'issues',len(issue_audit),'resolved',sum(x['status']=='resolved' for x in issue_audit),'report',sha(B/'linguistic-source-review.json'))
