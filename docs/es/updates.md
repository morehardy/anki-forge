# Actualiza y distribuye un mazo

Publica correcciones o tarjetas nuevas para un mazo de Anki que ya hayas distribuido. Los ID de nota estables ayudan a Anki Forge y a Anki a reconocer las notas existentes en lugar de tratarlas como nuevas; las opciones de importación de Anki y los cambios locales más recientes también influyen en lo que se actualiza.

Mantén estables el espacio de nombres del proyecto y las claves de notas, modelos, campos y plantillas. Genera cada versión nueva a partir del **APKG original de distribución** anterior, que contiene pruebas completas de identidad con su versión de formato. Guarda ese paquete junto con la revisión de su código fuente.

<!-- section: run-two-versions-and-compare -->
## Genera dos versiones y compáralas

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Este comando escribe `spanish-v1.apkg` y `spanish-v2.apkg`, compara la respuesta modificada y comprueba los recuentos del paquete actualizado. El código ejecutable es:

<!-- source: anki_forge/examples/docs_workflow.rs#updates -->
```rust
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("docs-spanish-updates")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", answer))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?.build(BuildOptions::to(&previous))?;
    let project = vocabulary("hello; hi")?;
    let comparison = project.compare(CompareOptions::against(&previous))?;
    ensure!(comparison.policy().allows_publication());
    let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
    verify(&built, 1, 1, 0)?;
    ensure!(built.report().comparison().is_some());
    Ok(())
}
```
<!-- /source -->

`compare` termina correctamente incluso cuando sus hallazgos impedirían publicar. Consulta `comparison.policy().allows_publication()` por separado del nivel de riesgo original y de las pruebas. `build(...update_from(...))` ejecuta el mismo análisis y devuelve `BuildError` antes de publicar si su política bloquea la versión candidata.

<!-- section: policy-and-inspection-limits -->
## Política y límites de inspección

Este programa completo elige un umbral más estricto y utiliza la misma configuración para la comparación y la actualización:

```rust
use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::InspectLimits;
use ankiforge::update::{CompareOptions, RiskLevel, UpdatePolicy};

fn main() -> anyhow::Result<()> {
    let mut first = Project::new("policy-example")?;
    first.add("cell", Note::basic("Cell?", "Unit of life"))?;
    first.build(BuildOptions::to("policy-v1.apkg"))?;
    let mut next = Project::new("policy-example")?;
    next.add("cell", Note::basic("Cell?", "The basic unit of life"))?;
    let mut limits = InspectLimits::default();
    limits.max_collection_bytes = 1 << 30;
    let policy = UpdatePolicy::default().fail_on(RiskLevel::Medium);
    let comparison = next.compare(CompareOptions::against("policy-v1.apkg")
        .inspect_limits(limits.clone()).update_policy(policy.clone()))?;
    assert!(comparison.policy().allows_publication());
    next.build(BuildOptions::to("policy-v2.apkg").update_from("policy-v1.apkg")
        .inspect_limits(limits).update_policy(policy))?;
    Ok(())
}
```

La política predeterminada bloquea los hallazgos High y Critical. Tras revisar una categoría concreta, puedes elegir de forma deliberada `UpdatePolicy::default().allow(RiskCode::NoteRemoved)`. Esta opción acepta **todos los hallazgos de esa categoría**, conservando su nivel original y sus pruebas. No significa que se hayan aprobado notas concretas de forma individual. Una excepción sin hallazgos coincidentes produce una advertencia; se rechazan los códigos de riesgo desconocidos. No autorices automáticamente todos los códigos que devuelva una comparación.

La ausencia o corrupción de las pruebas de identidad, un espacio de nombres distinto, los datos no válidos de la versión candidata y la superación de los límites de inspección son errores, no categorías de riesgo que se puedan autorizar. Si el análisis no puede terminar, la comparación devuelve un error con las observaciones que haya completado. Establecer una política explícita en una solicitud de creación sin `update_from` es un error de configuración, independientemente del orden de las llamadas a los métodos de configuración.

<!-- section: evidence-and-identity -->
## Pruebas e identidad

