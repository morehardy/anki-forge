# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 36.819 | 104.362 | 64.7% | 18.31 | 28.16 |
| basic-audio-unique-v2 | 200 | 52.029 | 116.474 | 55.3% | 21.03 | 29.05 |
| basic-audio-unique-v2 | 500 | 96.584 | 150.142 | 35.7% | 28.42 | 31.75 |
| basic-audio-unique-v2 | 1000 | 167.125 | 204.392 | 18.2% | 40.36 | 35.84 |
| basic-image-unique-v2 | 100 | 40.513 | 108.660 | 62.7% | 18.94 | 28.22 |
| basic-image-unique-v2 | 200 | 60.142 | 123.830 | 51.4% | 21.50 | 28.88 |
| basic-image-unique-v2 | 500 | 115.363 | 167.675 | 31.2% | 28.47 | 31.47 |
| basic-image-unique-v2 | 1000 | 210.560 | 242.407 | 13.1% | 40.06 | 35.59 |
| basic-mixed-shared-v2 | 100 | 33.068 | 100.976 | 67.3% | 18.67 | 28.06 |
| basic-mixed-shared-v2 | 200 | 36.617 | 103.467 | 64.6% | 20.55 | 28.50 |
| basic-mixed-shared-v2 | 500 | 50.751 | 108.141 | 53.1% | 26.11 | 30.17 |
| basic-mixed-shared-v2 | 1000 | 73.710 | 116.465 | 36.7% | 35.31 | 32.86 |
| basic-mixed-text-v1 | 100 | 25.305 | 94.546 | 73.2% | 14.19 | 27.73 |
| basic-mixed-text-v1 | 200 | 29.358 | 97.276 | 69.8% | 16.03 | 28.27 |
| basic-mixed-text-v1 | 500 | 40.609 | 101.733 | 60.1% | 21.73 | 29.91 |
| basic-mixed-text-v1 | 1000 | 61.884 | 110.234 | 43.9% | 30.66 | 32.23 |
| basic-mixed-unique-v2 | 100 | 36.004 | 103.993 | 65.4% | 18.73 | 28.14 |
| basic-mixed-unique-v2 | 200 | 48.614 | 113.922 | 57.3% | 21.34 | 28.94 |
| basic-mixed-unique-v2 | 500 | 87.805 | 143.130 | 38.7% | 27.89 | 31.22 |
| basic-mixed-unique-v2 | 1000 | 153.013 | 190.280 | 19.6% | 38.95 | 34.73 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
