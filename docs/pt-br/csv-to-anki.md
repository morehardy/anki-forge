# Converta CSV em cartões do Anki com Python

Transforme uma planilha de vocabulário em um arquivo `.apkg` do Anki com Python e Anki Forge. Este exemplo lê um CSV UTF-8 com as colunas `id`, `front` e `back`, cria cartões Basic de pergunta e resposta e exporta um baralho que você pode compartilhar. O leitor de CSV da biblioteca padrão do Python lê a planilha; o Anki Forge gera o pacote. A exportação não exige que o Anki esteja instalado.

Para uma importação pontual de planilha, o Anki já oferece suporte a [arquivos de texto delimitado](https://docs.ankiweb.net/manual/importing/text-files). Use este tutorial quando precisar de um script repetível, IDs de nota próprios e um APKG para distribuição. Primeiro salve arquivos Excel `.xlsx` como CSV UTF-8.

<!-- section: prepare-csv -->
## Prepare o CSV

Salve o conteúdo abaixo como `cards.csv` ou [baixe o exemplo](/generated/recipes/csv-to-anki/cards.csv):

<!-- source: docs/recipes/csv-to-anki/cards.csv -->
```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```
<!-- /source -->

Mantenha o cabeçalho exatamente como mostrado e use vírgulas como separadores. Coloque entre aspas os campos que contêm vírgulas, aspas duplas ou quebras de linha, seguindo as [regras de aspas do CSV](https://docs.python.org/3/library/csv.html). O script aceita a marca de ordem de bytes UTF-8, incluída por algumas exportações de planilhas. Ele rejeita IDs duplicados, valores ausentes, colunas extras e entradas vazias.

Atribua cada ID uma única vez e mantenha-o quando a pergunta ou a resposta mudar. Não derive os IDs da posição da linha nem do texto editável do cartão. Use outro namespace para conjuntos de dados sem relação entre si; mantenha o namespace nas versões seguintes deste baralho.

<!-- section: install -->
## Instale o Anki Forge

Use CPython convencional 3.11 ou posterior em uma [plataforma compatível](/docs/compatibility/). Crie e ative um ambiente virtual:

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

No Windows, ative o ambiente com `.venv\Scripts\Activate.ps1`. Essa conversão não exige uma biblioteca de planilhas nem um serviço de IA.

<!-- section: convert -->
## Gere o APKG

Salve o programa abaixo como `csv_to_anki.py` no mesmo diretório de `cards.csv` ou [baixe o script](/generated/recipes/csv-to-anki/csv_to_anki.py):

<!-- source: bindings/python/examples/csv_to_anki.py -->
```python
"""Convert a UTF-8 id,front,back CSV into a persistent Basic-card APKG."""
import argparse
import csv
from pathlib import Path

from ankiforge import BuildOptions, Note, Project


def build_csv(source: Path, destination: Path, namespace: str, deck: str) -> int:
    if destination.exists():
        raise ValueError('Choose a new output path; keep previously distributed APKGs')
    project = Project(namespace, default_deck=deck)
    seen = set()
    with source.open(encoding='utf-8-sig', newline='') as handle:
        rows = csv.DictReader(handle, strict=True)
        if rows.fieldnames != ['id', 'front', 'back']:
            raise ValueError('Expected the header id,front,back in that order')
        for row in rows:
            if None in row or any(value is None or not value.strip() for value in row.values()):
                raise ValueError(f'Line {rows.line_num}: expected three nonempty fields')
            key = row['id'].strip()
            if key in seen:
                raise ValueError(f'Line {rows.line_num}: duplicate id {key!r}')
            project.add(key, Note.basic(row['front'], row['back']))
            seen.add(key)
    if not seen:
        raise ValueError('The CSV must contain at least one note')
    project.build(BuildOptions.to(destination))
    return len(seen)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--namespace', default='csv-vocabulary')
    parser.add_argument('--deck', default='Vocabulary')
    args = parser.parse_args()
    count = build_csv(args.source, args.destination, args.namespace, args.deck)
    print(f'{args.destination}: {count} notes, {count} Basic cards')
```
<!-- /source -->

Execute-o nesse diretório:

```sh
python csv_to_anki.py cards.csv vocabulary-v1.apkg --namespace spanish-vocabulary --deck Spanish
```

O exemplo gera `vocabulary-v1.apkg` com **três notas e três cartões Basic** no baralho **Spanish**, sem mídia. Ele imprime:

```text
vocabulary-v1.apkg: 3 notes, 3 Basic cards
```

Escolha um caminho de saída que ainda não exista. O script se recusa a substituir um arquivo anterior para que você possa guardar o pacote original distribuído.

<!-- section: verify-and-extend -->
## Importe, confira e amplie

Abra `vocabulary-v1.apkg` no Anki. Confira se o baralho contém `hola → hello`, `gracias → thank you` e `adiós → goodbye`; visualize os cartões e verifique os acentos. Confira o conteúdo antes de distribuí-lo, especialmente se uma IA tiver criado o CSV.

Este tutorial cria cartões Basic somente com texto. Para outros fluxos:

- [Criação de cartões do Anki com IA](/docs/ai-anki-cards/): prepare e revise linhas CSV a partir das suas anotações.
- [JSON para Anki](/docs/json-to-anki/): use Python, Node / TypeScript ou Rust, inclua imagens e áudio e revise uma segunda versão.
- [Cartões Basic e Cloze](/docs/cards/): escolha perguntas e respostas ou preenchimento de lacunas.
- [Atualização de baralhos gerados](/docs/updates/): use o APKG original anterior como base de atualização e examine a comparação.

Para uma nova versão, preserve o namespace e os IDs das notas e siga o fluxo de atualização antes de publicar. IDs estáveis ajudam a identificar as notas; as opções de importação do Anki e as edições locais mais recentes também afetam as atualizações.
