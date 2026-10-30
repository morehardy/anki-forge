import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { rankSearchDocuments, plainSearchText, documentRole } from '../src/lib/search.mjs';

const vocabulary = JSON.parse(await readFile(new URL('../search-concepts.json', import.meta.url), 'utf8')).locales;
const doc = (id, title, terms = [], description = '') => ({ id, title, terms, description, type: 'docs', url: `/preview/en/docs/${id}/` });
const documents = [
  doc('python-quickstart', 'Generate a deck with Python', ['Python quick start', 'first Python APKG export']),
  doc('python-api', 'Python API reference', ['Python SDK API', 'typed media', 'Cloze reference']),
  doc('python-diagnostics', 'Python diagnostics', ['Python errors', 'missing Cloze cards']),
  doc('node-quickstart', 'Generate a deck with Node.js', ['JavaScript quick start', 'TypeScript getting started']),
  doc('node-api', 'Node.js API reference', ['TypeScript declarations']),
  doc('anki-forge-vs-genanki', 'Anki Forge vs genanki', ['genanki alternative', 'Python library comparison', 'migration cost']),
  doc('genanki-migration', 'Migrate from genanki', ['genanki migration', 'GUID boundary']),
  doc('updates', 'Update a publication', ['original distribution APKG', 'baseline', 'Anki re-export', 'stable ID']),
  doc('cards', 'Basic and Cloze cards', ['Cloze deletion', 'fill-in-the-blank']),
  doc('media', 'Images and sound', ['audio attachments', 'image files']),
  doc('image-occlusion', 'Image Occlusion', ['rectangle masks', 'hide labels on images']),
];
const rank = (query, docs = documents, extra = {}) => rankSearchDocuments(query, docs, 'en', { concepts: vocabulary.en, ...extra });

test('SDK task roles distinguish a first project, reference and diagnosis', () => {
  assert.equal(rank('Python beginner tutorial')[0].id, 'python-quickstart');
  assert.equal(rank('Python methods reference')[0].id, 'python-api');
  assert.equal(rank('Python SDK')[0].id, 'python-quickstart');
  assert.equal(rank('Python Cloze errors')[0].id, 'python-diagnostics');
  assert.equal(rank('TypeScript quick start')[0].id, 'node-quickstart');
});

test('independent concepts preserve comparison, migration and baseline tasks', () => {
  assert.equal(rank('genanki alternative')[0].id, 'anki-forge-vs-genanki');
  assert.equal(rank('migrating from genanki')[0].id, 'genanki-migration');
  assert.equal(rank('reexport previous package')[0].id, 'updates');
  assert.equal(rank('picture masking')[0].id, 'image-occlusion');
  assert.equal(rank('voice attachments')[0].id, 'media');
});

test('unknown qualifiers do not AND away relevant words; nonsense and digits cannot retrieve', () => {
  assert.equal(rank('serendipitous Python beginner')[0].id, 'python-quickstart');
  assert.deepEqual(rank('qzxqv 18492'), []);
  assert.deepEqual(rank('18492', documents, { pagefindResults: [{ url: documents[0].url, content: 'Python 18492', excerpt: '18492' }] }), []);
  assert.deepEqual(rank('qzxqv', documents, { pagefindResults: [{ url: documents[0].url, content: 'Python reference' }] }), []);
});

test('Pagefind supplies exact body matches and plain snippets only for catalog URLs', () => {
  const results = rank('surrogateescape', documents, { pagefindResults: [
    { url: `${documents[2].url}#native-paths`, content: 'surrogateescape native paths', excerpt: '<mark>surrogateescape</mark> &amp; paths' },
    { url: '/preview/zh-cn/docs/foreign/', content: 'surrogateescape' },
  ] });
  assert.equal(results.length, 1);
  assert.equal(results[0].url, documents[2].url);
  assert.equal(results[0].excerpt, 'surrogateescape & paths');
  assert.ok(results[0].score > 0);
  assert.deepEqual(results[0].matchedTerms, ['surrogateescape']);
  assert.equal(plainSearchText('<mark>safe</mark> &#x4e2d;'), 'safe 中');
});

