from common import Role, read_more
from imap_codec import DecodeFailed, DecodeIncomplete, GreetingCodec

WELCOME = r"""# Parsing of IMAP greetings

"S:" denotes the server.

Note: "\n" will be automatically replaced by "\r\n".

--------------------------------------------------------------------------------------------------

Enter IMAP greeting (or "exit").
"""


def main():
    print(WELCOME)

    buffer = bytearray()

    while True:
        # Try to parse the first greeting in `buffer`.
        try:
            remaining, greeting = GreetingCodec.decode(bytes(buffer))
            # Parser succeeded.
            # Do something with the greeting ...
            print(greeting)
            # ... and proceed with the remaining data.
            buffer = bytearray(remaining)
        except DecodeIncomplete:
            # Parser needs more data.
            # Read more data.
            read_more(buffer, Role.Server)
        except DecodeFailed:
            # Parser failed.
            print("Error parsing greeting.")
            print("Clearing buffer.")

            # Clear the buffer and proceed with loop.
            buffer.clear()


if __name__ == "__main__":
    main()
