# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 37.826 | 105.913 | 64.3% | 18.36 | 28.53 |
| basic-audio-unique-v2 | 200 | 51.580 | 116.448 | 55.7% | 21.00 | 29.03 |
| basic-audio-unique-v2 | 500 | 96.444 | 150.012 | 35.7% | 28.52 | 31.81 |
| basic-audio-unique-v2 | 1000 | 169.252 | 205.992 | 17.8% | 40.33 | 36.17 |
| basic-image-unique-v2 | 100 | 40.906 | 109.485 | 62.6% | 18.97 | 28.42 |
| basic-image-unique-v2 | 200 | 60.100 | 122.578 | 51.0% | 21.42 | 29.12 |
| basic-image-unique-v2 | 500 | 116.105 | 167.432 | 30.7% | 28.50 | 31.72 |
| basic-image-unique-v2 | 1000 | 210.921 | 242.906 | 13.2% | 40.28 | 35.84 |
| basic-mixed-shared-v2 | 100 | 33.326 | 101.345 | 67.1% | 18.62 | 28.20 |
| basic-mixed-shared-v2 | 200 | 36.559 | 103.852 | 64.8% | 20.59 | 28.58 |
| basic-mixed-shared-v2 | 500 | 50.977 | 108.959 | 53.2% | 26.06 | 30.33 |
| basic-mixed-shared-v2 | 1000 | 72.780 | 117.432 | 38.0% | 35.06 | 32.94 |
| basic-mixed-text-v1 | 100 | 24.794 | 96.260 | 74.2% | 14.33 | 27.94 |
| basic-mixed-text-v1 | 200 | 28.712 | 97.766 | 70.6% | 16.14 | 28.56 |
| basic-mixed-text-v1 | 500 | 40.844 | 104.525 | 60.9% | 21.78 | 29.89 |
| basic-mixed-text-v1 | 1000 | 61.749 | 110.396 | 44.1% | 30.66 | 32.28 |
| basic-mixed-unique-v2 | 100 | 36.041 | 104.025 | 65.4% | 18.84 | 28.28 |
| basic-mixed-unique-v2 | 200 | 48.367 | 114.718 | 57.8% | 21.30 | 29.16 |
| basic-mixed-unique-v2 | 500 | 85.769 | 142.341 | 39.7% | 28.05 | 31.61 |
| basic-mixed-unique-v2 | 1000 | 154.516 | 191.940 | 19.5% | 38.84 | 34.94 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
