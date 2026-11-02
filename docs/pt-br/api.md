# Visão geral da API pública

Anki Forge é uma biblioteca de código aberto para gerar APKG do Anki, com núcleo em Rust e interfaces para Rust, Python e Node.js. Use o pacote público `ankiforge` para criar notas, incluir mídia e inspecionar atualizações antes de distribuir um novo baralho. A licença é MIT.

| Tarefa | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| Criar notas e modelos | [API de Rust](/docs/rust-api/) | [API de Python](/docs/python-api/) | [API de Node](/docs/node-api/) |
| Imagens, áudio e oclusão | [Mídia](/docs/media/) | [Mídia com dados próprios](/docs/python-api/#media-and-occlusion) | [Mídia com dados próprios](/docs/node-api/#media-and-image-occlusion) |
| Diagnosticar falhas | [Solução de problemas](/docs/troubleshooting/) | [Diagnósticos](/docs/python-diagnostics/) | [Erros](/docs/node-api/#errors) |
| Comparar e publicar | [Fluxo de atualização](/docs/updates/) | [Geração e comparação](/docs/python-api/) | [Geração e comparação](/docs/node-api/#build-compare-and-output) |

<!-- section: stable-note-identity -->
## Identidade estável das notas

Mantenha um namespace estável de projeto e uma chave estável para cada nota lógica. Atualizar uma versão existente também exige seu APKG original distribuído. Uma reexportação do Anki não substitui as evidências completas de identidade desse pacote original.

<!-- section: start-with-a-task -->
## Comece por uma tarefa

[Escolha sua linguagem](/docs/languages/) ou siga a [receita completa de geração e atualização a partir de JSON](/docs/json-to-anki/). Veja [compatibilidade](/docs/compatibility/) para os ambientes de execução dos pacotes publicados e [garantias de saída](/docs/build-guarantees/) para entender quem mantém os arquivos disponíveis.
