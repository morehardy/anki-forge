# Genera un APKG de Anki con Node.js

Usa Node.js 22.13 o posterior. En un directorio nuevo de aplicación:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

Este comando instala el [paquete publicado en npm](https://www.npmjs.com/package/ankiforge) y la dependencia nativa opcional correspondiente. Consulta los [entornos verificados](/docs/compatibility/) para conocer las plataformas cubiertas.

<!-- section: export-a-deck -->
## Exporta un mazo

Guarda lo siguiente como `main.mjs`:

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

Abre el archivo persistente `spanish.apkg` en Anki. Contiene **hola → hello** en el mazo **Spanish**. Conserva el espacio de nombres `spanish` y la clave `es:hola` cuando edites el contenido. Cerrar el objeto que gestiona el artefacto persistente deja el archivo en el disco. No necesitas instalar Anki para exportarlo.

<!-- section: typescript-and-next-tasks -->
## TypeScript y siguientes pasos

El [tutorial de JSON a Anki](/docs/json-to-anki/#node) incluye el proceso completo de compilación y ejecución de TypeScript.

Usa `.mjs` para ESM o `require('ankiforge')` para CommonJS. El paquete incluye las declaraciones de tipos de TypeScript; consulta la [configuración de TypeScript](/docs/node-api/#typescript).

- [Imágenes y audio](/docs/media/) y [API multimedia de Node](/docs/node-api/#media-and-image-occlusion).
- [Tipos de nota personalizados](/docs/custom-notetypes/) y [paquetes de plantillas](/docs/templates/).
- [Oclusión de imágenes](/docs/image-occlusion/).
- [Comparación y actualización](/docs/updates/).
- [API de Node](/docs/node-api/) y [solución de problemas](/docs/troubleshooting/).

La [guía de compilación desde el código fuente](/docs/development/) explica las compilaciones y los paquetes locales.
