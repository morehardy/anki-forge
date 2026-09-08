"""Predeclared paired timing and independent RSS passes across all 29 frozen cases."""
from pathlib import Path
import subprocess,sys
W=Path(__file__).resolve().parent
for name,repeats in [('before-after-timing',7),('before-after-rss',5)]:
    subprocess.run([sys.executable,str(W/'run.py'),name,'current','accepted','--warmups','2','--repeats',str(repeats)],check=True)
