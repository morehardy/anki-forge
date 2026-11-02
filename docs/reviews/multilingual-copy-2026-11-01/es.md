# 西班牙语文案审阅

整体自然，nota、tarjeta、mazo 区分稳定，apuntes 与 Anki 的 nota 也有区分。主要问题是个别句法、搜索范围、线程术语和性能标签中的歧义，不需要整体重译。

保留 mazo、nota、tarjeta、tipo de nota；保留 “Ideas y guías para crear mazos.”。无需为了逐词对应英文而恢复博客双关。

P2 表示应优先处理的事实范围或理解歧义；P3 表示可选的自然度、清晰度和术语优化。编号按位置统计，同一原因可能影响多处文字。本页建议已应用到产品源文件，并补齐相关术语的一致性。表中原文和行号保留审阅时快照，便于追溯；不作为正式发布的独立审阅凭据。

## 界面文案

| 编号与位置 | 原文 | 建议替换 | 问题 |
| --- | --- | --- | --- |
| es-01 P3<br>[home.choose](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:9) | Elige tu lenguaje. | Elige tu lenguaje de programación. | 明确是编程语言，与站点语言切换区分。 |
| es-02 P3<br>[ai.language](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:144) | Lenguaje de tu aplicación | Lenguaje de programación | 选项卡无障碍标签可直接说选择对象。 |
| es-03 P3<br>[ai.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:137) | Elige un lenguaje, describe la tarea y pega las instrucciones en tu asistente de programación con IA. | Elige un lenguaje de programación, describe la tarea y pega las instrucciones en tu asistente de programación con IA. | 语言指代具体化；其余表达自然，可以保留。 |
| es-04 P3<br>[ai.request](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:138) | ¿Qué debe hacer el mazo? | ¿Qué tipo de mazo quieres crear? | 比“牌组应该做什么”更符合生成内容的任务。 |
| es-05 P3<br>[ai.noScript](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:143) | Copia una plantilla de lenguaje de abajo y sustituye el texto de la tarea. Los comandos de instalación y la Skill también están disponibles sin JavaScript. | Copia la plantilla correspondiente a tu lenguaje de programación y sustituye el marcador de la tarea. Los comandos de instalación y la Skill también están disponibles sin JavaScript. | plantilla de lenguaje 是英文 language template 的生硬组合；说明模板与编程语言的对应关系。 |
| es-06 P3<br>[card.concert](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:66) | Un segundo de un la de afinación. | Un la de afinación que dura un segundo. | 原句连续两个 un 且 la 容易读成冠词；先点明是一个音。 |
| es-07 P3<br>[update.comparison](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:82) | Comparación real | Resultado de la comparación | Comparación real 的 real 没有说明作用。 |
| es-08 P3<br>[update.none](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:83) | Sin hallazgos | No se detectaron cambios | Sin hallazgos 适合审计报告，交互演示用“未检测到变化”更直接。 |
| es-09 P3<br>[examples.updateDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:129) | La segunda generación cambia la respuesta de «hello» a «hello; hi». Se compara con el primer paquete y comprueba que se conserva la identidad de la nota. | La segunda versión cambia la respuesta de «hello» a «hello; hi». El programa la compara con el primer paquete y comprueba que el ID de la nota no cambia. | 第二次生成被拟人化成“检查”的主语；改成版本与执行程序，并明确 ID。 |
| es-10 P3<br>[examples.occlusion](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:135) | Un rectángulo con clave oculta el núcleo en un dibujo de una célula en corte. El mazo descargable usa la misma imagen y máscara que esta vista previa. | Una máscara rectangular con una clave estable oculta el núcleo en un dibujo en corte de una célula. El mazo descargable usa la misma imagen y máscara que esta vista previa. | rectángulo con clave 不够明确；这里是带稳定键的矩形遮罩。 |
| es-11 P3<br>[performance.title](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:165) | Medimos la exportación. | Tiempos de exportación | Medimos la exportación 偏口号，标题应说明数据是什么。 |
| es-12 P3<br>[performance.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:172) | Cada carga contiene {notes} notas Basic: solo texto, archivos multimedia únicos o recursos mixtos compartidos. Rust necesitó entre un {minimum} y un {maximum} % menos de tiempo de exportación que genanki. | Cada prueba usa {notes} notas Basic. Las cinco pruebas abarcan notas de solo texto, archivos multimedia distintos y recursos compartidos entre notas. En estas pruebas, la API de Rust de Anki Forge tardó entre un {minimum} y un {maximum} % menos en exportar que genanki. | 明确不同场景属于五组测试，并避免把整个 Rust 语言作为性能主语。 |
| es-13 P3<br>[performance.basic-image-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:177) | Imágenes únicas | Una imagen distinta por nota | únicas 未说明在哪个范围唯一；这里每条笔记使用不同图片。 |
| es-14 P2<br>[performance.basic-audio-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/es.json:178) | Audio único | Un audio distinto por nota | Audio único 容易理解为只有一个音频文件，与不共享的含义相反。 |

## 正文与元数据

### es-15 P2

[website/src/content/i18n/es.json](/Users/hp/Desktop/2026/anki-forge/website/src/content/i18n/es.json:5)

搜索范围是全站，原文未表达 en este sitio。

原文片段：

> "pagefind.search_label": "Buscar página"

建议替换：

> "pagefind.search_label": "Buscar en este sitio"

### es-16 P3

[docs/es/updates.md](/Users/hp/Desktop/2026/anki-forge/docs/es/updates.md:1)

publicación 作为被更新和分发的对象含糊；直接说 mazo。 此项是仓库源文标题；生成的网站 H1 由 site-model.mjs 提供，不应把它报成网页标题错误。

原文片段：

> # Actualiza y distribuye una publicación

建议替换：

> # Actualiza y distribuye un mazo

### es-17 P3

[docs/es/anki-forge-vs-genanki.md](/Users/hp/Desktop/2026/anki-forge/docs/es/anki-forge-vs-genanki.md:3)

原句把诊断、比较、策略审查全接到“项目保留”后面，语义并列不当。

原文片段：

> necesitas que el proyecto conserve los recursos multimedia, diagnósticos estructurados, comparaciones y una revisión de políticas antes de publicar actualizaciones.

建议替换：

> necesitas conservar los datos multimedia sin depender de los archivos originales, obtener diagnósticos estructurados y revisar las comparaciones y la política de riesgos antes de publicar actualizaciones.

### es-18 P2

[docs/es/build-guarantees.md](/Users/hp/Desktop/2026/anki-forge/docs/es/build-guarantees.md:54)

workers 指后台工作线程，trabajadores 会让人想到工作人员；应明确技术对象。

原文片段：

> el funcionamiento de los trabajadores.

建议替换：

> el funcionamiento de los hilos de trabajo.

### es-19 P2

[docs/es/updates.md](/Users/hp/Desktop/2026/anki-forge/docs/es/updates.md:71)

上限本身不是错误，超过上限才是错误；原文遗漏触发条件。

原文片段：

> los datos no válidos de la versión candidata y los límites de inspección son errores

建议替换：

> los datos no válidos de la versión candidata y la superación de los límites de inspección son errores

### es-20 P3

[website/src/content/blog/es/stable-ids-and-deck-updates.mdx](/Users/hp/Desktop/2026/anki-forge/website/src/content/blog/es/stable-ids-and-deck-updates.mdx:38)

generation 翻为 generación 虽然可懂，但此标题整体很机械；直接写操作。

原文片段：

> ## Dar un paquete de referencia a la siguiente generación

建议替换：

> ## Usar el paquete anterior como referencia
