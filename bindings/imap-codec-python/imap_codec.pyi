from __future__ import annotations

from typing import Tuple, Union

class DecodeError(Exception):
    """
    Error during decoding.
    """

class DecodeFailed(DecodeError):
    """
    "Failed" error during decoding:
    Decoding failed.
    """

class DecodeIncomplete(DecodeError):
    """
    "Incomplete" error during decoding:
    More data is needed.
    """

class DecodeLiteralFound(DecodeError):
    """
    "LiteralFound" error during decoding:
    The decoder stopped at the beginning of literal data.
    """

class LiteralMode:
    """
    Literal mode, i.e., sync or non-sync.

    - Sync: A synchronizing literal, i.e., `{<n>}\r\n<data>`.
    - NonSync: A non-synchronizing literal according to RFC 7888, i.e., `{<n>+}\r\n<data>`.

    Warning: The non-sync literal extension must only be used when the server advertised support
             for it sending the LITERAL+ or LITERAL- capability.
    """

    Sync: LiteralMode
    NonSync: LiteralMode

class LineFragment:
    """
    Fragment of a line that is ready to be send.
    """

    def __init__(self, data: bytes):
        """
        Create a line fragment from data bytes

        :param data: Data bytes of fragment
        :raises TypeError: `data` is not byte-like
        """

    @property
    def data(self) -> bytes:
        """
        Get line fragment data bytes

        :return: Data bytes of fragment
        """

class LiteralFragment:
    """
    Fragment of a literal that may require an action before it should be send.
    """

    def __init__(self, data: bytes, mode: LiteralMode):
        """
        Create a literal fragment from data bytes and literal mode

        :param data: Data bytes of fragment
        :param mode: Literal mode
        :raises TypeError: `data` is not byte-like
        :raises TypeError: `mode` is invalid
        """

    @property
    def data(self) -> bytes:
        """
        Get literal fragment data bytes

        :return: Data bytes of fragment
        """

    @property
    def mode(self) -> LiteralMode:
        """
        Get literal fragment literal mode

        :return: Literal mode
        """

class Encoded:
    """
    An encoded message.

    This struct facilitates the implementation of IMAP client- and server implementations by
    yielding the encoding of a message through fragments. This is required, because the usage of
    literals (and some other types) may change the IMAP message flow. Thus, in many cases, it is an
    error to just "dump" a message and send it over the network.
    """

    def __iter__(self) -> Encoded: ...
    def __next__(self) -> Union[LineFragment, LiteralFragment]: ...
    def dump(self) -> bytes:
        """
        Dump the (remaining) encoded data without being guided by fragments.
        """

class Greeting:
    """
    Greeting.

    Note: Don't use `code: None` *and* a `text` that starts with "[" as this would be ambiguous in IMAP.
    We could fix this but the fix would make this type unconformable to use.
    """

    @staticmethod
    def from_dict(greeting: dict) -> Greeting:
        """
        Create greeting from `dict`

        :param greeting: Dictionary representation of greeting
        :raises RuntimeError: Dictionary could not be deserialized into greeting
        """

    def as_dict(self) -> dict:
        """
        Return greeting as `dict`

        :return: Dictionary representation of greeting
        """

class GreetingCodec:
    """
    Codec for greetings.
    """

    @staticmethod
    def decode(bytes: bytes) -> Tuple[bytes, Greeting]:
        """
        Decode greeting from given bytes.

        :param bytes: Given bytes
        :raises DecodeFailed: Decoding failed.
        :raises DecodeIncomplete: More data is needed.
        :return: Tuple of remaining bytes and decoded greeting
        """

    @staticmethod
    def encode(greeting: Greeting) -> Encoded:
        """
        Encode greeting into fragments.

        :param greeting: Given greeting
        :return: `Encoded` type holding fragments of encoded greeting
        """

class Command:
    """
    Command.
    """

    @staticmethod
    def from_dict(command: dict) -> Command:
        """
        Create command from `dict`

        :param command: Dictionary representation of command
        :raises RuntimeError: Dictionary could not be deserialized into command
        """

    def as_dict(self) -> dict:
        """
        Return command as `dict`

        :return: Dictionary representation of command
        """

