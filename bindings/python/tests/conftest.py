from __future__ import annotations

import sys
from pathlib import Path

PYTHON_SRC = Path(__file__).resolve().parents[1] / "src"
PYTHON_SRC_TEXT = str(PYTHON_SRC)

if PYTHON_SRC_TEXT not in sys.path:
    sys.path.insert(0, PYTHON_SRC_TEXT)
