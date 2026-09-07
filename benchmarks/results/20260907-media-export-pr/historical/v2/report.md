# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 35.553 | 102.309 | 65.2% | 17.94 | 26.78 |
| basic-audio-unique-v2 | 200 | 48.544 | 119.290 | 59.3% | 20.59 | 28.22 |
| basic-audio-unique-v2 | 500 | 86.569 | 170.256 | 49.2% | 27.66 | 30.81 |
| basic-audio-unique-v2 | 1000 | 146.673 | 251.673 | 41.7% | 39.61 | 35.08 |
| basic-image-unique-v2 | 100 | 39.507 | 105.840 | 62.7% | 18.72 | 26.91 |
| basic-image-unique-v2 | 200 | 57.874 | 125.243 | 53.8% | 21.20 | 28.23 |
| basic-image-unique-v2 | 500 | 113.196 | 183.836 | 38.4% | 28.11 | 30.73 |
| basic-image-unique-v2 | 1000 | 198.130 | 285.012 | 30.5% | 40.12 | 34.91 |
| basic-mixed-shared-v2 | 100 | 32.296 | 94.701 | 65.9% | 18.61 | 26.86 |
| basic-mixed-shared-v2 | 200 | 34.719 | 95.965 | 63.8% | 20.55 | 27.84 |
| basic-mixed-shared-v2 | 500 | 48.291 | 101.318 | 52.3% | 25.88 | 29.78 |
| basic-mixed-shared-v2 | 1000 | 67.270 | 108.899 | 38.2% | 35.27 | 32.22 |
| basic-mixed-text-v1 | 100 | 24.346 | 86.779 | 71.9% | 14.09 | 26.69 |
| basic-mixed-text-v1 | 200 | 26.568 | 88.028 | 69.8% | 15.94 | 27.02 |
| basic-mixed-text-v1 | 500 | 37.593 | 92.554 | 59.4% | 21.31 | 28.33 |
| basic-mixed-text-v1 | 1000 | 56.150 | 100.412 | 44.1% | 30.45 | 31.19 |
| basic-mixed-unique-v2 | 100 | 33.981 | 99.463 | 65.8% | 18.66 | 27.34 |
| basic-mixed-unique-v2 | 200 | 46.379 | 113.970 | 59.3% | 21.03 | 27.69 |
| basic-mixed-unique-v2 | 500 | 81.539 | 152.930 | 46.7% | 27.45 | 30.28 |
| basic-mixed-unique-v2 | 1000 | 141.650 | 220.897 | 35.9% | 38.69 | 33.83 |

Cells meeting the proposed 30% reduction: 20/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
