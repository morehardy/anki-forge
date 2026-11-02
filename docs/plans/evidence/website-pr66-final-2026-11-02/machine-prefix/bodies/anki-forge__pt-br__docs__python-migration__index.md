# Transição para a nova API de Python

Canonical page: https://morehardy.github.io/anki-forge/pt-br/docs/python-migration/
Language: pt-BR
Description: Adapte seu código ao SDK atual de Python, que mantém a propriedade dos seus valores, e entenda a ruptura com a API anterior.

Use `python -m pip install ankiforge` e `from ankiforge import ...`. A antiga distribuição `anki-forge` e o nome de importação `anki_forge` não são aliases de compatibilidade. Ao usar uma cópia do código-fonte, recompile a extensão nativa com Maturin depois de alterar os nomes. Consulte o [guia de instalação](https://morehardy.github.io/anki-forge/pt-br/docs/python-quickstart/).

Não há uma camada de compatibilidade com a API anterior. Substitua as antigas APIs de Deck, registro e execução pelo modelo público que mantém a propriedade dos seus valores:

| Fluxo anterior | Fluxo atual |
| --- | --- |
| Deck ou Project com ID estável opcional | `Project(namespace, default_deck=...)` |
| `add_note(note)` e identidade derivada do conteúdo | `add(stable_key, note)` |
| NoteType mutável e add\_notetype | `NoteType.builder(key)...build()`, seguido de `model.note()` |
| Referências aos modelos por strings | As notas mantêm a propriedade do modelo validado |
| Registro de mídia do projeto | Snapshots de `Media.file` / `Media.bytes` que não dependem dos arquivos de origem |
| Mídia já renderizada e métodos de atribuição específicos para cada campo | `Content.sequence`, `media.image()`, `media.sound()`, `Note.field` |
| HTML implícito | Strings são Text; use `Content.html` |
| Máscaras sem identidade estável | `Mask.rect(key, x, y, width, height)` |
| Artefato opcional e ensure\_success | `BuildOutput` com artefato garantido; falhas lançam erros estruturados |
| Resultado deduzido dos diagnósticos | As representações de Output/Error fornecem o resultado real; Report contém apenas observações |
| Opções de diff, lockfile e segurança de atualização | `compare(CompareOptions.against(...))`, `BuildOptions.update_from(...)` |
| Pacote executado em um subprocesso | Binding PyO3 no mesmo processo para a API padrão de Rust |

Somente APKGs originais distribuídos completos, com as evidências de identidade atuais, servem como base de atualização. Pacotes de templates antigos e evidências de APKGs anteriores não são recuperados automaticamente. Consulte o [README do SDK](https://github.com/morehardy/anki-forge/blob/main/bindings/python/README.md) e seus exemplos executáveis para seguir o novo fluxo.
