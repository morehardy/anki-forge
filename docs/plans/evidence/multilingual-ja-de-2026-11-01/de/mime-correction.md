# DE-ARTIFACT-001：MIME 通配符渲染修正

独立机器读取者在 `8f2c77e` 的真实prefix HTTP产物中发现：德语Python／Node API段落的裸 `image/*`、`audio/*` 受到相邻句号／分号影响，被Markdown当作强调，HTML文本丢失星号，机器Markdown出现 `image/_`、`audio/_`。这是德语新增段落的渲染问题；逐个读取旧五语真实机器响应后确认它们仍正确保留通配符，最初的“继承问题”判断已纠正。

实际红证据在 `mime-red-prefix.json`／`.log`，输入是已冻结的旧产物而不是合成Markdown。原 `prefix/root-origin-manifest.json`、`engineering/{prefix,root}`、`usage-prefix`、`machine-prefix`、`layout-prefix`及原报告保持原路径原字节。旧prefix使用任务的通过仅对该产物和它们实际读取的任务内容有效，不能授权修正后的候选；旧候选未完成整体发布验收。

修正只在 `docs/de/python-api.md` 与 `docs/de/node-api.md` 各给 `image/*`、`audio/*`、`video/*` 加行内代码标记。两个独立审阅者实读了段落差分；204条源合同中只有这两个德语targetHash变化，其余202条及所有sourceHash／technicalDependencyHash不变。完整原204条报告保留，另通过 `technical-source-review-mime-fixed.json` 和 `linguistic-source-review-mime-fixed.json` 记录两个增量。

扩大同类扫描后，发现 `docs/de/rust-api.md` 也有同样问题，并给三个模式补上行内代码。五个包含 MIME 模式的德语页面（Python／Node／Rust API、media、rust-guide）共 15 处现在均有代码标记。Rust 的单条技术／语言增量报告分别为 `technical-source-review-mime-rust.json` 和 `linguistic-source-review-mime-rust.json`，相对 `6f5f860` 仅该条 targetHash 变化，其余 203 条不变。前两页修正后的中间工程产物保存在 `intermediate-mime-prefix-origin-manifest.json`、`engineering/intermediate-mime-prefix` 和同名 target 目录；它没有完成构建后独立验收，不能作为最终候选。

最终报告路径在重建前固定为 `usage-tasks-final-{prefix,root}.json` 和 `machine-read-final-{prefix,root}.json`。最终构建、复建、八项工程门禁和全部当前语言的真实任务将全部重新执行；新产物、manifest、捕获及收据使用 `final-prefix/final-root` 标签。旧证据不会靠改 inputSha 或 configuration 冒充新产物。`verify-mime-literals.mjs` 核验五页实际 HTML 文本／代码节点和机器 Markdown 的 15 个模式，并通过本地服务器读取这五页的 HTML 与 Markdown、保存响应字节与头信息。最终是否关闭以新产物绿证据及独立 HTTP 审读为准。
