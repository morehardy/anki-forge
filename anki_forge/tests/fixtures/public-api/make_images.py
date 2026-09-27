"""Generate original image-occlusion test inputs; requires Pillow for JPEG."""
from pathlib import Path
from PIL import Image

root = Path(__file__).parent
image = Image.new("RGB", (100, 80), (0, 120, 240))
image.save(root / "occlusion.png")
exif = Image.Exif()
exif[274] = 6  # Display rotated 90 degrees clockwise: 80 × 100 pixels.
image.save(root / "rotated.jpg", exif=exif)
