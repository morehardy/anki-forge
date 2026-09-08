# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 38.233 | 100.365 | 61.9% | 17.95 | 27.20 |
| basic-audio-unique-v2 | 200 | 51.678 | 109.719 | 52.9% | 20.61 | 28.67 |
| basic-audio-unique-v2 | 500 | 98.147 | 143.319 | 31.5% | 27.72 | 31.22 |
| basic-audio-unique-v2 | 1000 | 169.540 | 189.510 | 10.5% | 39.91 | 35.56 |
| basic-image-unique-v2 | 100 | 45.068 | 110.736 | 59.3% | 18.80 | 28.11 |
| basic-image-unique-v2 | 200 | 63.401 | 119.906 | 47.1% | 21.17 | 29.06 |
| basic-image-unique-v2 | 500 | 128.728 | 159.820 | 19.5% | 28.09 | 31.52 |
| basic-image-unique-v2 | 1000 | 220.666 | 223.654 | 1.3% | 39.83 | 36.22 |
| basic-mixed-shared-v2 | 100 | 32.823 | 95.742 | 65.7% | 18.53 | 28.23 |
| basic-mixed-shared-v2 | 200 | 36.897 | 94.242 | 60.8% | 20.36 | 28.20 |
| basic-mixed-shared-v2 | 500 | 49.001 | 102.829 | 52.3% | 25.80 | 29.78 |
| basic-mixed-shared-v2 | 1000 | 69.876 | 114.279 | 38.9% | 35.23 | 32.33 |
| basic-mixed-text-v1 | 100 | 24.364 | 89.371 | 72.7% | 14.06 | 27.44 |
| basic-mixed-text-v1 | 200 | 28.029 | 90.948 | 69.2% | 15.84 | 28.38 |
| basic-mixed-text-v1 | 500 | 38.359 | 96.275 | 60.2% | 21.38 | 29.25 |
| basic-mixed-text-v1 | 1000 | 58.950 | 107.240 | 45.0% | 30.38 | 32.08 |
| basic-mixed-unique-v2 | 100 | 36.053 | 101.577 | 64.5% | 18.64 | 28.11 |
| basic-mixed-unique-v2 | 200 | 49.261 | 104.288 | 52.8% | 21.00 | 28.34 |
| basic-mixed-unique-v2 | 500 | 88.046 | 130.045 | 32.3% | 27.50 | 30.41 |
| basic-mixed-unique-v2 | 1000 | 163.697 | 193.102 | 15.2% | 38.39 | 34.55 |

Cells meeting the proposed 30% reduction: 16/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
