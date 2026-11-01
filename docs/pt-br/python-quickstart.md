# Gere cartões do Anki com Python

Crie e ative um ambiente virtual e instale o [pacote publicado no PyPI](https://pypi.org/project/ankiforge/):

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

No Windows, ative o ambiente com `.venv\Scripts\Activate.ps1`. Use CPython convencional 3.11 ou posterior e um wheel correspondente à plataforma; as verificações registradas de lançamento abrangem CPython 3.11–3.14. Consulte os [ambientes verificados](/docs/compatibility/).

<!-- section: export-a-deck -->
## Exporte um baralho

Salve o programa como `main.py`:

<!-- source: bindings/python/examples/quickstart.py -->
```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```
<!-- /source -->

```sh
python main.py
```

Abra o arquivo persistente `spanish.apkg` no Anki. Ele contém **hola → hello** no baralho **Spanish**. Mantenha o namespace `spanish` e a chave da nota `es:hola` estáveis nas próximas edições. A exportação não exige que o Anki esteja instalado.

<!-- section: continue -->
## Continue

Para uma planilha de vocabulário, siga [CSV para Anki](/docs/csv-to-anki/). Para criar rascunhos de cartões a partir de anotações, comece pela [criação de cartões com IA](/docs/ai-anki-cards/).

Para uma tarefa completa com mídia e atualização, siga [JSON para Anki](/docs/json-to-anki/#python).

- [Imagens e áudio](/docs/media/) e [API de mídia do Python](/docs/python-api/#media-and-occlusion)
- [Tipos de nota personalizados](/docs/custom-notetypes/) e [pacotes de modelos](/docs/templates/)
- [Oclusão de imagem](/docs/image-occlusion/)
- [Comparação e atualização](/docs/updates/)
- [API de Python](/docs/python-api/) e [diagnósticos](/docs/python-diagnostics/)
- [Migração do genanki](/docs/genanki-migration/)

Para compilar a extensão nativa, consulte [compilação a partir do código-fonte](/docs/development/).
