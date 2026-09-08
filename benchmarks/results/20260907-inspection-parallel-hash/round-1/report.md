# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 37.877 | 106.200 | 64.3% | 18.39 | 28.48 |
| basic-audio-unique-v2 | 200 | 52.930 | 115.742 | 54.3% | 21.12 | 29.11 |
| basic-audio-unique-v2 | 500 | 94.882 | 149.791 | 36.7% | 28.39 | 31.86 |
| basic-audio-unique-v2 | 1000 | 169.726 | 204.797 | 17.1% | 40.47 | 36.14 |
| basic-image-unique-v2 | 100 | 40.953 | 109.210 | 62.5% | 18.88 | 28.61 |
| basic-image-unique-v2 | 200 | 60.599 | 124.901 | 51.5% | 21.47 | 28.75 |
| basic-image-unique-v2 | 500 | 116.861 | 168.108 | 30.5% | 28.50 | 31.95 |
| basic-image-unique-v2 | 1000 | 210.979 | 243.466 | 13.3% | 40.38 | 36.05 |
| basic-mixed-shared-v2 | 100 | 33.773 | 101.071 | 66.6% | 18.66 | 28.44 |
| basic-mixed-shared-v2 | 200 | 35.656 | 103.870 | 65.7% | 20.62 | 29.12 |
| basic-mixed-shared-v2 | 500 | 50.049 | 108.251 | 53.8% | 26.08 | 30.56 |
| basic-mixed-shared-v2 | 1000 | 72.465 | 117.791 | 38.5% | 35.27 | 33.16 |
| basic-mixed-text-v1 | 100 | 24.599 | 95.461 | 74.2% | 14.27 | 28.14 |
| basic-mixed-text-v1 | 200 | 28.597 | 97.145 | 70.6% | 16.09 | 28.59 |
| basic-mixed-text-v1 | 500 | 40.811 | 103.400 | 60.5% | 21.86 | 29.86 |
| basic-mixed-text-v1 | 1000 | 61.809 | 111.694 | 44.7% | 30.59 | 32.53 |
| basic-mixed-unique-v2 | 100 | 34.684 | 103.641 | 66.5% | 18.86 | 28.30 |
| basic-mixed-unique-v2 | 200 | 48.800 | 114.010 | 57.2% | 21.25 | 29.08 |
| basic-mixed-unique-v2 | 500 | 88.231 | 143.325 | 38.4% | 27.77 | 31.08 |
| basic-mixed-unique-v2 | 1000 | 153.674 | 192.179 | 20.0% | 38.97 | 34.95 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
