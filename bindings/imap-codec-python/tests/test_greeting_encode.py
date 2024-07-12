import unittest

from imap_codec import Encoded, GreetingCodec


class TestGreetingEncode(unittest.TestCase):
    def test_simple_greeting(self):
        greeting = {"code": None, "kind": "Ok", "text": "Hello, World!"}
        encoded = GreetingCodec.encode(greeting)
        self.assertIsInstance(encoded, Encoded)
        fragments = list(encoded)
        self.assertEqual(
            fragments, [{"Line": {"data": list(b"* OK Hello, World!\r\n")}}]
        )

    def test_simple_greeting_dump(self):
        greeting = {"code": None, "kind": "Ok", "text": "Hello, World!"}
        encoded = GreetingCodec.encode(greeting)
        self.assertIsInstance(encoded, Encoded)
        self.assertEqual(encoded.dump(), b"* OK Hello, World!\r\n")