class CommandCodec:
    """
    Codec for commands.
    """

    @staticmethod
    def decode(bytes: bytes) -> Tuple[bytes, Command]:
        """
        Decode command from given bytes.

        :param bytes: Given bytes
        :raises DecodeFailed: Decoding failed.
        :raises DecodeIncomplete: More data is needed.
        :raises DecodeLiteralFound: The decoder stopped at the beginning of literal data.
        :return: Tuple of remaining bytes and decoded command
        """

    @staticmethod
    def encode(command: Command) -> Encoded:
        """
        Encode command into fragments.

        :param command: Given command
        :return: `Encoded` type holding fragments of encoded command
        """

class AuthenticateData:
    """
    Authenticate data line

    Data line used, e.g., during AUTHENTICATE.
    """

    @staticmethod
    def from_dict(authenticate_data: dict) -> AuthenticateData:
        """
        Create authenticate data line from `dict`

        :param authenticate_data: Dictionary representation of authenticate data line
        :raises RuntimeError: Dictionary could not be deserialized into authenticate data line
        """

    def as_dict(self) -> dict:
        """
        Return authenticate data line as `dict`

        :return: Dictionary representation of authenticate data line
        """

class AuthenticateDataCodec:
    """
    Codec for authenticate data lines.
    """

    @staticmethod
    def decode(bytes: bytes) -> Tuple[bytes, AuthenticateData]:
        """
        Decode authenticate data line from given bytes.

        :param bytes: Given bytes
        :raises DecodeFailed: Decoding failed.
        :raises DecodeIncomplete: More data is needed.
        :return: Tuple of remaining bytes and decoded authenticate data line
        """

    @staticmethod
    def encode(authenticate_data: AuthenticateData) -> Encoded:
        """
        Encode authenticate data line into fragments.

        :param authenticate_data: Given authenticate data line
        :return: `Encoded` type holding fragments of encoded authenticate data line
        """

class Response:
    """
    Response.
    """

    @staticmethod
    def from_dict(response: dict) -> Response:
        """
        Create response from `dict`

        :param response: Dictionary representation of response
        :raises RuntimeError: Dictionary could not be deserialized into response
        """

    def as_dict(self) -> dict:
        """
        Return response as `dict`

        :return: Dictionary representation of response
        """

class ResponseCodec:
    """
    Codec for responses.
    """

    @staticmethod
    def decode(bytes: bytes) -> Tuple[bytes, Response]:
        """
        Decode response from given bytes.

        :param bytes: Given bytes
        :raises DecodeFailed: Decoding failed.
        :raises DecodeIncomplete: More data is needed.
        :raises DecodeLiteralFound: The decoder stopped at the beginning of literal data.
        :return: Tuple of remaining bytes and decoded response
        """

    @staticmethod
    def encode(response: Response) -> Encoded:
        """
        Encode response into fragments.

        :param response: Given response
        :return: `Encoded` type holding fragments of encoded response
        """

class IdleDone:
    """
    Denotes the continuation data message "DONE\r\n" to end the IDLE command.
    """

    def __new__(cls) -> IdleDone:
        """
        Create idle done
        """

class IdleDoneCodec:
    """
    Codec for idle dones.
    """

    @staticmethod
    def decode(bytes: bytes) -> Tuple[bytes, IdleDone]:
        """
        Decode idle done from given bytes.

        :param bytes: Given bytes
        :raises DecodeFailed: Decoding failed.
        :raises DecodeIncomplete: More data is needed.
        :raises DecodeLiteralFound: The decoder stopped at the beginning of literal data.
        :return: Tuple of remaining bytes and decoded idle done
        """

    @staticmethod
    def encode(idle_done: IdleDone) -> Encoded:
        """
        Encode idle done into fragments.

        :param idle_done: Given idle done
        :return: `Encoded` type holding fragments of encoded idle done
        """
