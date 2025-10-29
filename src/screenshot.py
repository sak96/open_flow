from asyncio import Queue, sleep
from models import Screenshot
from datetime import datetime
from pyscreenshot import grab
from listeners import ACTIONS
from models import Action
from pathlib import Path
from json import dumps
import base64
from io import BytesIO
from typing import TYPE_CHECKING


if TYPE_CHECKING:
    from PIL import Image  # noqa: F401


SCREENSHOT: Queue[Screenshot] = Queue()
WAIT_TIME: int = 1


def screenshot_url() -> str:
    image: Image.Image = grab()  # pyright: ignore[reportAssignmentType]
    buf = BytesIO()
    image.save(buf, format="PNG")
    image_bytes = buf.getvalue()
    image_b64 = base64.b64encode(image_bytes).decode("utf-8")
    return f"data:image/png;base64,{image_b64}"


async def screenshot_task():
    while True:
        if not ACTIONS.empty():
            now = datetime.now()
            img = screenshot_url()
            events: list[Action] = []
            while not ACTIONS.empty():
                event = await ACTIONS.get()
                events.append(event)
            screenshot = Screenshot(
                time=now,
                events=events,
                img=img,  # type: ignore
            )
            _ = (Path("~/temp/screenshot") / f"{now}.json").write_text(
                dumps(screenshot)
            )
            await SCREENSHOT.put(screenshot)
            print(f"Screenshot taken at {now}")
        await sleep(WAIT_TIME)
