from common import Role, read_more
from imap_codec import DecodeFailed, DecodeIncomplete, DecodeLiteralFound, ResponseCodec

WELCOME = r"""# Parsing of IMAP responses

"S:" denotes the server, and
".." denotes the continuation of an (incomplete) response, e.g., due to the use of an IMAP literal.

Note: "\n" will be automatically replaced by "\r\n".

--------------------------------------------------------------------------------------------------

Enter IMAP response (or "exit").
"""


def main():
    print(WELCOME)

    buffer = bytearray()

    while True:
        # Try to parse the first response in `buffer`.
        try:
            remaining, response = ResponseCodec.decode(bytes(buffer))
            # Parser succeeded.
            # Do something with the response ...
            print(response)
            # ... and proceed with the remaining data.
            buffer = bytearray(remaining)
        except DecodeIncomplete:
            # Parser needs more data.
            # Read more data.
            read_more(buffer, Role.Server)
        except DecodeLiteralFound:
            # Parser needs more data.
            #
            # A client MUST receive any literal and can't reject it. However, if the literal is too
            # large, the client would have the (semi-optimal) option to still *read it* but discard
            # the data chunk by chunk. It could also close the connection. This is why we have this
            # option.
            #
            # Read more data.
            read_more(buffer, Role.Server)
        except DecodeFailed:
            # Parser failed.
            print("Error parsing response.")
            print("Clearing buffer.")

            # Clear the buffer and proceed with loop.
            buffer.clear()


if __name__ == "__main__":
    main()
