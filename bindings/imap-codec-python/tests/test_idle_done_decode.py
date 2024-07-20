import unittest

from imap_codec import DecodeFailed, DecodeIncomplete, IdleDone, IdleDoneCodec


class TestIdleDoneDecode(unittest.TestCase):
    def test_idle_done(self):
        buffer = b"done\r\n<remaining>"
        remaining, idle_done = IdleDoneCodec.decode(buffer)
        self.assertEqual(idle_done, IdleDone())
        self.assertEqual(remaining, b"<remaining>")

    def test_idle_done_without_remaining(self):
        buffer = b"done\r\n"
        remaining, idle_done = IdleDoneCodec.decode(buffer)
        self.assertEqual(idle_done, IdleDone())
        self.assertEqual(remaining, b"")

    def test_idle_done_error_incomplete(self):
        buffer = b"do"
        with self.assertRaises(DecodeIncomplete) as cm:
            IdleDoneCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")

    def test_idle_done_error_failed(self):
        buffer = b"done \r\n"
        with self.assertRaises(DecodeFailed) as cm:
            IdleDoneCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")
