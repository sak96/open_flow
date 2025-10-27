from datetime import datetime
from typing import TypedDict
from PIL import Image


class Screenshot(TypedDict):
    time: datetime
    img: Image.Image
    events: str
