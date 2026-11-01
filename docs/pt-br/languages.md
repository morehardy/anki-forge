Use Rust, Node / TypeScript ou Python para gerar baralhos do Anki a partir dos seus dados. As três opções exportam um arquivo persistente `spanish.apkg` sem precisar instalar o Anki.

| Linguagem | Pacote publicado | Instalação | Primeiro baralho completo |
| --- | --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | `cargo add ankiforge@0.3.1` | [Início rápido com Rust](/docs/quickstart/) |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | `npm install --include=optional ankiforge@0.3.1` | [Início rápido com Node](/docs/node-quickstart/) |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | `python -m pip install ankiforge==0.3.1` | [Início rápido com Python](/docs/python-quickstart/) |

Rust exige a versão 1.92 ou posterior. Node exige a versão 22.13 ou posterior e a dependência nativa opcional correspondente. Python usa um wheel nativo para um interpretador CPython convencional; as verificações registradas de lançamento abrangem CPython 3.11–3.14. Consulte os [ambientes verificados](/docs/compatibility/).

<!-- section: next-tasks -->
## Próximas tarefas

Gere e atualize um baralho a partir dos mesmos [arquivos JSON, imagem e áudio](/docs/json-to-anki/), ou compare [Anki Forge e genanki](/docs/anki-forge-vs-genanki/) para escolher a biblioteca mais adequada ao seu fluxo.

Adicione [imagens e áudio](/docs/media/), crie [tipos de nota personalizados](/docs/custom-notetypes/) ou [compare e atualize um baralho já distribuído](/docs/updates/).

Para TypeScript, consulte a [configuração do compilador](/docs/node-api/#typescript). Se você usa genanki, leia o [guia de migração](/docs/genanki-migration/). O desenvolvimento do repositório e os recursos disponíveis apenas no código-fonte têm suas próprias [instruções de compilação](/docs/development/#source-builds).
