# Diagnósticos e política de atualização em Python

Uma geração bem-sucedida retorna BuildOutput com um artefato garantido. Uma geração que falha levanta BuildError com observações e fatos da publicação. Diagnósticos, por si só, nunca determinam o sucesso.

```python
import json
from ankiforge import Project, Note, BuildOptions, BuildError

project = Project('course').add('term', Note.basic('Question', 'Answer'))
try:
    output = project.build(BuildOptions.to('course.apkg'))
except BuildError as error:
    print(error.kind, error.code)
    print(json.dumps(error.snapshot(), ensure_ascii=False))
else:
    print(output.artifact.path)
    print(json.dumps(output.report.snapshot(), ensure_ascii=False))
```

Use os campos `kind` e `code` para tomar decisões, sem analisar mensagens. Todas as exceções públicas de domínio preservam o texto da cadeia de causas em `causes` e encadeiam sua exceção nativa. `details` contém evidências específicas da operação: localizações na origem dos templates, caminhos de mídia e medições de limites, observações de comparação incompletas ou fatos da publicação. Uma causa de E/S se torna um OSError encadeado quando disponível. `source_details` preserva erros aninhados tipados: causas de limite de texto de pacotes incluem `limit` e `observed`, enquanto causas de mídia incluem `code`, `path` e `limit_exceeded`.

BuildReport contém somente observações. Seu snapshot tem contagens, diagnósticos, tempo decorrido e qualquer comparação concluída. Snapshots da saída e de BuildError contêm o resultado real Success/Failure. Snapshots de falha distinguem um destino nunca publicado de um destino publicado antes de uma falha de durabilidade. Caminhos JSON não mantêm arquivos nem prolongam a vida útil de artefatos temporários.

<!-- section: compare-before-publication -->
## Compare antes de publicar

```python
from ankiforge import CompareOptions, UpdatePolicy

comparison = project.compare(CompareOptions.against('course-v1.apkg'))
print(comparison.findings)
print(comparison.policy)
```

Uma comparação concluída retorna normalmente mesmo quando encontra risco High. Consulte `comparison.allows_publication` para ver a decisão da política. Uma base ilegível, evidências completas de identidade ausentes, um candidato inválido ou um limite excedido geram CompareError; seu `.report` contém as observações concluídas antes da falha.

A política padrão bloqueia resultados High e Critical. Depois de revisar uma categoria específica de risco, a autorização é explícita:

```python
policy = UpdatePolicy().allow('RISK.NOTE_REMOVED')
comparison = project.compare(CompareOptions.against('course-v1.apkg').update_policy(policy))
output = project.build(BuildOptions.to('course-v2.apkg')
    .update_from('course-v1.apkg').update_policy(policy))
```

A autorização abrange todos os resultados da categoria nomeada. Eles mantêm o risco original e suas evidências. Autorizações sem correspondência produzem avisos; avisos não transformam uma publicação bem-sucedida em falha. Códigos desconhecidos, evidências ausentes e outros erros impeditivos não podem ser autorizados. Não autorize automaticamente todas as categorias retornadas por uma comparação.

`update_from` exige o APKG original distribuído anteriormente, incluindo suas evidências completas incorporadas. Reexportações do Anki não preservam essas evidências. Notas/cartões omitidos de um APKG não são excluídos automaticamente da coleção do estudante. Revise mudanças estruturais considerando as configurações de importação do Anki pretendidas.

<!-- section: budgets-and-artifact-ownership -->
## Limites e propriedade dos artefatos

```python
from ankiforge import InspectLimits

limits = InspectLimits(max_archive_bytes=3 << 30)
options = BuildOptions.to('course-v2.apkg').update_from('course-v1.apkg').inspect_limits(limits)
```

Os limites se aplicam separadamente a cada base e candidato inspecionado. A importação de mídia tem um limite anterior próprio, MediaLimits. Falhas de limite preservam o recurso, o limite e os valores observados em `details['limit_exceeded']`; aumente deliberadamente o limite relevante e tente novamente. Para pacotes de templates, passe `NoteType.from_bundle(path, limits=MediaLimits(...))` para ajustar o limite de leitura de cada recurso; o manifesto e os textos de template mantêm seus limites fixos separados. Inspeções obrigatórias não podem ser desativadas.

Use `BuildOptions.temporary()` quando não quiser um caminho persistente. Mantenha a saída ou um handle de artefato enquanto consome o arquivo; `artifact.persist_to(path)` cria uma cópia persistente. Se a persistência falhar, os detalhes de PersistError registram a etapa de publicação do destino e o handle original continua utilizável.

Caminhos em snapshots e detalhes de erro são strings quando representam Unicode válido. Nomes em bytes do Unix que Python representa com `surrogateescape` aparecem em JSON como `{"encoding": "unix_bytes", "bytes": [...]}`; restaure-os com `Path(os.fsdecode(bytes(value["bytes"])))`. Nomes Windows com substitutos UTF-16 sem par usam `{"encoding": "windows_wide", "units": [...]}`, com cada entrada do array contendo uma unidade de código original de 16 bits. Caminhos codificados descrevem sua plataforma original. Em tempo de execução, `artifact.path` continua sendo um `pathlib.Path`, e `BuildOptions.to`, `update_from` e `CompareOptions.against` preservam valores de caminho nativos através do binding.

Uma extensão nativa ausente ou incompatível gera um erro de importação com `BINDING.EXTENSION_UNAVAILABLE` ou `BINDING.VERSION_MISMATCH`. Instale um wheel compatível ou recompile o código-fonte com maturin. Operações simultâneas em um mesmo Project são rejeitadas; use projetos independentes para trabalho concorrente.
