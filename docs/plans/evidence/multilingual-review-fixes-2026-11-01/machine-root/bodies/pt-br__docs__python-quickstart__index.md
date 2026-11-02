# Gere cartões do Anki com Python

Canonical page: https://ankiforge.dev/pt-br/docs/python-quickstart/
Language: pt-BR
Description: Instale ankiforge pelo PyPI, crie cartões de pergunta e resposta com um script Python curto e exporte um baralho APKG para importar no Anki.

Crie e ative um ambiente virtual e instale o [pacote publicado no PyPI](https://pypi.org/project/ankiforge/):

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

No Windows, ative o ambiente com `.venv\Scripts\Activate.ps1`. Use CPython convencional 3.11 ou posterior e um wheel correspondente à plataforma; as verificações registradas de lançamento abrangem CPython 3.11–3.14. Consulte os [ambientes verificados](https://ankiforge.dev/pt-br/docs/compatibility/).

## Exporte um baralho

Salve o programa como `main.py`:

```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```

```sh
python main.py
```

Abra o arquivo persistente `spanish.apkg` no Anki. Ele contém **hola → hello** no baralho **Spanish**. Mantenha o namespace `spanish` e a chave da nota `es:hola` estáveis nas próximas edições. A exportação não exige que o Anki esteja instalado.

## Continue

Para uma planilha de vocabulário, siga [CSV para Anki](https://ankiforge.dev/pt-br/docs/csv-to-anki/). Para criar rascunhos de cartões a partir de anotações, comece pela [criação de cartões com IA](https://ankiforge.dev/pt-br/docs/ai-anki-cards/).

Para uma tarefa completa com mídia e atualização, siga [JSON para Anki](https://ankiforge.dev/pt-br/docs/json-to-anki/#python).

-   [Imagens e áudio](https://ankiforge.dev/pt-br/docs/media/) e [API de mídia do Python](https://ankiforge.dev/pt-br/docs/python-api/#media-and-occlusion)
-   [Tipos de nota personalizados](https://ankiforge.dev/pt-br/docs/custom-notetypes/) e [pacotes de modelos](https://ankiforge.dev/pt-br/docs/templates/)
-   [Oclusão de imagem](https://ankiforge.dev/pt-br/docs/image-occlusion/)
-   [Comparação e atualização](https://ankiforge.dev/pt-br/docs/updates/)
-   [API de Python](https://ankiforge.dev/pt-br/docs/python-api/) e [diagnósticos](https://ankiforge.dev/pt-br/docs/python-diagnostics/)
-   [Migração do genanki](https://ankiforge.dev/pt-br/docs/genanki-migration/)

Para compilar a extensão nativa, consulte [compilação a partir do código-fonte](https://ankiforge.dev/pt-br/docs/development/).
