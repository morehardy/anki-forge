# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v1 | 100 | 33.650 | 102.208 | 67.1% | 18.05 | 27.02 |
| basic-audio-unique-v1 | 200 | 49.067 | 118.468 | 58.6% | 20.53 | 27.86 |
| basic-audio-unique-v1 | 500 | 86.402 | 169.539 | 49.0% | 28.05 | 31.17 |
| basic-audio-unique-v1 | 1000 | 156.783 | 250.682 | 37.5% | 40.08 | 36.09 |
| basic-image-unique-v1 | 100 | 40.815 | 105.329 | 61.2% | 18.91 | 26.86 |
| basic-image-unique-v1 | 200 | 56.521 | 122.906 | 54.0% | 21.19 | 27.80 |
| basic-image-unique-v1 | 500 | 109.251 | 183.855 | 40.6% | 27.86 | 30.88 |
| basic-image-unique-v1 | 1000 | 197.606 | 288.048 | 31.4% | 40.50 | 35.36 |
| basic-mixed-shared-v1 | 100 | 34.550 | 98.929 | 65.1% | 18.81 | 26.75 |
| basic-mixed-shared-v1 | 200 | 38.248 | 98.720 | 61.3% | 20.77 | 27.30 |
| basic-mixed-shared-v1 | 500 | 49.556 | 104.535 | 52.6% | 26.11 | 29.16 |
| basic-mixed-shared-v1 | 1000 | 69.373 | 111.767 | 37.9% | 35.48 | 32.05 |
| basic-mixed-text-v1 | 100 | 25.355 | 86.993 | 70.9% | 14.12 | 26.53 |
| basic-mixed-text-v1 | 200 | 26.198 | 88.727 | 70.5% | 15.97 | 27.19 |
| basic-mixed-text-v1 | 500 | 39.453 | 93.004 | 57.6% | 21.28 | 28.41 |
| basic-mixed-text-v1 | 1000 | 57.293 | 99.578 | 42.5% | 30.33 | 31.23 |
| basic-mixed-unique-v1 | 100 | 34.674 | 98.425 | 64.8% | 18.83 | 27.31 |
| basic-mixed-unique-v1 | 200 | 46.275 | 113.344 | 59.2% | 20.97 | 27.72 |
| basic-mixed-unique-v1 | 500 | 83.318 | 151.239 | 44.9% | 27.64 | 30.38 |
| basic-mixed-unique-v1 | 1000 | 141.769 | 218.251 | 35.0% | 38.95 | 34.81 |

Cells meeting the proposed 30% reduction: 20/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
