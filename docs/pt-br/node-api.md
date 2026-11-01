# API de Node e TypeScript

O ponto de entrada `ankiforge` expõe um único modelo de criação de conteúdo, Project. Os tipos são definidos no [código público](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/index.ts), nas [classes de relatório](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/report.ts) e nas [declarações dos snapshots JSON](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/snapshots.ts). Comece pelo [guia de início rápido](/docs/node-quickstart/).

<!-- section: project-and-notes -->
## Projeto e notas

| API | Comportamento |
| --- | --- |
| `new Project(namespace)` | Validar um namespace estável de publicação |
| `.defaultDeck(name)` | Definir o destino padrão; retorna o projeto |
| `.add(key, note)` | Incorporar atomicamente nota/modelo/mídia; chaves duplicadas causam falha |
| `.addAsset(media)` | Incluir um recurso explícito de HTML/CSS/script bruto |
| `.clone()` | Projeto editável independente que compartilha valores imutáveis |
| `.length` | Quantidade de notas registradas |
| `Note.basic(front, back)` / `Note.cloze(text)` | Nota imutável que mantém seu modelo integrado |
| `model.note().field(key, content)` | Nota personalizada imutável que mantém seu modelo |
| `note.deck(name).tag(tag).tags(iterable)` | Retornar uma nova nota configurada |

Strings sempre se tornam Text, inclusive strings Cloze. A marcação explícita usa `Content.html`. `Content.sequence(iterable)` combina texto, HTML e mídia tipada. Use chaves explícitas do seu domínio; a identidade não é derivada do conteúdo nem dos nomes de exibição.

<!-- section: immutable-models -->
## Modelos imutáveis

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

Builders retornam novos valores. Um modelo finalizado não tem métodos para modificar campos/templates. `builder.clozeField(key)` seleciona a semântica Cloze personalizada; `builder.css` e `builder.asset` fornecem estilos e dependências explícitas. Templates também aceitam `name`, `browserFront`, `browserBack` e `targetDeck`. As referências dos templates e as condições de geração usam chaves de campo; os nomes de exibição são compilados para o Anki. `GenerationRule.ankiDefault()` seleciona o comportamento padrão do Anki; `all` e `any` selecionam condições explícitas.

`await NoteType.fromBundle(path, mediaLimits?)` retorna o mesmo modelo imutável a partir de `template-bundle-v2`. Após o sucesso, os textos/recursos importados não dependem mais do diretório de origem. Modelos podem ser reutilizados entre projetos; definições incompatíveis sob a mesma chave de modelo causam uma falha atômica.

<!-- section: media-and-image-occlusion -->
## Mídia e oclusão de imagem

`await Media.file(path, limits?)` e `await Media.bytes(Uint8Array, mime, limits?)` retornam snapshots com dados próprios. Arquivos de origem podem mudar ou desaparecer após o await terminar com sucesso. `withExportName(name)` retorna um valor renomeado sem alterar clones ou conteúdo anteriores. Leia `filename`, `mediaType` e `byteLength` como propriedades.

`media.image()` e `.sound()` retornam Content tipado que incorpora seu recurso automaticamente a um projeto. `project.addAsset` explícito e `builder.asset` do modelo cobrem referências brutas em HTML, CSS e scripts. Os limites de mídia aceitam `{ maxBytes }`; o padrão é 256 MiB e bytes nativos não têm a restrição de 64 KiB para dados inline.

```js
import { Media, Note, Mask } from 'ankiforge';
const image = await Media.file('diagram.png');
const note = Note.imageOcclusion(image)
  .mask(Mask.rect('nucleus', 10, 10, 20, 20))
  .mask(Mask.rect('wall', 50, 50, 20, 20))
  .mode('hide_one_guess_one')
  .build().field('header', 'Cell');
```

O modo padrão é `hide_all_guess_one`; ambos os modos geram cartões separados para máscaras com chave. `.build()` valida as dimensões decodificadas, os retângulos finitos e as chaves únicas. `image` e `occlusion` são campos gerados reservados. Veja [oclusão de imagem](/docs/image-occlusion/) para os limites de decodificação e atualização.

<!-- section: build-compare-and-output -->
## Geração, comparação e saída

