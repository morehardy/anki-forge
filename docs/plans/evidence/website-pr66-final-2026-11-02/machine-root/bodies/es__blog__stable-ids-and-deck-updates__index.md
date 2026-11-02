# Identidades estables para actualizar mazos de Anki

Canonical page: https://ankiforge.dev/es/blog/stable-ids-and-deck-updates/
Language: es
Description: Cómo combinar ID estables y un paquete de referencia al actualizar un mazo de Anki generado.
Published: 2026-09-22T00:00:00.000Z

Un mazo generado rara vez queda terminado tras la primera exportación. Las definiciones mejoran, los ejemplos se aclaran y los errores se corrigen. La siguiente generación necesita saber qué notas siguen siendo las mismas, aunque su contenido haya mejorado.

Considera este pequeño cambio:

| Campo | Primera versión | Siguiente versión |
| --- | --- | --- |
| Anverso | hola | hola |
| Reverso | hello | hello; hi |
| ID estable | es:hola | es:hola |

El significado se ha ampliado. La nota lógica sigue siendo la misma.

## Separar la identidad del contenido

Un ID estable explícito proporciona a la nota una clave duradera. Conserva ese ID cuando edites la nota y elige uno nuevo cuando crees una nota lógica diferente.

La identidad también tiene un contexto de proyecto. Mantén el mismo espacio de nombres del proyecto entre las versiones de un mazo. Un ID de nota por sí solo es únicamente una parte de un flujo de actualización repetible.

## Usar el paquete anterior como referencia

Un paquete anterior indica a Anki Forge qué se exportó antes. Utilízalo al generar el proyecto revisado:

```rust
let comparison = project.compare(CompareOptions::against(&previous))?;
ensure!(comparison.policy().allows_publication());
let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
```

En este fragmento, `project` contiene la nota revisada, `previous` apunta a la versión 1 y `next` es una ruta distinta para la versión 2. El [ejemplo completo](https://ankiforge.dev/generated/website_showcase.rs) incluye ambas definiciones del proyecto y verifica que se conserva la identidad de una nota.

Mantén el paquete de entrada y el de salida en rutas diferentes. Sobrescribir el paquete de referencia elimina las pruebas que necesitas para entender y verificar la publicación.

## Conservar el paquete original distribuido

Cada paquete generado contiene pruebas completas de identidad y revisión. La siguiente generación valida esas pruebas frente a la base de datos y los recursos multimedia reales antes de reutilizar identidades. Conserva el paquete original que distribuiste; una reexportación desde Anki no sirve como paquete de referencia compatible.

La [guía de actualización](https://ankiforge.dev/es/docs/updates/) describe las actualizaciones estrictas, las políticas de riesgo y las limitaciones de importación del cliente.

## Revisar la generación y la importación

Comprueba el informe de generación y sus diagnósticos de seguridad de actualización antes de distribuir un paquete. Si la generación no puede establecer la correspondencia de identidad de una nota, investiga los datos de origen y el paquete de referencia en lugar de considerar la existencia de un archivo de salida como prueba de que la actualización es correcta.

Anki aplica después su propio comportamiento de importación. La configuración de importación, el contenido de las notas existentes, las modificaciones locales y la versión de Anki pueden influir en el resultado. La identidad estable ayuda a identificar la nota correspondiente; no anula esas decisiones ni promete que cada importación conserve todos los aspectos del estado local.

El [ejemplo con dos versiones](https://ankiforge.dev/es/examples/#updates) proporciona ambos paquetes para que pruebes una actualización pequeña y fácil de inspeccionar antes de diseñar tu propio proceso de publicación.
