import unittest

from imap_codec import Encoded, IdleDone, IdleDoneCodec, LineFragment


class TestIdleDoneEncode(unittest.TestCase):
    def test_idle_done(self):
        idle_done = IdleDone()
        encoded = IdleDoneCodec.encode(idle_done)
        self.assertIsInstance(encoded, Encoded)
        fragments = list(encoded)
        self.assertEqual(fragments, [LineFragment(b"DONE\r\n")])

    def test_idle_done_dump(self):
        idle_done = IdleDone()
        encoded = IdleDoneCodec.encode(idle_done)
        self.assertIsInstance(encoded, Encoded)
        self.assertEqual(encoded.dump(), b"DONE\r\n")