| API | Resultado |
| --- | --- |
| `await project.build(BuildOptions.to(path))` | BuildOutput que mantém um artefato persistente |
| `await project.build(BuildOptions.temporary())` | BuildOutput que mantém um artefato temporário |
| `await project.compare(CompareOptions.against(path))` | ComparisonReport concluído, inclusive quando a política bloqueia a publicação |
| `options.updateFrom(path)` | Configurar a atualização a partir das evidências completas da distribuição original |
| `options.inspectLimits(limits)` | Substituir os limites independentes de inspeção |
| `options.updatePolicy(policy)` | Configurar a política de risco de atualização/comparação |

Uma saída bem-sucedida oferece `.artifact`, `.report` e `.snapshot()`. O `BuildReport` oferece `.counts`, `.baselineCounts`, `.durationMs`, `.diagnostics`, `.comparison` e `.snapshot()`; ele não informa o resultado da operação nem mantém arquivos. `.comparison` é um `ComparisonReport` ou `null`, usando a mesma classe de `project.compare()`. Comparações oferecem `.findings`, `.highestRisk`, `.policy`, `.diagnostics` e `.snapshot()`. Os nomes dos campos de snapshot refletem o JSON de Rust, incluindo `allows_publication` e `schema_version`; não são cópias em camelCase do relatório nativo. Use `report.comparison.highestRisk` em um objeto de comparação ou `report.snapshot().comparison.highest_risk` em seu snapshot JSON, depois de verificar se não é `null`.

`new UpdatePolicy()` bloqueia High/Critical. `.failOn('medium')` altera o limiar. `.allow('RISK.NOTE_REMOVED')` autoriza explicitamente toda essa categoria, mantendo suas evidências e gravidade. Códigos desconhecidos e erros impeditivos não podem ser autorizados. Definir uma política ao gerar um pacote sem base de atualização é um erro. Use a mesma política e os mesmos limites em compare e build quando precisar de decisões de publicação correspondentes.

As opções de inspeção incluem `maxArchiveBytes`, `maxEntries`, `maxCentralDirectoryBytes`, `maxZipEntryBytes`, `maxZipTotalBytes`, `maxMetaBytes`, `maxMediaMapBytes`, `maxIdentityBytes`, `maxCollectionBytes`, `maxMediaBytes`, `maxDecodedTotalBytes` e `maxZstdWindowBytes`. Os valores são inteiros seguros não negativos ou `bigint` até o máximo de u64; zero é um limite realmente igual a zero. Os contadores se aplicam de forma independente à base e ao candidato. Opções desconhecidas causam falha.

<!-- section: artifact-lifetime-and-concurrency -->
## Vida útil dos artefatos e concorrência

Caminhos dos snapshots JSON usam o tipo exportado `PathSnapshot`. Caminhos Unicode são strings; caminhos nativos não Unicode usam `{ encoding: "unix_bytes", bytes: number[] }` ou `{ encoding: "windows_wide", units: number[] }`. Os arrays preservam exatamente os bytes Unix ou as unidades de código UTF-16 do Windows. Isso inclui caminhos de artefatos em snapshots de geração, caminhos de publicação em falhas e detalhes estruturados de caminhos de erro. Decodifique um caminho codificado somente para sua plataforma original; no Unix, `Buffer.from(value.bytes)` preserva um caminho em bytes para as funções de sistema de arquivos de Node. Os métodos de criação de conteúdo de Node continuam aceitando caminhos como strings.

Caminhos relativos de mídia e pacotes são fixados quando a leitura é chamada; destinos de geração e bases de atualização são fixados quando as opções são construídas. Em POSIX, links simbólicos seguidos de `..` preservam a semântica de navegação do sistema de arquivos. `persistTo` resolve destinos relativos em relação ao diretório de trabalho capturado pela geração original, inclusive em handles clonados e persistidos. Mudanças posteriores no diretório de trabalho não redirecionam esses caminhos.

Mantenha um proprietário do artefato vivo enquanto consome `.path`. `.clone()` cria um proprietário independente; `await .close()` libera esse proprietário. O último proprietário temporário exclui o arquivo. `await .persistTo(path)` retorna um artefato persistente e mantém o original utilizável mesmo em caso de falha. Streams de arquivo comuns de Node podem ler um artefato enquanto seu proprietário é mantido.

Snapshots são dados JSON congelados e não mantêm arquivos nativos. Um arquivo persistente permanece após a liberação dos proprietários. `BuildOutput` não pode ser construído pelo chamador; ele é criado somente por gerações bem-sucedidas.

Build e compare executam de forma assíncrona sobre um snapshot próprio do projeto, capturado na chamada. Edições posteriores não alteram essa solicitação. Leituras de mídia e carregamentos de pacotes também executam em workers. ESM e CJS de Node compartilham o mesmo módulo nativo e as mesmas identidades de classe.

