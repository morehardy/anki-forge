# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 43.710 | 115.243 | 62.1% | 18.50 | 26.75 |
| basic-audio-unique-v2 | 200 | 60.422 | 128.100 | 52.8% | 20.94 | 28.86 |
| basic-audio-unique-v2 | 500 | 109.844 | 167.774 | 34.5% | 28.28 | 30.34 |
| basic-audio-unique-v2 | 1000 | 195.837 | 231.950 | 15.6% | 39.84 | 35.12 |
| basic-image-unique-v2 | 100 | 44.716 | 119.168 | 62.5% | 19.78 | 27.08 |
| basic-image-unique-v2 | 200 | 67.431 | 137.633 | 51.0% | 22.02 | 27.81 |
| basic-image-unique-v2 | 500 | 127.239 | 187.974 | 32.3% | 29.05 | 30.39 |
| basic-image-unique-v2 | 1000 | 235.151 | 276.640 | 15.0% | 39.92 | 34.59 |
| basic-mixed-shared-v2 | 100 | 37.483 | 111.736 | 66.5% | 19.03 | 27.09 |
| basic-mixed-shared-v2 | 200 | 42.445 | 113.005 | 62.4% | 21.06 | 27.47 |
| basic-mixed-shared-v2 | 500 | 57.460 | 119.412 | 51.9% | 26.44 | 29.33 |
| basic-mixed-shared-v2 | 1000 | 80.108 | 128.709 | 37.8% | 35.39 | 32.25 |
| basic-mixed-text-v1 | 100 | 27.640 | 103.191 | 73.2% | 14.05 | 26.55 |
| basic-mixed-text-v1 | 200 | 31.053 | 105.013 | 70.4% | 15.88 | 27.20 |
| basic-mixed-text-v1 | 500 | 44.769 | 114.659 | 61.0% | 21.30 | 28.75 |
| basic-mixed-text-v1 | 1000 | 69.111 | 121.707 | 43.2% | 30.19 | 31.22 |
| basic-mixed-unique-v2 | 100 | 40.294 | 113.154 | 64.4% | 19.50 | 26.78 |
| basic-mixed-unique-v2 | 200 | 53.793 | 126.107 | 57.3% | 21.47 | 28.05 |
| basic-mixed-unique-v2 | 500 | 99.370 | 159.336 | 37.6% | 28.31 | 29.77 |
| basic-mixed-unique-v2 | 1000 | 173.039 | 217.672 | 20.5% | 38.53 | 33.70 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
