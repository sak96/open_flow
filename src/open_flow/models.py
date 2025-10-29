from enum import StrEnum, auto

from pydantic import BaseModel


class ActionType(StrEnum):
    mouse = auto()
    key = auto()


class Action(BaseModel):
    ty: ActionType
    value: str
    position: tuple[int, int] | None


class Screenshot(BaseModel):
    time: str
    img: str
    events: list[Action]