<!-- section: errors -->
## Erros

`SchemaError`, `AddError`, `MediaError`, `TemplateBundleError`, `ImageOcclusionError`, `PolicyError`, `ConfigurationError`, `CompareError`, `BuildError` e `PersistError` estendem `ForgeError`. Eles oferecem `kind`, `code`, `domain`, o texto da cadeia de causas nativas em `causes`, `sourceDetails` estruturado para causas reconhecidas de E/S, schema, mídia e limites, e `details` da operação. A exceção nativa original é preservada como `cause` do JavaScript.

`BuildError.snapshot()` preserva a falha e os fatos da publicação; `BuildError.report` e `CompareError.report` retornam `BuildReport`, assim como `BuildOutput.report`. Use `.baselineCounts` e `.comparison` nesses relatórios; use `.snapshot()` para os campos JSON em snake_case. Uma comparação que falha pode ter observações parciais e uma comparação `null`. `PersistError.publication` registra se o destino já foi publicado e se a durabilidade foi confirmada. Erros de mídia/inspeção preservam os detalhes dos limites. Use esses campos para processamento, em vez de analisar mensagens de erro. `ArtifactClosedError` informa o uso de um proprietário fechado; `NativeLoadError` informa pacotes nativos ausentes, incompatíveis ou desatualizados.

<!-- section: addition-context -->
## Contexto da adição

Adições que falham deixam o projeto inalterado. O contexto estruturado registra as chaves originais da nota/modelo e o alvo, com detalhes opcionais de conflito ou uso de mídia. Rust oferece `note::{AddContext, AddTarget, AddDetail}` por meio de `AddError::context()` / `detail()`; os SDKs oferecem `AddError.details` com contexto/detalhes tipados. Os campos de Node usam camelCase e são profundamente congelados; Python usa TypedDicts em snake_case. Tags preservam seu índice de inserção; localizações de campo preservam os índices originais da sequência (null/None para o campo, [] para sua raiz) e intervalos opcionais de bytes UTF-8. Erros de baralho padrão na geração não têm chaves de nota/modelo e continuam disponíveis na cadeia de causas.

Imagens tipadas exigem MIME image/* na adição; referências de som aceitam audio/* ou video/*. Outras categorias geram `NOTE.MEDIA_USAGE_INVALID`. Os construtores continuam sem possibilidade de falha; recursos explícitos e HTML bruto não são afetados. A verificação usa o MIME preservado na importação e não certifica a reprodução. A validação independente de MIME/extensão durante a geração continua valendo. Veja o [documento de projeto](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md) para a tabela completa de alvos/detalhes.

<a id="source-api-details"></a>

<!-- section: batch-media-and-prepared-publication -->
## Mídia em lote e publicação preparada

A importação de mídia em lote e a publicação preparada estão incluídas no pacote público `0.3.1`.

<!-- section: prepare-review-and-publish -->
### Prepare, revise e publique

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

`publish()` assume o proprietário nativo imediatamente e executa uma tarefa em um worker. Toda tentativa o consome, inclusive rejeições de política e E/S. Chamadas repetidas são rejeitadas com `PreparedPublicationStateError`, código `BUILD.PREPARED_UNAVAILABLE` e `reason` igual a `consumed` ou `closed`. Fechar após o início de publish não o cancela. Relatórios permanecem legíveis após close. As opções são fixadas na chamada de preparação; mudar a política ou o destino exige outra preparação. A duração não inclui o tempo de espera pela revisão. Nem relatórios nem JSON podem ser convertidos de volta em um proprietário.

Build, compare, preparação e clones compartilham versões imutáveis do Project nativo. Alterações na criação de conteúdo continuam síncronas e isoladas. A primeira alteração enquanto outro snapshot está vivo pode copiar o Project; alterações exclusivas posteriores não fazem essa cópia. Isso desloca a cópia para fora da submissão da Promise e não promete primeiras edições mais rápidas, menor atraso máximo de timers ou RSS mais baixo. Entradas de bytes do JavaScript continuam sendo copiadas antes da submissão ao worker. Use binários de protocolo 6 com esses wrappers.

<!-- section: typescript -->
## TypeScript

Instale `typescript` e `@types/node` na sua aplicação. Use `main.mts` ou `"type": "module"` e este `tsconfig.json`:

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

Compile com `npx tsc` e execute `node dist/main.mjs` para um ponto de entrada `.mts`.
