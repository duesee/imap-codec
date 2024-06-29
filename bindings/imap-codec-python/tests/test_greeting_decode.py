import unittest

from imap_codec import GreetingCodec, GreetingDecodeFailed, GreetingDecodeIncomplete


class TestGreetingDecode(unittest.TestCase):
    def test_greeting(self):
        buffer = b"* OK Hello, World!\r\n<remaining>"
        remaining, greeting = GreetingCodec.decode(buffer)
        self.assertEqual(
            greeting, {"code": None, "kind": "Ok", "text": "Hello, World!"}
        )
        self.assertEqual(remaining, b"<remaining>")

    def test_greeting_with_code(self):
        buffer = b"* OK [ALERT] Hello, World!\r\n<remaining>"
        remaining, greeting = GreetingCodec.decode(buffer)
        self.assertEqual(
            greeting, {"code": "Alert", "kind": "Ok", "text": "Hello, World!"}
        )
        self.assertEqual(remaining, b"<remaining>")

    def test_greeting_without_remaining(self):
        buffer = b"* OK Hello, World!\r\n"
        remaining, greeting = GreetingCodec.decode(buffer)
        self.assertEqual(
            greeting, {"code": None, "kind": "Ok", "text": "Hello, World!"}
        )
        self.assertEqual(remaining, b"")

    def test_greeting_error_incomplete(self):
        buffer = b"* OK Hello, World!"
        with self.assertRaises(GreetingDecodeIncomplete) as cm:
            GreetingCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")

    def test_greeting_error_failed(self):
        buffer = b"OK"
        with self.assertRaises(GreetingDecodeFailed) as cm:
            GreetingCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")
