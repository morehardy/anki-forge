# 日德样稿与八语菜单：执行前固定的探针意图

2026-11-01，独立审阅者 `/root/baseline_acceptance`。本文件先于观察结果写入；不根据结果添加同义词、重排目标或降低标准。

1. 使用当前 `website/src/lib/search.mjs` 的公开排序函数，以每语现有五类样稿作为小型语料。标题取真实 H1，摘要取第一段，正文取真实样稿；首页取实际 home UI 文案。不补造正式 metadata／terms，不构建 Pagefind，也不把传入正文叫 Pagefind 输出。记录两种模式：纯词法；仅用 brief 术语构造的独立小词表（note/card/deck/namespace/baseline/API 的词面，不加入查询变体或结果专用词）。自然任务句的预期目标提前固定，以目标出现在前三项作为诊断观察，不用它代替30题发布门槛。
2. 日语四个任务与一个写法对照：`Rustで最初のデッキを生成したい` → quickstart；`JSONから画像と音声付きデッキを生成したい` → json-to-anki；`配布済みデッキを元のAPKGから更新したい` → updates；`PythonでBuildOptions.temporaryとcloseの寿命を知りたい` → python-api；`でっきをこうしんしたい` 与 `デッキを更新したい` → updates。观察无空格、汉字/假名和拉丁 API 混排，不假定假名同义词已存在。
3. 德语四个任务：`Wie erzeuge ich mit Rust meinen ersten Stapel?` → quickstart；`Stapelaktualisierung mit dem ursprünglichen APKG` → updates；`Wie behalte ich stabile Notizschlüssel bei einer Aktualisierung?` → updates；`Wie verwalte ich die Lebensdauer von BuildOptions.temporary in Python?` → python-api。观察复合词和自然句；不假定英语 stemming 能处理德语。
4. 每语一个无关负例 `zzzxqv 987654321`，应无结果，避免仅数字或未知输入制造命中。保留 Intl.Segmenter 实际词段以解释检索，不修改模块内部函数。
5. 将已冻结 prefix artifact 复制到 target 的隔离目录。只修改复制件：采用当前语言菜单 DOM/CSS，按 configuredLocales 顺序补足八语（当前语+七个目标），真实日德 quickstart/API Markdown 和选取的真实 UI 文案。href 指向本地 fixture 页面，明确不是已发布语言路线。
6. 菜单在 320×360 和 390×360 CSS px，分别100%/200%文字大小，JavaScript开启/关闭共8种条件，另以德语当前项补做320px/200%无JS。200%通过逐元素将实际计算字号乘二，只改字号、不缩放视口或所有间距，不能等同所有浏览器原生text-only zoom。检查原生details键盘Enter打开、七目标逐项Tab与Enter导航、无JS同样可达；记录每项视口矩形、菜单滚动高度、页面滚动和点击点可见性。目标是无需横向滚动即可读到且能聚焦/激活全部候选项。若必须滚动，必须有实际可用滚动路径，不能只因DOM存在就通过。
7. 直接查看少量真实日德文字和菜单截图。只报告此fixture中观察到的布局；完整日德页面、真实Pagefind、社交图、字体覆盖、跨浏览器、移动设备触摸和正式上线均未验收。发现产品问题先报告父任务，不修改共享源码或dist。
