import unittest

from imap_codec import (
    AuthenticateData,
    AuthenticateDataCodec,
    DecodeFailed,
    DecodeIncomplete,
)


class TestAuthenticateDataDecode(unittest.TestCase):
    def test_authenticate_data(self):
        buffer = b"VGVzdA==\r\n<remaining>"
        remaining, authenticate_data = AuthenticateDataCodec.decode(buffer)
        self.assertEqual(
            authenticate_data, AuthenticateData.from_dict({"Continue": list(b"Test")})
        )
        self.assertEqual(remaining, b"<remaining>")

    def test_authenticate_data_without_remaining(self):
        buffer = b"VGVzdA==\r\n"
        remaining, authenticate_data = AuthenticateDataCodec.decode(buffer)
        self.assertEqual(
            authenticate_data, AuthenticateData.from_dict({"Continue": list(b"Test")})
        )
        self.assertEqual(remaining, b"")

    def test_authenticate_data_error_incomplete(self):
        buffer = b"VGV"
        with self.assertRaises(DecodeIncomplete) as cm:
            AuthenticateDataCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")

    def test_authenticate_data_error_failed(self):
        buffer = b"VGVzdA== \r\n"
        with self.assertRaises(DecodeFailed) as cm:
            AuthenticateDataCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")