test('Chinese compounds and semantic aliases work without whitespace', () => {
  const zh = [
    doc('cards', 'Basic 与 Cloze 卡片', ['填空提示语法', '挖空卡片']),
    doc('image-occlusion', '图片遮挡', ['矩形遮罩', '遮住图片标签']),
    doc('media', '图片与音频', ['给卡片加图片', '打包声音文件']),
    doc('updates', '更新已分发牌组', ['原始分发 APKG', 'Anki 重新导出基线限制']),
    doc('python-quickstart', '用 Python 制作记忆卡片', ['Python 入门', 'pip install ankiforge']),
    doc('python-api', 'Python API 参考', ['Python SDK 参考']),
  ];
  const search = query => rankSearchDocuments(query, zh, 'zh-CN', { concepts: vocabulary['zh-CN'] });
  assert.equal(search('想在图像上遮住标签')[0].id, 'image-occlusion');
  assert.equal(search('完形填空卡片')[0].id, 'cards');
  assert.equal(search('给卡片配图和发音')[0].id, 'media');
  assert.equal(search('重新导出的旧包能做基线吗')[0].id, 'updates');
  assert.equal(search('Python快速上手')[0].id, 'python-quickstart');
  assert.deepEqual(search('毫无意义的量子蔬菜999999'), []);
});

test('a new locale supplies vocabulary as data without changing the engine', () => {
  const concepts = { __language: { stopWords: ['comment'] }, audio: { terms: ['audio', 'son', 'prononciation'] } };
  const docs = [doc('media', 'Fichiers audio'), doc('cards', 'Cartes de texte')];
  assert.equal(rankSearchDocuments('comment ajouter la prononciation', docs, 'fr', { concepts })[0].id, 'media');
});

test('English plural forms share lexical evidence without fuzzy substring matches', () => {
  assert.equal(rank('snapshots', [doc('a', 'Immutable snapshot')])[0].id, 'a');
  assert.equal(rank('creating', [doc('a', 'Create content')])[0].id, 'a');
  assert.equal(rank('published', [doc('a', 'Publish content')])[0].id, 'a');
  assert.deepEqual(rank('cat', [doc('a', 'Publication')]), []);
});

test('definitions, grouping and package structure outrank broad publishing recipes', () => {
  const docs = [
    doc('concepts', 'Core concepts', ['note versus card versus deck', 'Project and deck relationship', 'typed content and media dependencies', 'report versus artifact']),
    doc('json-to-anki', 'JSON notes and cards into Anki decks', ['batch note creation', 'APKG export', 'grouping data', 'package contents']),
    doc('csv-to-anki', 'Spreadsheet notes into cards and decks', ['CSV flashcard generator', 'APKG package export']),
    doc('ai-anki-cards', 'Generate study cards from notes', ['AI answers', 'deck generation']),
    doc('blog:your-first-anki-deck', 'Your first deck of cards and notes', ['vocabulary notes', 'APKG package contents']),
  ];
  for (const query of ['definitions of notes and cards', 'what is the relationship of notes to a deck', 'how are notes grouped into decks', 'what components are inside an exported APKG package']) {
    assert.equal(rank(query, docs)[0].id, 'concepts', query);
  }
  // Recipe-specific evidence still selects the corresponding workflow.
  assert.equal(rank('CSV export spreadsheet', docs)[0].id, 'csv-to-anki');
});

test('unspecified SDK failures lead to generic troubleshooting, named SDK stays specific', () => {
  const docs = [
    doc('troubleshooting', 'Troubleshooting', ['note key conflicts', 'native package load failure', 'missing Cloze cards']),
    doc('python-diagnostics', 'Python generation errors and diagnostics', ['Python errors', 'BuildError snapshot', 'failed note generation']),
    doc('docs', 'Generate notes, handle errors and publish decks', ['Anki deck generation library', 'structured diagnostics']),
    doc('json-to-anki', 'Generate notes from JSON', ['generation errors', 'failed batch note creation']),
  ];
  assert.equal(rank('note generation failed', docs)[0].id, 'troubleshooting');
  assert.equal(rank('Python generation failed', docs)[0].id, 'python-diagnostics');
  assert.equal(rank('BuildError snapshot', docs)[0].id, 'python-diagnostics');
});

test('learning review does not activate publication history without update context', () => {
  const docs = [
    doc('image-occlusion', 'Image Occlusion', ['rectangle masks', 'hide diagram labels', 'image masking flashcards']),
    doc('updates', 'Update a distributed deck', ['review history', 'scheduling', 'baseline', 'publication review']),
    doc('ai-anki-cards', 'Review AI study questions', ['learning', 'question and answer cards']),
  ];
  assert.equal(rank('masking diagram questions for study and review', docs)[0].id, 'image-occlusion');
  const review = rank('review history', docs);
  assert.ok(review.every(result => !result.matchedTerms.includes('update')));
  assert.equal(rank('update while preserving review history', docs)[0].id, 'updates');
  const zh = [
    doc('concepts', '核心概念', ['笔记、卡片与牌组的区别', 'Project 与牌组的关系']),
    doc('json-to-anki', '从 JSON 生成笔记和卡片', ['牌组导出', '批量笔记创建']),
    doc('image-occlusion', '图片遮挡', ['遮罩', '图表区域制卡']),
    doc('updates', '更新已分发牌组', ['复习历史', '复习进度', '更新基线']),
  ];
  const search = query => rankSearchDocuments(query, zh, 'zh-CN', { concepts: vocabulary['zh-CN'] });
  assert.equal(search('笔记与卡片的关系和分组')[0].id, 'concepts');
  assert.equal(search('笔记与卡片有什么区别')[0].id, 'concepts');
  assert.equal(search('遮罩图表做学习复习题')[0].id, 'image-occlusion');
  assert.equal(search('更新时保留复习进度')[0].id, 'updates');
  assert.deepEqual(rank('inside qzxvzx', [doc('concepts', 'Core concepts')]), []);
});

