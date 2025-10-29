from openai import AsyncOpenAI
from screenshot import SCREENSHOT
from textwrap import dedent
from models import Action

client = AsyncOpenAI()

SYSTEM_PROMPT = dedent(
    """
    You are an interaction event summarizer.
    Given previous 5 actions, recent keyboard/mouse events, and a screenshot image, generate a chronological list of actions in order.
    Each entry must be a JSON object with fields: object_type (string), action (string), properties (object mapping string to string), identifier (string).
    Order actions by real-time occurrence from earliest to latest. Output only the list of JSON objects, one per line.
    """.strip()
)


async def describe_pillow_image(image_url: str, events: list[Action]):
    # Convert Pillow image to PNG in memory and encode as base64
    response = await client.chat.completions.create(
        model="gemma3:latest",
        messages=[
            {
                "role": "system",
                "content": [
                    {
                        "type": "text",
                        "text": SYSTEM_PROMPT,
                    },
                ],
            },
            {
                "role": "user",
                "content": [
                    # {
                    #     "type": "text",
                    #     "text": events,
                    # },
                    {"type": "image_url", "image_url": {"url": image_url}},
                ],
            },
        ],
    )
    if content := response.choices[0].message.content:
        print(events)
        print("output:", content.strip())
    else:
        print("issues")


async def dequeue_and_print_task():
    while True:
        details = await SCREENSHOT.get()
        print("getting description")
        await describe_pillow_image(details["img"], details["events"])
