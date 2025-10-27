from openai import AsyncOpenAI
from io import BytesIO
import base64
from PIL import Image
from screenshot import SCREENSHOT
from listeners import ACTIONS

client = AsyncOpenAI()

SYSTEM_PROMPT = (
    "You are an interaction event summarizer."
    " Given previous 5 actions, recent keyboard/mouse events, and a screenshot image, generate a chronological list of actions in order."
    " Each entry must describe the event type and relevant screenshot context."
    " Order actions by real-time occurrence from earliest to latest."
    " Output only the ordered list, no extra text."
)


async def describe_pillow_image(image: Image.Image, events: str):
    # Convert Pillow image to PNG in memory and encode as base64
    buf = BytesIO()
    image.save(buf, format="PNG")
    image_bytes = buf.getvalue()
    image_b64 = base64.b64encode(image_bytes).decode("utf-8")
    image_url = f"data:image/png;base64,{image_b64}"

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
                    {
                        "type": "text",
                        # "text": events,
                        "text": "",
                    },
                    {"type": "image_url", "image_url": {"url": image_url}},
                ],
            },
        ],
        max_tokens=256,
    )
    if content := response.choices[0].message.content:
        print("output:", content.strip())
    else:
        print("issues")


async def dequeue_and_print_task():
    while True:
        details = await SCREENSHOT.get()
        print("getting description")
        await describe_pillow_image(details["img"], details["events"])
