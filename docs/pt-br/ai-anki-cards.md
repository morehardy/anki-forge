# Gere cartões do Anki com um assistente de IA

Use um assistente de IA para criar rascunhos de perguntas e respostas a partir das suas anotações, confira-os com o material de origem e use o Anki Forge para exportar um baralho `.apkg`. Nesse fluxo, o Anki Forge é a biblioteca de código aberto que gera o pacote: o assistente prepara o conteúdo ou escreve um script, e seu programa local em Python, Node.js ou Rust gera os cartões.

Você pode [copiar uma descrição de tarefa para um assistente de programação](/#ai-quickstart) para começar com uma linguagem ou seguir o fluxo com CSV abaixo. O gerador de instruções do site monta o texto localmente; ele não chama um modelo de IA nem envia suas anotações.

<!-- section: choose-input -->
## Comece com material que você possa conferir

Use um trecho curto sobre um assunto claro, como suas próprias anotações de aula ou uma lista de vocabulário. Para PDFs, digitalizações ou slides de aula, extraia e confira o texto com outra ferramenta primeiro. O Anki Forge aceita o conteúdo que você fornece pelas APIs; extração de PDF e OCR não fazem parte da biblioteca.

Faça cada pergunta avaliar um único fato. Inclua contexto suficiente para que a resposta faça sentido quando o cartão aparecer fora do documento original.

<!-- section: draft-cards -->
## Peça os cartões em CSV

Cole este exemplo no seu assistente. Ao criar seu próprio baralho, substitua o trecho de origem e ajuste a quantidade de notas e os IDs solicitados para as novas notas:

```text
Turn the source below into exactly three Basic Anki notes.
Use only facts stated in the source. Do not add facts from memory.
Each question must test one fact and be understandable on its own.
Return only UTF-8 CSV with the header id,front,back, with no Markdown fence.
Use the IDs es:hola, es:gracias and es:adios for these three vocabulary items.
Quote any CSV field containing a comma, quote or newline correctly.

Source:
In Spanish, hola means hello, gracias means thank you, and adiós means goodbye.
```

O conteúdo esperado equivale a:

```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```

O assistente pode formular a frente do cartão de outra maneira. Confira cada linha com o material de origem, remova respostas ambíguas ou sem fundamento e salve o resultado revisado como `cards.csv`. Atribua um ID único a cada nova nota e mantenha-o nas próximas edições; não peça ao assistente que gere novos IDs para um baralho existente.

<!-- section: export-apkg -->
## Transforme os cartões revisados em um APKG

Siga [CSV para Anki com Python](/docs/csv-to-anki/) para obter o programa completo, os comandos de instalação e os dados de exemplo. O script verifica a estrutura do CSV, exige IDs únicos e grava um pacote persistente. O exemplo cria três cartões Basic sem mídia.

Se os seus dados já estiverem em JSON ou você precisar de imagens e áudio, use o [tutorial de JSON para Anki](/docs/json-to-anki/), com programas completos em Python, Node / TypeScript e Rust. Sua aplicação lê o CSV ou JSON antes de passar as notas ao Anki Forge.

Para receber ajuda ao escrever um programa próprio, use a [descrição de tarefa para assistentes de programação](/#ai-quickstart) e informe os campos de entrada, o tipo de cartão desejado e o nome do arquivo de saída. A [Skill opcional do Anki Forge](https://github.com/morehardy/anki-forge/blob/main/docs/ai-assistants.md) fornece referências da API pública e fluxos de trabalho para assistentes que carregam Skills.

<!-- section: check-result -->
## Confira os cartões antes de estudar

Abra o APKG gerado no Anki e confira a quantidade de cartões, as perguntas, as respostas e a formatação. No exemplo de CSV, espere três notas e três cartões no baralho Spanish. Uma exportação bem-sucedida confirma que um pacote foi gerado; ela não demonstra que as respostas da IA estão corretas ou são úteis para estudar.

Para perguntas de preenchimento de lacunas, use [cartões Cloze](/docs/cards/). Para identificar partes de diagramas, use [oclusão de imagem](/docs/image-occlusion/) com a imagem e as coordenadas das máscaras que você fornecer; o recurso não detecta rótulos automaticamente em imagens.

Guarde o APKG original distribuído para a próxima versão e siga o [fluxo de atualização](/docs/updates/). Preserve o namespace e os IDs das notas, examine a comparação e considere as opções de importação do Anki e as edições locais mais recentes.