test('composed Chinese customization, cardinality and diagram actions choose their domain', () => {
  const docs = [
    doc('custom-notetypes', '自定义笔记类型', ['自定义 Anki 模型', '定义笔记字段', '自定义卡片模板', '按条件生成卡片']),
    doc('cards', 'Basic 与 Cloze 卡片', ['正反面记忆卡片', '两条笔记三张卡片', '填空提示语法']),
    doc('concepts', '核心概念', ['笔记、卡片与牌组的区别', '字段键与模板键', 'Project 与牌组的关系']),
    doc('image-occlusion', '图片遮挡', ['矩形遮罩', '图表区域制卡', '遮住图片标签']),
    doc('media', '图片与音频', ['给记忆卡片加图片', '给卡片加音频']),
    doc('updates', '更新已分发牌组', ['复习历史', '复习进度', '更新基线']),
    doc('python-quickstart', '用 Python 制作记忆卡片', ['Python 入门', '首次导出 APKG']),
    doc('ai-anki-cards', 'AI 辅助制卡', ['笔记转 Anki', '自动制卡流程', '核对 AI 答案']),
  ];
  const search = query => rankSearchDocuments(query, docs, 'zh-CN', { concepts: vocabulary['zh-CN'] });
  for (const query of ['想定制卡面并额外增加词性字段', '自己设计正面和背面的布局', '怎样新增笔记字段']) {
    assert.equal(search(query)[0].id, 'custom-notetypes', query);
  }
  for (const query of ['单条笔记与多张卡片的对应方式', '笔记和卡片的数量对应是什么', '一则Note可以得到多张Cards吗']) {
    assert.equal(search(query)[0].id, 'concepts', query);
  }
  for (const query of ['把示意图各部位的名称盖住做复习练习', '隐藏插图中的标注考自己', '截图上的名字蒙住之后再答题']) {
    assert.equal(search(query)[0].id, 'image-occlusion', query);
  }
  assert.equal(search('字段值里加图片')[0].id, 'media');
  assert.equal(search('给卡片加多个图片')[0].id, 'media');
  assert.ok(search('隐藏答案').every(result => !result.matchedTerms.includes('遮盖')));
});

test('compositional context is language data and does not mistake plural media for note cardinality', () => {
  assert.equal(rank('cover a diagram for practice')[0].id, 'image-occlusion');
  assert.equal(rank('how many cards can one note produce', [
    doc('concepts', 'Core concepts', ['note and card relationship']),
    doc('cards', 'Basic cards', ['one note creates a Basic card']),
  ])[0].id, 'concepts');
  assert.equal(rank('many pictures')[0].id, 'media');
});

test('real HTML titles recover normal-page navigation without overtaking SDK tasks', () => {
  const hubs = [
    { ...doc('home', 'Make learning material programmable', [], 'Rust, Python and Node.js packages for Anki'), type: 'home', pageTitle: 'Anki Forge homepage', url: '/preview/en/' },
    { ...doc('examples', 'See what you can build', [], 'Python, Rust and Node.js deck authoring'), type: 'examples', pageTitle: 'Examples | Anki Forge' },
    { ...doc('blog', 'Notes on making notes'), type: 'blog-list', pageTitle: 'Blog | Anki Forge' },
  ];
  const all = [...documents, ...hubs];
  assert.equal(rank('homepage', all)[0].id, 'home');
  assert.equal(rank('examples', all)[0].id, 'examples');
  assert.equal(rank('blog', all)[0].id, 'blog');
  assert.equal(rank('Anki Python quick start', all)[0].id, 'python-quickstart');
  assert.equal(rank('Anki TypeScript API', all)[0].id, 'node-api');
  assert.deepEqual(hubs.map(documentRole), ['overview', 'examples', 'collection']);
  assert.equal(rank('homepage', all)[0].title, 'Make learning material programmable');
});
