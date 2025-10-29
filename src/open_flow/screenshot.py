import base64
from asyncio import Queue, sleep
from datetime import datetime
from io import BytesIO
from typing import TYPE_CHECKING

from pyscreenshot import grab
from pywinctl import getActiveWindow

from open_flow.listeners import ACTIONS
from open_flow.models import Action, Screenshot

if TYPE_CHECKING:
    from PIL import Image  # noqa: F401


SCREENSHOT: Queue[Screenshot] = Queue()
WAIT_TIME: int = 1


def screenshot_url(bbox: tuple[int, int, int, int]) -> str:
    image: Image.Image = grab(bbox)  # pyright: ignore[reportAssignmentType]
    buf = BytesIO()
    image.save(buf, format="PNG")
    image_bytes = buf.getvalue()
    image_b64 = base64.b64encode(image_bytes).decode("utf-8")
    return f"data:image/png;base64,{image_b64}"


async def screenshot_task():
    while True:
        if not ACTIONS.empty():
            now = datetime.now()
            if not (win := getActiveWindow()):
                print("cound not get active window")
                continue
            img = screenshot_url(win.getClientFrame())
            events: list[Action] = []
            while not ACTIONS.empty():
                event = await ACTIONS.get()
                events.append(event)
            screenshot = Screenshot(
                time=now.isoformat(),
                events=events,
                img=img,  # type: ignore
            )
            await SCREENSHOT.put(screenshot)
            print(f"Screenshot taken at {now}")
        await sleep(WAIT_TIME)
