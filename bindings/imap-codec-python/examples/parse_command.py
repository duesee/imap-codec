from common import COLOR_SERVER, RESET, Role, read_more
from imap_codec import CommandCodec, DecodeFailed, DecodeIncomplete, DecodeLiteralFound

WELCOME = r"""# Parsing of IMAP commands

"C:" denotes the client,
"S:" denotes the server, and
".." denotes the continuation of an (incomplete) command, e.g., due to the use of an IMAP literal.

Note: "\n" will be automatically replaced by "\r\n".

--------------------------------------------------------------------------------------------------

Enter IMAP command (or "exit").
"""


def main():
    print(WELCOME)

    buffer = bytearray()

    while True:
        # Try to parse the first command in `buffer`.
        try:
            remaining, command = CommandCodec.decode(bytes(buffer))
            # Parser succeeded.
            # Do something with the command ...
            print(command)
            # ... and proceed with the remaining data.
            buffer = bytearray(remaining)
        except DecodeIncomplete:
            # Parser needs more data.
            # Read more data.
            read_more(buffer, Role.Client)
        except DecodeLiteralFound:
            # Parser needs more data, and a command continuation request is expected.
            # Simulate literal acknowledgement ...
            print(f"S: {COLOR_SERVER}+ {RESET}")

            # ... and read more data.
            read_more(buffer, Role.Client)
        except DecodeFailed:
            # Parser failed.
            print("Error parsing command.")
            print("Clearing buffer.")

            # Clear the buffer and proceed with loop.
            buffer.clear()


if __name__ == "__main__":
    main()
