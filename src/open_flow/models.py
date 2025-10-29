from datetime import datetime
from enum import StrEnum, auto
from typing import TypedDict


class ActionType(StrEnum):
    mouse = auto()
    key = auto()


class Action(TypedDict):
    ty: ActionType
    value: str
    position: tuple[int, int] | None


class Screenshot(TypedDict):
    time: datetime
    img: str
    events: list[Action]
