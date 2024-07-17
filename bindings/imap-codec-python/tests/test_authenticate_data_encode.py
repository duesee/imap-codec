import unittest

from imap_codec import AuthenticateDataCodec, Encoded, LineFragment


class TestAuthenticateDataEncode(unittest.TestCase):
    def test_authenticate_data(self):
        authenticate_data = {"Continue": list(b"Test")}
        encoded = AuthenticateDataCodec.encode(authenticate_data)
        self.assertIsInstance(encoded, Encoded)
        fragments = list(encoded)
        self.assertEqual(fragments, [LineFragment(b"VGVzdA==\r\n")])

    def test_authenticate_data_dump(self):
        authenticate_data = {"Continue": list(b"Test")}
        encoded = AuthenticateDataCodec.encode(authenticate_data)
        self.assertIsInstance(encoded, Encoded)
        self.assertEqual(encoded.dump(), b"VGVzdA==\r\n")
