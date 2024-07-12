import unittest

from imap_codec import Encoded, IdleDoneCodec


class TestIdleDoneEncode(unittest.TestCase):
    def test_idle_done(self):
        idle_done = ()
        encoded = IdleDoneCodec.encode(idle_done)
        self.assertIsInstance(encoded, Encoded)
        fragments = list(encoded)
        self.assertEqual(fragments, [{"Line": {"data": list(b"DONE\r\n")}}])

    def test_idle_done_dump(self):
        idle_done = ()
        encoded = IdleDoneCodec.encode(idle_done)
        self.assertIsInstance(encoded, Encoded)
        self.assertEqual(encoded.dump(), b"DONE\r\n")
