from asyncio import Queue, Task, TaskGroup

from open_flow.models import Action, ActionType

ACTIONS: Queue[Action] = Queue()


async def start_listeners():
    try:
        from pynput import keyboard, mouse

        def on_key_event(key: keyboard.KeyCode | keyboard.Key | None):
            if isinstance(key, keyboard.KeyCode):
                ACTIONS.put_nowait(
                    Action(ty=ActionType.key, value=repr(key), position=None)
                )

        def on_mouse_event(x: int, y: int, button: mouse.Button, pressed: bool):
            if pressed:
                ACTIONS.put_nowait(
                    Action(ty=ActionType.mouse, value=str(button), position=(x, y))
                )

        keyboard.Listener(on_press=on_key_event).start()
        mouse.Listener(on_click=on_mouse_event).start()
    except Exception:
        from evdev import (  # pyright: ignore[reportUnknownVariableType]
            InputDevice,
            ecodes,
            list_devices,
        )
        from evdev.ecodes import keys
        from evdev.eventio_async import EventIO

        async def keyboard_listener(dev: EventIO):
            async for event in dev.async_read_loop():
                if event.type == ecodes.EV_KEY and (value := keys.get(event.code)):
                    if isinstance(value, tuple):
                        value = "+".join(value)
                    await ACTIONS.put(
                        Action(ty=ActionType.key, value=value, position=None)
                    )

        async def mouse_listener(dev: EventIO):
            x, y = 0, 0
            async for event in dev.async_read_loop():
                if event.type == ecodes.EV_REL:
                    if event.code == ecodes.REL_X:
                        x += event.value
                    elif event.code == ecodes.REL_Y:
                        y += event.value
                elif event.type == ecodes.EV_KEY:
                    button = event.code
                    pressed = event.value == 1
                    if pressed:
                        await ACTIONS.put(
                            Action(
                                ty=ActionType.mouse, value=str(button), position=(x, y)
                            )
                        )

        tasks: list[Task[None]] = []
        async with TaskGroup() as tg:
            for path in list_devices():
                dev = InputDevice(path)
                if "keyboard" in dev.name.lower():
                    tasks.append(tg.create_task(keyboard_listener(dev)))
                if "pointer" in dev.name.lower() or "mouse" in dev.name.lower():
                    tasks.append(tg.create_task(mouse_listener(dev)))
