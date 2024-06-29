import unittest

from imap_codec import (
    CommandCodec,
    CommandDecodeFailed,
    CommandDecodeIncomplete,
    CommandDecodeLiteralFound,
)


class TestCommandDecode(unittest.TestCase):
    def test_command(self):
        buffer = b"a NOOP\r\n<remaining>"
        remaining, command = CommandCodec.decode(buffer)
        self.assertEqual(command, {"tag": "a", "body": "Noop"})
        self.assertEqual(remaining, b"<remaining>")

    def test_command_without_remaining(self):
        buffer = b"a NOOP\r\n"
        remaining, command = CommandCodec.decode(buffer)
        self.assertEqual(command, {"tag": "a", "body": "Noop"})
        self.assertEqual(remaining, b"")

    def test_command_error_incomplete(self):
        buffer = b"a NOOP"
        with self.assertRaises(CommandDecodeIncomplete) as cm:
            CommandCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")

    def test_command_error_literal_found(self):
        buffer = b"a SELECT {5}\r\n"
        with self.assertRaises(CommandDecodeLiteralFound) as cm:
            CommandCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "{'tag': 'a', 'length': 5, 'mode': 'Sync'}")

    def test_command_error_failed(self):
        buffer = b"* NOOP"
        with self.assertRaises(CommandDecodeFailed) as cm:
            CommandCodec.decode(buffer)
        self.assertEqual(str(cm.exception), "")
