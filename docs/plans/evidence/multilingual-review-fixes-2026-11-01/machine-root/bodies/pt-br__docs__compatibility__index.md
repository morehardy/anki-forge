# Compatibilidade e ambientes verificados

Canonical page: https://ankiforge.dev/pt-br/docs/compatibility/
Language: pt-BR
Description: Consulte os pacotes públicos, os ambientes testados e as condições de importação e reprodução nos clientes Anki.

## Pacotes e ambientes verificados

Os pacotes públicos de Rust, npm e PyPI estão na versão `0.3.1`, conforme verificado em 2026-10-13. Comece por [Rust](https://ankiforge.dev/pt-br/docs/installation/), [Node / TypeScript](https://ankiforge.dev/pt-br/docs/node-quickstart/) ou [Python](https://ankiforge.dev/pt-br/docs/python-quickstart/).

| Interface | Pacote | Ambiente de execução e verificações |
| --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | Rust 1.92.0 e a versão estável atual; aplicações consumidoras do pacote nas quatro plataformas compatíveis, além de uma aplicação nova usando o pacote público do crates.io |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | Node 22.13.0 e versões atuais das séries 22, 24 e 26; 16 verificações de instalação pelo npm público, ESM/CJS e TypeScript |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | CPython convencional 3.11–3.14; 16 verificações isoladas de instalação de wheels do PyPI público |

Os pacotes nativos publicados abrangem macOS arm64/x64, Linux x64 GNU/glibc e Windows x64. A disponibilidade de um pacote, por si só, não comprova que ele foi executado com sucesso em toda configuração de máquina. A [cobertura de Node](https://github.com/morehardy/anki-forge/blob/main/bindings/node/COVERAGE.md), a [cobertura de Python](https://github.com/morehardy/anki-forge/blob/main/bindings/python/COVERAGE.md) e as evidências de lançamento registram as verificações exatas. As execuções bem-sucedidas de lançamento da versão `0.3.1` são as de [Rust](https://github.com/morehardy/anki-forge/actions/runs/38371221696), [npm](https://github.com/morehardy/anki-forge/actions/runs/38371222114) e [PyPI](https://github.com/morehardy/anki-forge/actions/runs/38371222047).

Outras implementações de Python, interpretadores sem GIL e subinterpretadores exigem validação separada. Navegadores, Electron, Bun, Alpine/musl e Linux/Windows ARM64 estão fora do escopo registrado de verificação de Node.

Os recursos dos guias de [mídia em lote](https://ankiforge.dev/pt-br/docs/media/#batch-import-from-source) e [publicação preparada](https://ankiforge.dev/pt-br/docs/updates/#build-once-review-then-publish-from-source) estão disponíveis nos pacotes públicos `0.3.1`.

## Importação e reprodução no Anki

A exportação normal não exige que o Anki esteja instalado. A importação, a renderização e a reprodução de mídia continuam dependendo da versão do cliente, dos codecs e das opções de importação. O benchmark de 2026-10-08 passou por 40 verificações de importação, conteúdo e renderização representativa no Anki, sendo 20 por implementação; ele não abrangeu todos os clientes nem todos os comportamentos da interface gráfica e do áudio.

Para atualizar um baralho, guarde o APKG original distribuído da versão anterior. Pacotes reexportados pelo Anki não preservam as evidências de identidade necessárias. Omitir uma nota de um pacote não a exclui da coleção de quem estuda. Alterações de campos, modelos e campos de ordenação, edições locais e histórico de revisão exigem testes específicos para cada cliente.

Use os [cenários manuais de cliente](https://github.com/morehardy/anki-forge/blob/main/docs/manual-validation/anki-desktop-v1/TEMPLATE.md) e a versão e as configurações exatas do Anki de cada verificação de referência para avaliar sua distribuição. Leia [atualizações](https://ankiforge.dev/pt-br/docs/updates/) e [garantias de geração](https://ankiforge.dev/pt-br/docs/build-guarantees/) para entender as condições dos clientes e o comportamento da publicação de cada arquivo.
