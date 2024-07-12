import unittest

from imap_codec import CommandCodec, Encoded


class TestCommandEncode(unittest.TestCase):
    def test_simple_command(self):
        command = {"tag": "a", "body": "Noop"}
        encoded = CommandCodec.encode(command)
        self.assertIsInstance(encoded, Encoded)
        fragments = list(encoded)
        self.assertEqual(fragments, [{"Line": {"data": list(b"a NOOP\r\n")}}])

    def test_simple_command_dump(self):
        command = {"tag": "a", "body": "Noop"}
        encoded = CommandCodec.encode(command)
        self.assertIsInstance(encoded, Encoded)
        self.assertEqual(encoded.dump(), b"a NOOP\r\n")

    _MULTI_FRAGMENT_COMMAND = {
        "tag": "A",
        "body": {
            "Login": {
                "username": {"Atom": "alice"},
                "password": {
                    "String": {"Literal": {"data": list(b"\xCA\xFE"), "mode": "Sync"}}
                },
            }
        },
    }

    def test_multi_fragment_command(self):
        encoded = CommandCodec.encode(self._MULTI_FRAGMENT_COMMAND)
        self.assertIsInstance(encoded, Encoded)
        fragments = list(encoded)
        self.assertEqual(
            fragments,
            [
                {"Line": {"data": list(b"A LOGIN alice {2}\r\n")}},
                {"Literal": {"data": list(b"\xCA\xFE"), "mode": "Sync"}},
                {"Line": {"data": list(b"\r\n")}},
            ],
        )

    def test_multi_fragment_command_dump(self):
        encoded = CommandCodec.encode(self._MULTI_FRAGMENT_COMMAND)
        self.assertIsInstance(encoded, Encoded)
        self.assertEqual(encoded.dump(), b"A LOGIN alice {2}\r\n\xCA\xFE\r\n")

    def test_multi_fragment_command_dump_remaining(self):
        encoded = CommandCodec.encode(self._MULTI_FRAGMENT_COMMAND)
        self.assertIsInstance(encoded, Encoded)
        self.assertEqual(
            next(encoded), {"Line": {"data": list(b"A LOGIN alice {2}\r\n")}}
        )
        self.assertEqual(encoded.dump(), b"\xCA\xFE\r\n")
