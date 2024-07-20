import unittest

from imap_codec import (
    DecodeFailed,
    DecodeIncomplete,
    DecodeLiteralFound,
    Response,
    ResponseCodec,
)


class TestResponseDecode(unittest.TestCase):
    def test_response(self):
        buffer = b"* SEARCH 1\r\n<remaining>"
        remaining, response = ResponseCodec.decode(buffer)
        self.assertEqual(response, Response.from_dict({"Data": {"Search": [1]}}))
        self.assertEqual(remaining, b"<remaining>")

    def test_response_without_remaining(self):
        buffer = b"* SEARCH 1\r\n"
        remaining, response = ResponseCodec.decode(buffer)
        self.assertEqual(response, Response.from_dict({"Data": {"Search": [1]}}))
        self.assertEqual(remaining, b"")

    def test_response_error_incomplete(self):
        buffer = b"* SEARCH 1"
        with self.assertRaises(DecodeIncomplete) as cm:
            ResponseCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")

    def test_response_error_literal_found(self):
        buffer = b"* 1 FETCH (RFC822 {5}\r\n"
        with self.assertRaises(DecodeLiteralFound) as cm:
            ResponseCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "{'length': 5}")

    def test_response_error_failed(self):
        buffer = b"A SEARCH\r\n"
        with self.assertRaises(DecodeFailed) as cm:
            ResponseCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")
