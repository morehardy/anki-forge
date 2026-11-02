# Gere cartões do Anki com Node.js

Canonical page: https://ankiforge.dev/pt-br/docs/node-quickstart/
Language: pt-BR
Description: Instale ankiforge pelo npm, gere cartões com Node.js ou TypeScript e exporte um baralho APKG para importar no Anki.

Use Node.js 22.13 ou posterior. Em um novo diretório de aplicação:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

O comando instala o [pacote publicado no npm](https://www.npmjs.com/package/ankiforge) e a dependência nativa opcional correspondente. Consulte os [ambientes verificados](https://ankiforge.dev/pt-br/docs/compatibility/) para saber quais plataformas foram testadas.

## Exporte um baralho

Salve o programa como `main.mjs`:

```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```

```sh
node main.mjs
```

Abra o arquivo persistente `spanish.apkg` no Anki. Ele contém **hola → hello** no baralho **Spanish**. Mantenha o namespace `spanish` e a chave `es:hola` estáveis nas próximas edições. Fechar o artefato persistente mantém o arquivo no disco. A exportação não exige que o Anki esteja instalado.

## TypeScript e próximas tarefas

O [tutorial de JSON para Anki](https://ankiforge.dev/pt-br/docs/json-to-anki/#node) inclui o fluxo completo de compilação e execução em TypeScript.

Use `.mjs` para ESM ou `require('ankiforge')` para CommonJS. O pacote inclui declarações de tipos TypeScript; consulte a [configuração do TypeScript](https://ankiforge.dev/pt-br/docs/node-api/#typescript).

-   [Imagens e áudio](https://ankiforge.dev/pt-br/docs/media/) e [API de mídia do Node](https://ankiforge.dev/pt-br/docs/node-api/#media-and-image-occlusion)
-   [Tipos de nota personalizados](https://ankiforge.dev/pt-br/docs/custom-notetypes/) e [pacotes de templates](https://ankiforge.dev/pt-br/docs/templates/)
-   [Oclusão de imagem](https://ankiforge.dev/pt-br/docs/image-occlusion/)
-   [Comparação e atualização](https://ankiforge.dev/pt-br/docs/updates/)
-   [API de Node](https://ankiforge.dev/pt-br/docs/node-api/) e [solução de problemas](https://ankiforge.dev/pt-br/docs/troubleshooting/)

A compilação e o empacotamento local estão descritos em [compilação a partir do código-fonte](https://ankiforge.dev/pt-br/docs/development/).
