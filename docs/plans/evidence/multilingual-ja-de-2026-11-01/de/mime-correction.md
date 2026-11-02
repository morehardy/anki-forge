# DE-ARTIFACT-001：MIME 通配符渲染修正

独立机器读取者在 `8f2c77e` 的真实prefix HTTP产物中发现：德语Python／Node API段落的裸 `image/*`、`audio/*` 受到相邻句号／分号影响，被Markdown当作强调，HTML文本丢失星号，机器Markdown出现 `image/_`、`audio/_`。这是德语新增段落的渲染问题；逐个读取旧五语真实机器响应后确认它们仍正确保留通配符，最初的“继承问题”判断已纠正。

实际红证据在 `mime-red-prefix.json`／`.log`，输入是已冻结的旧产物而不是合成Markdown。原 `prefix/root-origin-manifest.json`、`engineering/{prefix,root}`、`usage-prefix`、`machine-prefix`、`layout-prefix`及原报告保持原路径原字节。旧prefix使用任务的通过仅对该产物和它们实际读取的任务内容有效，不能授权修正后的候选；旧候选未完成整体发布验收。

修正只在 `docs/de/python-api.md` 与 `docs/de/node-api.md` 各给 `image/*`、`audio/*`、`video/*` 加行内代码标记。两个独立审阅者实读了段落差分；204条源合同中只有这两个德语targetHash变化，其余202条及所有sourceHash／technicalDependencyHash不变。完整原204条报告保留，另通过 `technical-source-review-mime-fixed.json` 和 `linguistic-source-review-mime-fixed.json` 记录两个增量。

最终报告路径在重建前固定为 `usage-tasks-final-{prefix,root}.json` 和 `machine-read-final-{prefix,root}.json`。最终构建、复建、八项工程门禁和全部当前语言的真实任务将全部重新执行；新产物、manifest、捕获及收据使用 `final-prefix/final-root` 标签。旧证据不会靠改inputSha或configuration冒充新产物。`verify-mime-literals.mjs` 直接核验两页实际HTML文本／代码节点和机器Markdown中的六个模式，最终是否关闭以新产物的绿证据及独立HTTP审读为准。
