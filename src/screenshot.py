from asyncio import Queue, sleep
from models import Screenshot
from datetime import datetime
from pyscreenshot import grab
from listeners import ACTIONS


SCREENSHOT: Queue[Screenshot] = Queue()
WAIT_TIME: int = 1


async def screenshot_task():
    while True:
        if not ACTIONS.empty():
            now = datetime.now()
            img = grab()
            events = ""
            while not ACTIONS.empty():
                events += "\n" + await ACTIONS.get()
            screenshot = Screenshot(
                time=now,
                events=events,
                img=img,  # type: ignore
            )
            await SCREENSHOT.put(screenshot)
            print(f"Screenshot taken at {now}")
        await sleep(WAIT_TIME)
