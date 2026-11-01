# Gere cartões do Anki com Node.js

Use Node.js 22.13 ou posterior. Em um novo diretório de aplicação:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

O comando instala o [pacote publicado no npm](https://www.npmjs.com/package/ankiforge) e a dependência nativa opcional correspondente. Consulte os [ambientes verificados](/docs/compatibility/) para saber quais plataformas foram testadas.

<!-- section: export-a-deck -->
## Exporte um baralho

Salve o programa como `main.mjs`:

<!-- source: bindings/node/examples/quickstart.mjs -->
```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```
<!-- /source -->

```sh
node main.mjs
```

Abra o arquivo persistente `spanish.apkg` no Anki. Ele contém **hola → hello** no baralho **Spanish**. Mantenha o namespace `spanish` e a chave `es:hola` estáveis nas próximas edições. Fechar o artefato persistente mantém o arquivo no disco. A exportação não exige que o Anki esteja instalado.

<!-- section: typescript-and-next-tasks -->
## TypeScript e próximas tarefas

O [tutorial de JSON para Anki](/docs/json-to-anki/#node) inclui o fluxo completo de compilação e execução em TypeScript.

Use `.mjs` para ESM ou `require('ankiforge')` para CommonJS. O pacote inclui declarações de tipos TypeScript; consulte a [configuração do TypeScript](/docs/node-api/#typescript).

- [Imagens e áudio](/docs/media/) e [API de mídia do Node](/docs/node-api/#media-and-image-occlusion)
- [Tipos de nota personalizados](/docs/custom-notetypes/) e [pacotes de modelos](/docs/templates/)
- [Oclusão de imagem](/docs/image-occlusion/)
- [Comparação e atualização](/docs/updates/)
- [API de Node](/docs/node-api/) e [solução de problemas](/docs/troubleshooting/)

A compilação e o empacotamento local estão descritos em [compilação a partir do código-fonte](/docs/development/).
