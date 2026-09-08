# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 42.830 | 115.492 | 62.9% | 18.73 | 27.23 |
| basic-audio-unique-v2 | 200 | 59.977 | 130.188 | 53.9% | 21.25 | 27.75 |
| basic-audio-unique-v2 | 500 | 111.070 | 166.951 | 33.5% | 28.16 | 30.59 |
| basic-audio-unique-v2 | 1000 | 198.161 | 232.252 | 14.7% | 39.72 | 35.06 |
| basic-image-unique-v2 | 100 | 45.725 | 124.138 | 63.2% | 19.56 | 27.14 |
| basic-image-unique-v2 | 200 | 66.725 | 140.074 | 52.4% | 22.27 | 28.03 |
| basic-image-unique-v2 | 500 | 129.145 | 189.115 | 31.7% | 28.97 | 30.52 |
| basic-image-unique-v2 | 1000 | 231.697 | 274.949 | 15.7% | 40.11 | 34.73 |
| basic-mixed-shared-v2 | 100 | 37.954 | 111.660 | 66.0% | 19.12 | 26.98 |
| basic-mixed-shared-v2 | 200 | 39.950 | 112.430 | 64.5% | 20.95 | 28.06 |
| basic-mixed-shared-v2 | 500 | 57.136 | 118.883 | 51.9% | 26.33 | 29.05 |
| basic-mixed-shared-v2 | 1000 | 82.075 | 132.430 | 38.0% | 35.20 | 32.61 |
| basic-mixed-text-v1 | 100 | 27.818 | 103.405 | 73.1% | 14.08 | 26.89 |
| basic-mixed-text-v1 | 200 | 32.886 | 105.954 | 69.0% | 15.84 | 27.17 |
| basic-mixed-text-v1 | 500 | 45.731 | 114.239 | 60.0% | 21.27 | 28.97 |
| basic-mixed-text-v1 | 1000 | 68.631 | 122.403 | 43.9% | 30.20 | 31.45 |
| basic-mixed-unique-v2 | 100 | 40.764 | 112.963 | 63.9% | 19.38 | 26.83 |
| basic-mixed-unique-v2 | 200 | 54.260 | 126.495 | 57.1% | 21.72 | 28.08 |
| basic-mixed-unique-v2 | 500 | 98.709 | 162.348 | 39.2% | 28.36 | 30.08 |
| basic-mixed-unique-v2 | 1000 | 173.480 | 216.844 | 20.0% | 38.52 | 34.25 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
