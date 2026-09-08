# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 41.730 | 114.012 | 63.4% | 18.62 | 27.42 |
| basic-audio-unique-v2 | 200 | 57.606 | 129.603 | 55.6% | 21.31 | 27.81 |
| basic-audio-unique-v2 | 500 | 109.344 | 167.553 | 34.7% | 28.34 | 30.67 |
| basic-audio-unique-v2 | 1000 | 196.871 | 229.306 | 14.1% | 40.50 | 34.75 |
| basic-image-unique-v2 | 100 | 44.172 | 120.126 | 63.2% | 19.81 | 26.83 |
| basic-image-unique-v2 | 200 | 66.660 | 137.398 | 51.5% | 22.11 | 27.75 |
| basic-image-unique-v2 | 500 | 127.530 | 187.471 | 32.0% | 28.97 | 30.78 |
| basic-image-unique-v2 | 1000 | 231.843 | 275.912 | 16.0% | 40.48 | 34.55 |
| basic-mixed-shared-v2 | 100 | 35.931 | 110.972 | 67.6% | 19.22 | 26.94 |
| basic-mixed-shared-v2 | 200 | 41.031 | 111.644 | 63.2% | 21.12 | 27.84 |
| basic-mixed-shared-v2 | 500 | 56.327 | 118.161 | 52.3% | 26.42 | 29.25 |
| basic-mixed-shared-v2 | 1000 | 80.671 | 127.257 | 36.6% | 35.69 | 31.81 |
| basic-mixed-text-v1 | 100 | 26.678 | 102.997 | 74.1% | 13.95 | 26.58 |
| basic-mixed-text-v1 | 200 | 30.909 | 105.246 | 70.6% | 15.77 | 27.56 |
| basic-mixed-text-v1 | 500 | 45.540 | 110.775 | 58.9% | 21.22 | 28.73 |
| basic-mixed-text-v1 | 1000 | 68.288 | 121.727 | 43.9% | 30.28 | 31.39 |
| basic-mixed-unique-v2 | 100 | 39.772 | 113.105 | 64.8% | 19.41 | 26.88 |
| basic-mixed-unique-v2 | 200 | 53.281 | 124.111 | 57.1% | 21.91 | 27.39 |
| basic-mixed-unique-v2 | 500 | 99.309 | 158.185 | 37.2% | 28.31 | 29.94 |
| basic-mixed-unique-v2 | 1000 | 171.982 | 214.877 | 20.0% | 39.27 | 33.91 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
