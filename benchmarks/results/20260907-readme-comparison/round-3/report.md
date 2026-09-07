# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 36.273 | 95.329 | 61.9% | 18.05 | 26.97 |
| basic-audio-unique-v2 | 200 | 51.552 | 105.283 | 51.0% | 20.50 | 28.38 |
| basic-audio-unique-v2 | 500 | 95.650 | 136.643 | 30.0% | 27.84 | 30.91 |
| basic-audio-unique-v2 | 1000 | 167.673 | 187.287 | 10.5% | 39.94 | 35.11 |
| basic-image-unique-v2 | 100 | 40.754 | 96.561 | 57.8% | 18.72 | 27.69 |
| basic-image-unique-v2 | 200 | 61.630 | 109.953 | 43.9% | 21.23 | 29.11 |
| basic-image-unique-v2 | 500 | 120.793 | 154.782 | 22.0% | 28.08 | 31.17 |
| basic-image-unique-v2 | 1000 | 222.420 | 222.726 | 0.1% | 39.80 | 35.27 |
| basic-mixed-shared-v2 | 100 | 31.687 | 90.929 | 65.2% | 18.39 | 27.72 |
| basic-mixed-shared-v2 | 200 | 34.830 | 93.767 | 62.9% | 20.19 | 28.44 |
| basic-mixed-shared-v2 | 500 | 49.445 | 98.852 | 50.0% | 25.95 | 29.48 |
| basic-mixed-shared-v2 | 1000 | 68.257 | 105.606 | 35.4% | 35.28 | 32.38 |
| basic-mixed-text-v1 | 100 | 23.219 | 83.427 | 72.2% | 14.00 | 27.09 |
| basic-mixed-text-v1 | 200 | 26.145 | 86.100 | 69.6% | 15.84 | 27.31 |
| basic-mixed-text-v1 | 500 | 37.637 | 92.012 | 59.1% | 21.56 | 29.53 |
| basic-mixed-text-v1 | 1000 | 55.814 | 98.171 | 43.1% | 30.44 | 31.77 |
| basic-mixed-unique-v2 | 100 | 35.070 | 94.557 | 62.9% | 18.69 | 27.28 |
| basic-mixed-unique-v2 | 200 | 47.030 | 100.658 | 53.3% | 20.83 | 27.95 |
| basic-mixed-unique-v2 | 500 | 88.048 | 126.921 | 30.6% | 27.42 | 30.45 |
| basic-mixed-unique-v2 | 1000 | 157.907 | 176.113 | 10.3% | 38.33 | 34.72 |

Cells meeting the proposed 30% reduction: 15/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
