# Media export benchmark

Timing: 10 independent processes per cell; RSS: 5 separate processes.

| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| basic-audio-unique-v2 | 100 | 40.837 | 113.675 | 64.1% | 18.73 | 26.84 |
| basic-audio-unique-v2 | 200 | 59.482 | 126.746 | 53.1% | 21.30 | 28.30 |
| basic-audio-unique-v2 | 500 | 107.857 | 166.930 | 35.4% | 28.53 | 31.00 |
| basic-audio-unique-v2 | 1000 | 194.208 | 231.928 | 16.3% | 40.41 | 35.31 |
| basic-image-unique-v2 | 100 | 45.644 | 119.714 | 61.9% | 19.75 | 26.81 |
| basic-image-unique-v2 | 200 | 67.136 | 135.858 | 50.6% | 22.28 | 27.69 |
| basic-image-unique-v2 | 500 | 127.233 | 187.083 | 32.0% | 29.02 | 30.53 |
| basic-image-unique-v2 | 1000 | 230.879 | 274.081 | 15.8% | 40.47 | 34.58 |
| basic-mixed-shared-v2 | 100 | 36.474 | 110.792 | 67.1% | 19.17 | 26.81 |
| basic-mixed-shared-v2 | 200 | 39.960 | 111.014 | 64.0% | 21.02 | 27.44 |
| basic-mixed-shared-v2 | 500 | 56.739 | 117.906 | 51.9% | 26.48 | 29.12 |
| basic-mixed-shared-v2 | 1000 | 79.955 | 127.795 | 37.4% | 35.34 | 32.22 |
| basic-mixed-text-v1 | 100 | 27.105 | 103.480 | 73.8% | 13.95 | 27.23 |
| basic-mixed-text-v1 | 200 | 31.209 | 104.452 | 70.1% | 15.80 | 27.55 |
| basic-mixed-text-v1 | 500 | 44.508 | 111.721 | 60.2% | 21.28 | 29.00 |
| basic-mixed-text-v1 | 1000 | 67.965 | 122.540 | 44.5% | 30.30 | 31.27 |
| basic-mixed-unique-v2 | 100 | 38.876 | 112.471 | 65.4% | 19.47 | 26.69 |
| basic-mixed-unique-v2 | 200 | 53.467 | 123.722 | 56.8% | 21.95 | 28.22 |
| basic-mixed-unique-v2 | 500 | 97.478 | 157.863 | 38.3% | 28.38 | 30.23 |
| basic-mixed-unique-v2 | 1000 | 173.214 | 217.364 | 20.3% | 39.06 | 33.55 |

Cells meeting the proposed 30% reduction: 17/20.

All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked.
