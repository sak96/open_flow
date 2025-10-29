from asyncio import Task, TaskGroup, run

from open_flow.ai import dequeue_and_print_task
from open_flow.listeners import start_listeners
from open_flow.screenshot import screenshot_task


async def main():
    tasks: list[Task[None]] = []
    async with TaskGroup() as tg:
        tasks.append(tg.create_task(screenshot_task()))
        tasks.append(tg.create_task(start_listeners()))
        tasks.append(tg.create_task(dequeue_and_print_task()))


def cli():
    run(main())


if __name__ == "__main__":
    run(main())
