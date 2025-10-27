from asyncio import run, TaskGroup
from listeners import start_listeners
from screenshot import screenshot_task
from ai import dequeue_and_print_task


async def main():
    tasks = []
    async with TaskGroup() as tg:
        tasks.append(tg.create_task(screenshot_task()))
        tasks.append(tg.create_task(start_listeners()))
        tasks.append(tg.create_task(dequeue_and_print_task()))


if __name__ == "__main__":
    run(main())
