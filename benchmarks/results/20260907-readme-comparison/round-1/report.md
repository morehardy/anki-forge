# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 34.712 | 91.552 | 62.1% | 17.89 | 27.20 |
| basic-audio-unique-v2 | 200 | 50.102 | 106.498 | 53.0% | 20.56 | 29.19 |
| basic-audio-unique-v2 | 500 | 92.689 | 137.654 | 32.7% | 27.92 | 32.09 |
| basic-audio-unique-v2 | 1000 | 167.702 | 187.897 | 10.7% | 39.81 | 36.09 |
| basic-image-unique-v2 | 100 | 41.811 | 97.636 | 57.2% | 18.75 | 27.86 |
| basic-image-unique-v2 | 200 | 60.357 | 112.133 | 46.2% | 21.20 | 27.77 |
| basic-image-unique-v2 | 500 | 118.881 | 149.671 | 20.6% | 27.86 | 30.81 |
| basic-image-unique-v2 | 1000 | 222.587 | 222.473 | -0.1% | 39.67 | 35.50 |
| basic-mixed-shared-v2 | 100 | 30.735 | 89.243 | 65.6% | 18.53 | 28.19 |
| basic-mixed-shared-v2 | 200 | 34.405 | 89.371 | 61.5% | 20.38 | 27.83 |
| basic-mixed-shared-v2 | 500 | 48.276 | 98.477 | 51.0% | 25.95 | 29.88 |
| basic-mixed-shared-v2 | 1000 | 67.168 | 104.484 | 35.7% | 35.25 | 32.98 |
| basic-mixed-text-v1 | 100 | 22.468 | 85.133 | 73.6% | 14.08 | 26.77 |
| basic-mixed-text-v1 | 200 | 26.322 | 85.865 | 69.3% | 15.83 | 27.72 |
| basic-mixed-text-v1 | 500 | 37.191 | 91.772 | 59.5% | 21.33 | 28.73 |
| basic-mixed-text-v1 | 1000 | 55.549 | 95.377 | 41.8% | 30.48 | 32.31 |
| basic-mixed-unique-v2 | 100 | 33.965 | 94.498 | 64.1% | 18.50 | 27.23 |
| basic-mixed-unique-v2 | 200 | 47.900 | 104.095 | 54.0% | 20.91 | 28.98 |
| basic-mixed-unique-v2 | 500 | 89.727 | 127.244 | 29.5% | 27.41 | 30.58 |
| basic-mixed-unique-v2 | 1000 | 155.313 | 174.764 | 11.1% | 38.33 | 33.53 |

Cells meeting the proposed 30% reduction: 15/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