El paquete incluye `ankiforge-identity.json` con el formato `ankiforge-identity-v1`. Vincula los ID de modelos y configuraciones, los GUID de notas, las posiciones históricas de campos y plantillas, los números ordinales de tarjetas y máscaras, las revisiones y los recursos multimedia con la colección y los datos reales. La reexportación desde Anki no conserva estas pruebas y no sirve como base de una actualización. Las pruebas ausentes nunca se reconstruyen de forma silenciosa.

Las ediciones de nombres visibles y contenido conservan las claves de identidad. El contenido que no cambia mantiene la revisión de su base; el contenido modificado avanza de revisión. Reordenar campos o plantillas conserva sus correspondencias históricas. Las identidades retiradas permanecen reservadas para poder reutilizarlas al restaurar la misma clave. Una clave de nota no se puede trasladar a otro modelo, y una clave de modelo no puede cambiar entre los tipos normal y Cloze.

<!-- section: client-import-behavior-and-release-review -->
## Importación en el cliente y revisión de versiones

Un APKG es un paquete de distribución; no sincroniza eliminaciones. Omitir notas, campos, plantillas o máscaras no elimina los datos existentes de quienes estudian. Los cambios de esquema de campos o plantillas y los cambios del campo de ordenación tienen riesgo alto porque su comportamiento depende de cómo Anki combine e importe los datos. Los cambios de esquema pueden copiar modelos o actualizar las marcas de tiempo de las notas de destino antes de comparar su contenido.

Prueba la primera importación y las actualizaciones de una colección existente en los clientes que admitas. Comprueba los campos, los GUID, los conjuntos de tarjetas, las ediciones locales y la programación de repaso existente con la configuración de combinación y actualización de notas prevista. La identidad estable, por sí sola, no garantiza que se sobrescriban ediciones locales más recientes ni que se restaure un historial eliminado.

Escribe las versiones candidatas en una ruta distinta de la base archivada. Tras la revisión, distribuye el paquete aceptado y consérvalo como prueba para la siguiente versión. Consulta [las garantías de generación](/docs/build-guarantees/) para conocer cómo se conservan los archivos y cómo se publican.

<a id="build-once-review-then-publish-from-source"></a>

<!-- section: build-once-review-then-publish -->
## Genera una vez, revisa y publica

Esta API está incluida en el paquete público `0.3.1`.

Genera una vez, inspecciona el informe y publica después el mismo archivo candidato. Revisar y ejecutar una segunda generación normal crea otra versión candidata; la publicación preparada mantiene el APKG real vinculado al informe revisado.

<!-- source: anki_forge/examples/docs_publication.rs -->
```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    project.build(BuildOptions::to("spanish-original.apkg"))?;

    let mut next = Project::new("spanish")?.default_deck("Spanish");
    next.add("es:hola", Note::basic("hola", "hello; hi"))?;
    let prepared = next.prepare_publication(
        BuildOptions::to("spanish-reviewed.apkg").update_from("spanish-original.apkg"),
    )?;
    println!("{:?}", prepared.report().comparison());
    if prepared
        .report()
        .comparison()
        .is_some_and(|report| report.policy().allows_publication())
    {
        let output = prepared.publish()?;
        println!("{}", output.artifact().path().display());
    }
    Ok(())
}
```
<!-- /source -->

En el repositorio descargado y fijado a una revisión, guarda el programa completo anterior como `anki_forge/examples/docs_publication.rs` (esta guía añade el ejemplo; no está incluido en `1199196`). Después ejecuta:

```sh
cargo run --locked -p ankiforge --example docs_publication
```

El ejemplo escribe `spanish-original.apkg` y, cuando la política permite publicar, `spanish-reviewed.apkg` en la raíz del repositorio. La preparación no publica el archivo de destino. Revisa los hallazgos y prueba la importación en tu cliente antes de aprobar la distribución; el ejemplo solo publica automáticamente esta sencilla edición de contenido cuando su política lo permite. El benchmark no mide este flujo de trabajo.

Consulta [el ciclo de generación](/docs/build-guarantees/#prepared-publication-from-source), [la API de Rust](/docs/rust-api/#review-once-publish-once), [la API de Node](/docs/node-api/#prepare-review-and-publish) y [la API de Python](/docs/python-api/#prepare-review-and-publish).
