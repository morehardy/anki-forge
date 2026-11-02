# API de Python

Canonical page: https://ankiforge.dev/pt-br/docs/python-api/
Language: pt-BR
Description: Project, mídia, opções de geração, relatórios e controle da vida útil dos artefatos temporários em Python.

O SDK chama a API pública padrão de Rust por meio de uma extensão nativa. Ele inclui `py.typed` e assinaturas públicas verificadas. Comece pelo [guia de início rápido](https://ankiforge.dev/pt-br/docs/python-quickstart/).

## Valores de projeto e nota

| Entrada | Comportamento |
| --- | --- |
| `Project(namespace, *, default_deck=None)` | Valida o namespace estável explícito e configura o baralho padrão |
| `project.add(key, note)` | Incorpora atomicamente a nota, o modelo e a mídia com dados próprios; chaves de nota duplicadas causam falha |
| `project.add_asset(media)` | Inclui uma dependência explícita de HTML/CSS/script bruto |
| `len(project)` | Quantidade de notas adicionadas com sucesso |
| `Note.basic(front, back)` | Modelo Basic integrado; strings representam Text |
| `Note.cloze(text)` | Modelo Cloze integrado; strings representam Text e preservam a sintaxe Cloze |
| `note.field(key, content)` | Retorna uma nova nota com a atribuição do campo; a chave é validada na adição |
| `note.deck(name)` / `.tag(tag)` / `.tags(tags)` | Retorna uma nota configurada |
| `note.note_type` | O modelo imutável mantido pela nota |

Strings comuns se tornam Text. `Content.text(value)` e `Content.html(value)` são construtores explícitos; `Content.sequence(iterable)` combina strings e conteúdo sem renderizar nem perder dependências de mídia. Um campo não aceita um nome de arquivo bruto como referência de mídia.

## Modelos e templates

`NoteType.builder(key)` retorna um builder imutável. Seus métodos `.name`, `.field`, `.template`, `.css`, `.cloze_field` e `.asset` retornam builders atualizados. `.build()` valida a definição completa e retorna um NoteType imutável. `model.note()` cria uma nota que mantém esse modelo. `NoteType.from_bundle(path, limits=MediaLimits())` carrega o mesmo modelo e seus recursos a partir do formato atual de pacote; o limite opcional é aplicado separadamente a cada recurso antes da leitura.

`Field(key, name=None, required=False, sort=False)` separa a chave estável do nome de exibição. `Template(key, front, back, name=None, browser_front=None, browser_back=None, target_deck=None, generation=GenerationRule())` usa chaves de campo em todas as strings de origem. `GenerationRule.all(keys)` e `.any(keys)` configuram a geração de cartões. O padrão infere a condição de presença de campo do Anki. `.cloze_field(key)` no builder declara uma geração Cloze personalizada.

## Mídia e oclusão

| Entrada | Comportamento |
| --- | --- |
| `Media.file(path, *, limits=MediaLimits())` | Lê imediatamente um snapshot com dados próprios |
| `Media.bytes(data, media_type, *, limits=MediaLimits())` | Mantém bytes com um tipo MIME explícito |
| `media.with_export_name(name)` | Retorna um novo valor de nome de arquivo que compartilha o snapshot; referências anteriores preservam seus nomes |
| `media.filename`, `media.media_type`, `len(media)` | Metadados do snapshot somente para leitura |
| `media.image()` / `.sound()` | Conteúdo tipado que mantém o snapshot |

`MediaLimits(max_bytes=256 << 20)` controla cada importação antes da criação do snapshot. Mídias grandes podem usar armazenamento temporário sob sua própria responsabilidade. Conflitos de nomes são verificados entre todos os recursos explícitos e incorporados automaticamente.

`Note.image_occlusion(media)` retorna um ImageOcclusionBuilder. Encadeie `.mask(Mask.rect(key, x, y, width, height))`, opcionalmente `.mode(OcclusionMode.HIDE_ONE_GUESS_ONE)` e, por fim, `.build()`. O modo padrão é `HIDE_ALL_GUESS_ONE`. A finalização decodifica a imagem e valida chaves estáveis de máscara e coordenadas finitas em pixels dentro da imagem. Na Note retornada, defina `header`, `back_extra` ou `comments` normalmente. Os campos gerados `image` e `occlusion` não podem ser atribuídos manualmente.

## Operações e resultados

| Entrada | Resultado |
| --- | --- |
| `project.build(BuildOptions.to(path))` | BuildOutput com um artefato persistente garantido |
| `project.build(BuildOptions.temporary())` | BuildOutput com um artefato temporário de vida útil controlada |
| `options.update_from(path)` | Retorna opções para atualizar a partir das evidências completas da base |
| `project.compare(CompareOptions.against(path))` | ComparisonReport concluído, incluindo uma decisão separada de política de publicação |
| `options.update_policy(policy)` | Aplica uma política de risco explícita a Update/Compare; inválida para Create |
| `options.inspect_limits(limits)` | Aplica limites finitos de forma independente à base e ao candidato |

BuildOutput tem `.artifact`, `.report` e `.snapshot()`. BuildReport oferece `.counts` (BuildCounts com notes/cards/media), `.diagnostics`, `.comparison` e `.snapshot()`. Um relatório não informa o resultado da operação nem mantém um proprietário de arquivo. ComparisonReport oferece `.findings`, `.policy`, `.allows_publication` e `.snapshot()`. Snapshots são dicionários comuns serializáveis em JSON e não mantêm a propriedade de artefatos.

ApkgArtifact oferece `.path`, `.persist_to(path)` e `.close()`. Copiá-lo com `copy.copy` mantém outro proprietário; um gerenciador de contexto libera esse proprietário na saída. Uma saída temporária desaparece quando seu último proprietário é liberado. Caminhos persistentes permanecem após a liberação dos handles. Uma falha na persistência mantém o handle de origem utilizável.

`UpdatePolicy()` bloqueia resultados High e Critical. `.fail_on(RiskLevel.MEDIUM)` altera o limiar. `.allow('RISK.NOTE_REMOVED')` autoriza explicitamente toda a categoria registrada, preservando as evidências. Códigos desconhecidos causam falha imediata. `InspectLimits` expõe todos os limites finitos de bytes/contagens, incluindo `max_identity_bytes`; não é possível desativar inspeções obrigatórias.

## Erros e concorrência

Falhas geram SchemaError, AddError, MediaError, ImageOcclusionError, TemplateBundleError, CompareError, PolicyError, BuildError ou PersistError. Cada um oferece `kind`, `code`, `details`, observações estruturadas em `source_details` e o texto da cadeia de causas em `causes`, além de preservar a exceção nativa como `__cause__`. Causas de E/S mantêm um OSError encadeado quando disponível. BuildError oferece `.report` e `.snapshot()`; CompareError oferece um `.report` parcial; os detalhes de PersistError incluem os fatos reais da publicação.

Operações nativas de Project liberam o GIL e rejeitam o uso simultâneo do mesmo objeto com `BINDING.PROJECT_BUSY`; projetos independentes podem executar concorrentemente. Após um fork do processo, recrie todos os valores e handles nativos. Handles herdados rejeitam operações com `BINDING.FORKED_OBJECT`, e sua liberação não remove arquivos pertencentes ao processo pai. Valores criados pelo processo filho funcionam e são liberados normalmente. Veja [diagnósticos](https://ankiforge.dev/pt-br/docs/python-diagnostics/).

## Contexto da adição

Adições que falham deixam o projeto inalterado. O contexto estruturado registra as chaves originais da nota/modelo e o alvo, com detalhes opcionais de conflito ou uso de mídia. Rust oferece `note::{AddContext, AddTarget, AddDetail}` por meio de `AddError::context()` / `detail()`; os SDKs oferecem `AddError.details` com contexto/detalhes tipados. Os campos de Node usam camelCase e são profundamente congelados; Python usa TypedDicts em snake\_case. Tags preservam seu índice de inserção; localizações de campo preservam os índices originais da sequência (null/None para o campo, \[\] para sua raiz) e intervalos opcionais de bytes UTF-8. Erros de baralho padrão na geração não têm chaves de nota/modelo e continuam disponíveis na cadeia de causas.

Imagens tipadas exigem MIME image/\* na adição; referências de som aceitam audio/\* ou video/\*. Outras categorias geram `NOTE.MEDIA_USAGE_INVALID`. Os construtores continuam sem possibilidade de falha; recursos explícitos e HTML bruto não são afetados. A verificação usa o MIME preservado na importação e não certifica a reprodução. A validação independente de MIME/extensão durante a geração continua valendo. Veja o [documento de projeto](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md) para a tabela completa de alvos/detalhes.

## Detalhes adicionais da API

A importação de mídia em lote e a publicação preparada estão incluídas no pacote público `0.3.1`.

### Prepare, revise e publique

```python
from ankiforge import Project, Note, BuildOptions, BuildError
project = Project('review-once').add('one', Note.basic('Question', 'Answer'))
previous = project.build(BuildOptions.to('previous.apkg'))
previous.artifact.close()
prepared = project.prepare_publication(
    BuildOptions.to('next.apkg').update_from('previous.apkg')
)
try:
    report = prepared.report  # observations; does not own the candidate
    if report.comparison is None or report.comparison.allows_publication:
        output = prepared.publish()
        output.artifact.close()  # persistent next.apkg remains
except BuildError as error:
    print(error.snapshot()['result'])  # actual publication/durability facts
    raise
finally:
    prepared.close()  # idempotent; deletes an unused candidate
```

Uma tentativa de publicação consome o proprietário, inclusive quando há falha de política ou E/S. Chamadas posteriores geram `PreparedPublicationStateError` com o código `BUILD.PREPARED_UNAVAILABLE` e o motivo `closed` ou `consumed`; close não cancela uma publicação já em andamento. Caminhos relativos são fixados na chamada de preparação, e alterações no Project não modificam o APKG revisado. Relatórios continuam disponíveis após close sem manter arquivos. Para alterar opções ou tentar novamente, faça outra preparação. A duração não inclui o tempo de espera pela revisão. Erros tardios podem significar que a substituição foi bem-sucedida, mas a durabilidade não foi confirmada; consulte os fatos da publicação antes de decidir o que fazer. São necessários o protocolo 1 do binding nativo e o pacote de contratos 2.1.0; binários mais antigos com o mesmo número de versão são rejeitados.
