# Phase 4 Product Build CI

Phase 4 CI uses the Product build report as the single machine-readable result.

```bash
contract_tools product-build \
  --manifest contracts/manifest.yaml \
  --product-input project.product.json \
  --apkg-out deck.apkg \
  --compare-to previous.apkg \
  --fail-on high \
  --report-json build-report.json \
  --output contract-json
```

Exit codes:

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | setup or deserialization failure before a BuildReport exists, or stdout rendering failure |
| 2 | policy blocked |
| 3 | BuildReport status invalid, such as an invalid baseline or report-producing validation diagnostics |
| 4 | infrastructure or execution error |

When an invalid baseline also triggers `fail_on`, the top-level `status` is `invalid`, the exit code is `3`, and `policy.status` is `blocked`.

Malformed Product JSON, missing required ProductDocument fields, and manifest setup failures exit `1` before stdout JSON or a report file is produced. Unsupported output modes also exit `1`; when `--report-json` is set and the build reaches report creation, the report file may already exist before stdout rendering fails.

## GitHub Actions Example

```yaml
- name: Download previous APKG
  uses: actions/download-artifact@v4
  with:
    name: previous-apkg
    path: .

- name: Build Anki package
  run: |
    contract_tools product-build \
      --manifest contracts/manifest.yaml \
      --product-input project.product.json \
      --apkg-out deck.apkg \
      --compare-to previous.apkg \
      --fail-on high \
      --report-json build-report.json \
      --output contract-json

- name: Upload build report
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: anki-forge-build-report
    path: build-report.json
    if-no-files-found: ignore
```

## Failure Modes

- Build failure: `status = error`, exit `4`, diagnostics explain the execution failure.
- Invalid baseline: `status = invalid`, `comparison = unavailable`, `risk.findings` includes `RISK.BASELINE_UNAVAILABLE`.
- Pure policy-blocked update: `status = blocked`, exit `2`, and `policy.blocking_findings` lists risk codes at or above the threshold. If invalid or error diagnostics are also present, the top-level status and exit code follow the more severe result.
- Warning-only build: `status = success`, exit `0`, warning diagnostics remain in the report.
