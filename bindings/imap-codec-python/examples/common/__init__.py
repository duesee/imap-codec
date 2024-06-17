import sys
from enum import Enum
from typing import Optional

COLOR_CLIENT = "\x1b[31m"
COLOR_SERVER = "\x1b[34m"
RESET = "\x1b[0m"


class Role(Enum):
    Client = 1
    Server = 2


def read_more(buffer: bytearray, role: Role):
    if not buffer:
        prompt = "C: " if role == Role.Client else "S: "
    else:
        prompt = ".. "

    line = read_line(prompt, role)

    # If `read_line` returns `None`, standard input has been closed.
    if line is None or line.strip() == "exit":
        print("Exiting.")
        sys.exit(0)

    buffer += line.encode()


def read_line(prompt: str, role: Role) -> Optional[str]:
    color = COLOR_CLIENT if role == Role.Client else COLOR_SERVER

    try:
        line = input(f"{prompt}{color}")
    except EOFError:
        # Standard input has been closed.
        return None
    finally:
        print(RESET, end=None)

    return line + "\r\n"
