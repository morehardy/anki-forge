# Genera un APKG de Anki con Node.js

Canonical page: https://ankiforge.dev/es/docs/node-quickstart/
Language: es
Description: Instala el paquete publicado de Node.js y exporta tu primer mazo con un programa completo. Continúa con TypeScript, imágenes y actualizaciones.

Usa Node.js 22.13 o posterior. En un directorio nuevo de aplicación:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

Este comando instala el [paquete publicado en npm](https://www.npmjs.com/package/ankiforge) y la dependencia nativa opcional correspondiente. Consulta los [entornos verificados](https://ankiforge.dev/es/docs/compatibility/) para conocer las plataformas cubiertas.

## Exporta un mazo

Guarda lo siguiente como `main.mjs`:

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

Abre el archivo persistente `spanish.apkg` en Anki. Contiene **hola → hello** en el mazo **Spanish**. Conserva el espacio de nombres `spanish` y la clave `es:hola` cuando edites el contenido. Cerrar el objeto que gestiona el artefacto persistente deja el archivo en el disco. No necesitas instalar Anki para exportarlo.

## TypeScript y siguientes pasos

El [tutorial de JSON a Anki](https://ankiforge.dev/es/docs/json-to-anki/#node) incluye el proceso completo de compilación y ejecución de TypeScript.

Usa `.mjs` para ESM o `require('ankiforge')` para CommonJS. El paquete incluye las declaraciones de tipos de TypeScript; consulta la [configuración de TypeScript](https://ankiforge.dev/es/docs/node-api/#typescript).

-   [Imágenes y audio](https://ankiforge.dev/es/docs/media/) y [API multimedia de Node](https://ankiforge.dev/es/docs/node-api/#media-and-image-occlusion).
-   [Tipos de nota personalizados](https://ankiforge.dev/es/docs/custom-notetypes/) y [paquetes de plantillas](https://ankiforge.dev/es/docs/templates/).
-   [Oclusión de imágenes](https://ankiforge.dev/es/docs/image-occlusion/).
-   [Comparación y actualización](https://ankiforge.dev/es/docs/updates/).
-   [API de Node](https://ankiforge.dev/es/docs/node-api/) y [solución de problemas](https://ankiforge.dev/es/docs/troubleshooting/).

La [guía de compilación desde el código fuente](https://ankiforge.dev/es/docs/development/) explica las compilaciones y los paquetes locales.
