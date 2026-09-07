# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 38.144 | 103.643 | 63.2% | 17.94 | 27.31 |
| basic-audio-unique-v2 | 200 | 56.433 | 124.391 | 54.6% | 20.44 | 27.78 |
| basic-audio-unique-v2 | 500 | 100.017 | 171.760 | 41.8% | 27.69 | 31.17 |
| basic-audio-unique-v2 | 1000 | 189.870 | 266.781 | 28.8% | 40.03 | 35.23 |
| basic-image-unique-v2 | 100 | 43.811 | 111.043 | 60.5% | 18.72 | 27.45 |
| basic-image-unique-v2 | 200 | 65.018 | 129.702 | 49.9% | 21.12 | 28.22 |
| basic-image-unique-v2 | 500 | 126.500 | 192.252 | 34.2% | 28.11 | 31.00 |
| basic-image-unique-v2 | 1000 | 236.810 | 299.155 | 20.8% | 40.00 | 35.11 |
| basic-mixed-shared-v2 | 100 | 33.981 | 99.023 | 65.7% | 18.55 | 27.50 |
| basic-mixed-shared-v2 | 200 | 36.343 | 99.701 | 63.5% | 20.39 | 27.98 |
| basic-mixed-shared-v2 | 500 | 53.808 | 103.605 | 48.1% | 25.94 | 29.25 |
| basic-mixed-shared-v2 | 1000 | 72.819 | 117.566 | 38.1% | 34.97 | 32.81 |
| basic-mixed-text-v1 | 100 | 24.710 | 93.990 | 73.7% | 14.11 | 27.28 |
| basic-mixed-text-v1 | 200 | 29.764 | 93.445 | 68.1% | 15.80 | 27.66 |
| basic-mixed-text-v1 | 500 | 42.284 | 95.413 | 55.7% | 21.39 | 29.83 |
| basic-mixed-text-v1 | 1000 | 60.474 | 104.258 | 42.0% | 30.38 | 31.84 |
| basic-mixed-unique-v2 | 100 | 37.428 | 97.922 | 61.8% | 18.66 | 27.20 |
| basic-mixed-unique-v2 | 200 | 51.511 | 113.489 | 54.6% | 20.94 | 27.73 |
| basic-mixed-unique-v2 | 500 | 97.829 | 159.587 | 38.7% | 27.36 | 30.16 |
| basic-mixed-unique-v2 | 1000 | 164.019 | 233.197 | 29.7% | 38.44 | 34.33 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
