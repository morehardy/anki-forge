# 首次日语机器文本预览失败

首次 `run-artifact.py ja prefix preview` 在 `generate:machine` 失败。下列是当时工具返回的原始错误摘录；后一次成功预览复用了日志路径，本文件是对已观察控制台输出的记录，不冒称保存了完整原始日志。

```text
Error: ja: the Optional protocol heading must not be translated
    at renderLlms (.../website/scripts/machine-content.mjs:162:84)
    at generateMachineContent (.../website/scripts/machine-content.mjs:246:55)
```

作者词典当时将 `machine.optional` 写为 `任意のリソース`。集成人将该键恢复为协议要求的字面量 `Optional`，未修改验证器。后续完整预览和正式双配置构建均使用修正后的词典；独立技术和表达源审也实际确认了这一协议边界。德语作者包原本使用 `Optional`，无需改动。
