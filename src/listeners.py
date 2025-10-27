from asyncio import Queue, TaskGroup
from concurrent.futures import ThreadPoolExecutor

ACTIONS: Queue[str] = Queue()


async def start_listeners():
    try:
        from pynput import keyboard, mouse

        def on_key_event(key: keyboard.KeyCode | keyboard.Key | None):
            if isinstance(key, keyboard.KeyCode):
                ACTIONS.put_nowait(f"key pressed: {key.char}")

        def on_mouse_event(x: int, y: int, button: mouse.Button, pressed: bool):
            ACTIONS.put_nowait(
                f"mouse {button} {'pressed' if pressed else 'released'} @ ({x},{y})"
            )

        keyboard.Listener(on_press=on_key_event).start()
        mouse.Listener(on_click=on_mouse_event).start()
    except Exception:
        from evdev import InputDevice, ecodes, list_devices

        async def keyboard_listener(dev: InputDevice):
            async for event in dev.async_read_loop():
                if event.type == ecodes.EV_KEY:
                    await ACTIONS.put(f"key pressed: {event.code}")

        async def mouse_listener(dev: InputDevice):
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
                    await ACTIONS.put(
                        f"mouse {button} {'pressed' if pressed else 'released'} @ ({x},{y})"
                    )

        tasks = []
        async with TaskGroup() as tg:
            for path in list_devices():
                dev = InputDevice(path)
                if "keyboard" in dev.name.lower():
                    tg.create_task(keyboard_listener(dev))
                if "pointer" in dev.name.lower() or "mouse" in dev.name.lower():
                    tg.create_task(mouse_listener(dev))
