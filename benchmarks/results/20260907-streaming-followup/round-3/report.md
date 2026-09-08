# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 41.663 | 115.158 | 63.8% | 18.89 | 27.36 |
| basic-audio-unique-v2 | 200 | 58.806 | 127.195 | 53.8% | 21.03 | 28.05 |
| basic-audio-unique-v2 | 500 | 109.623 | 166.153 | 34.0% | 28.58 | 30.44 |
| basic-audio-unique-v2 | 1000 | 196.666 | 232.677 | 15.5% | 40.19 | 35.02 |
| basic-image-unique-v2 | 100 | 45.433 | 119.816 | 62.1% | 19.89 | 27.34 |
| basic-image-unique-v2 | 200 | 66.589 | 137.853 | 51.7% | 22.27 | 27.66 |
| basic-image-unique-v2 | 500 | 128.303 | 187.596 | 31.6% | 29.05 | 30.48 |
| basic-image-unique-v2 | 1000 | 233.010 | 274.502 | 15.1% | 40.58 | 34.86 |
| basic-mixed-shared-v2 | 100 | 37.383 | 110.687 | 66.2% | 19.34 | 26.86 |
| basic-mixed-shared-v2 | 200 | 41.596 | 111.593 | 62.7% | 21.11 | 27.22 |
| basic-mixed-shared-v2 | 500 | 56.605 | 119.364 | 52.6% | 26.62 | 29.39 |
| basic-mixed-shared-v2 | 1000 | 81.531 | 129.340 | 37.0% | 35.73 | 31.92 |
| basic-mixed-text-v1 | 100 | 27.630 | 102.761 | 73.1% | 13.92 | 26.61 |
| basic-mixed-text-v1 | 200 | 30.438 | 104.648 | 70.9% | 15.80 | 27.42 |
| basic-mixed-text-v1 | 500 | 44.954 | 111.104 | 59.5% | 21.20 | 28.98 |
| basic-mixed-text-v1 | 1000 | 67.065 | 120.102 | 44.2% | 30.16 | 31.19 |
| basic-mixed-unique-v2 | 100 | 39.359 | 112.498 | 65.0% | 19.42 | 27.34 |
| basic-mixed-unique-v2 | 200 | 54.432 | 124.476 | 56.3% | 21.88 | 27.86 |
| basic-mixed-unique-v2 | 500 | 98.441 | 160.124 | 38.5% | 28.38 | 30.16 |
| basic-mixed-unique-v2 | 1000 | 172.676 | 215.594 | 19.9% | 39.16 | 33.92 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
