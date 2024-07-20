import unittest

from imap_codec import DecodeFailed, DecodeIncomplete, Greeting, GreetingCodec


class TestGreetingDecode(unittest.TestCase):
    def test_greeting(self):
        buffer = b"* OK Hello, World!\r\n<remaining>"
        remaining, greeting = GreetingCodec.decode(buffer)
        self.assertEqual(
            greeting,
            Greeting.from_dict({"code": None, "kind": "Ok", "text": "Hello, World!"}),
        )
        self.assertEqual(remaining, b"<remaining>")

    def test_greeting_with_code(self):
        buffer = b"* OK [ALERT] Hello, World!\r\n<remaining>"
        remaining, greeting = GreetingCodec.decode(buffer)
        self.assertEqual(
            greeting,
            Greeting.from_dict(
                {"code": "Alert", "kind": "Ok", "text": "Hello, World!"}
            ),
        )
        self.assertEqual(remaining, b"<remaining>")

    def test_greeting_without_remaining(self):
        buffer = b"* OK Hello, World!\r\n"
        remaining, greeting = GreetingCodec.decode(buffer)
        self.assertEqual(
            greeting,
            Greeting.from_dict({"code": None, "kind": "Ok", "text": "Hello, World!"}),
        )
        self.assertEqual(remaining, b"")

    def test_greeting_error_incomplete(self):
        buffer = b"* OK Hello, World!"
        with self.assertRaises(DecodeIncomplete) as cm:
            GreetingCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")

    def test_greeting_error_failed(self):
        buffer = b"OK"
        with self.assertRaises(DecodeFailed) as cm:
            GreetingCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")
