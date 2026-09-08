# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 41.774 | 114.665 | 63.6% | 18.47 | 27.30 |
| basic-audio-unique-v2 | 200 | 57.859 | 128.026 | 54.8% | 21.14 | 27.67 |
| basic-audio-unique-v2 | 500 | 108.161 | 168.101 | 35.7% | 28.25 | 30.86 |
| basic-audio-unique-v2 | 1000 | 194.391 | 231.635 | 16.1% | 39.75 | 35.19 |
| basic-image-unique-v2 | 100 | 45.022 | 118.298 | 61.9% | 19.84 | 26.72 |
| basic-image-unique-v2 | 200 | 67.184 | 136.471 | 50.8% | 22.42 | 27.98 |
| basic-image-unique-v2 | 500 | 128.519 | 188.104 | 31.7% | 28.91 | 30.67 |
| basic-image-unique-v2 | 1000 | 229.907 | 276.149 | 16.7% | 40.16 | 35.02 |
| basic-mixed-shared-v2 | 100 | 36.117 | 111.869 | 67.7% | 19.36 | 26.97 |
| basic-mixed-shared-v2 | 200 | 40.484 | 111.754 | 63.8% | 21.11 | 27.75 |
| basic-mixed-shared-v2 | 500 | 57.391 | 118.484 | 51.6% | 26.36 | 29.33 |
| basic-mixed-shared-v2 | 1000 | 81.510 | 128.995 | 36.8% | 35.58 | 31.95 |
| basic-mixed-text-v1 | 100 | 27.477 | 103.865 | 73.5% | 14.08 | 26.61 |
| basic-mixed-text-v1 | 200 | 31.560 | 104.820 | 69.9% | 15.83 | 27.48 |
| basic-mixed-text-v1 | 500 | 45.300 | 111.986 | 59.5% | 21.27 | 29.11 |
| basic-mixed-text-v1 | 1000 | 67.124 | 120.212 | 44.2% | 30.19 | 31.48 |
| basic-mixed-unique-v2 | 100 | 39.146 | 113.800 | 65.6% | 19.61 | 27.38 |
| basic-mixed-unique-v2 | 200 | 54.696 | 125.057 | 56.3% | 21.52 | 27.72 |
| basic-mixed-unique-v2 | 500 | 97.912 | 158.194 | 38.1% | 28.34 | 29.92 |
| basic-mixed-unique-v2 | 1000 | 171.598 | 215.716 | 20.5% | 38.50 | 33.94 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
