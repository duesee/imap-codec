import unittest

from imap_codec import LineFragment, LiteralFragment, LiteralMode


class TestLineFragment(unittest.TestCase):
    def test_data(self):
        data = b"a NOOP\r\n"
        fragment = LineFragment(data)
        self.assertEqual(fragment.data, data)

    def test_repr(self):
        data = b"a NOOP\r\n"
        fragment = LineFragment(data)
        self.assertEqual(repr(fragment), f"LineFragment({data})")

    def test_str(self):
        data = b"a NOOP\r\n"
        fragment = LineFragment(data)
        self.assertEqual(str(fragment), str(data))

    def test_eq(self):
        fragment1 = LineFragment(b"a NOOP\r\n")
        fragment2 = LineFragment(b"a NOOP\r\n")
        fragment3 = LineFragment(b"a LOGIN alice pass\r\n")

        self.assertEqual(fragment1, fragment1)
        self.assertEqual(fragment1, fragment2)
        self.assertNotEqual(fragment1, fragment3)

        self.assertEqual(fragment2, fragment1)
        self.assertEqual(fragment2, fragment2)
        self.assertNotEqual(fragment2, fragment3)

        self.assertNotEqual(fragment3, fragment1)
        self.assertNotEqual(fragment3, fragment2)
        self.assertEqual(fragment3, fragment3)


class TestLiteralFragment(unittest.TestCase):
    def test_data(self):
        data = b"\x01\x02\x03\x04"
        fragment = LiteralFragment(data, LiteralMode.Sync)
        self.assertEqual(fragment.data, data)

    def test_mode(self):
        mode = LiteralMode.Sync
        fragment = LiteralFragment(b"\x01\x02\x03\x04", mode)
        self.assertEqual(fragment.mode, mode)

        mode = LiteralMode.NonSync
        fragment = LiteralFragment(b"\x01\x02\x03\x04", mode)
        self.assertEqual(fragment.mode, mode)

    def test_repr(self):
        data = b"\x01\x02\x03\x04"

        mode = LiteralMode.Sync
        fragment = LiteralFragment(data, mode)
        self.assertEqual(repr(fragment), f"LiteralFragment({data}, {mode})")

        mode = LiteralMode.NonSync
        fragment = LiteralFragment(data, mode)
        self.assertEqual(repr(fragment), f"LiteralFragment({data}, {mode})")

    def test_str(self):
        data = b"\x01\x02\x03\x04"

        mode = LiteralMode.Sync
        fragment = LiteralFragment(data, mode)
        self.assertEqual(str(fragment), f"({data}, {mode})")

        mode = LiteralMode.NonSync
        fragment = LiteralFragment(data, mode)
        self.assertEqual(str(fragment), f"({data}, {mode})")

    def test_eq(self):
        fragment1 = LiteralFragment(b"data", LiteralMode.Sync)
        fragment2 = LiteralFragment(b"data", LiteralMode.Sync)
        fragment3 = LiteralFragment(b"data", LiteralMode.NonSync)
        fragment4 = LiteralFragment(b"\x01\x02\x03\x04", LiteralMode.NonSync)

        self.assertEqual(fragment1, fragment1)
        self.assertEqual(fragment1, fragment2)
        self.assertNotEqual(fragment1, fragment3)
        self.assertNotEqual(fragment1, fragment4)

        self.assertEqual(fragment2, fragment1)
        self.assertEqual(fragment2, fragment2)
        self.assertNotEqual(fragment2, fragment3)
        self.assertNotEqual(fragment2, fragment4)

        self.assertNotEqual(fragment3, fragment1)
        self.assertNotEqual(fragment3, fragment2)
        self.assertEqual(fragment3, fragment3)
        self.assertNotEqual(fragment3, fragment4)

        self.assertNotEqual(fragment4, fragment1)
        self.assertNotEqual(fragment4, fragment2)
        self.assertNotEqual(fragment4, fragment3)
        self.assertEqual(fragment4, fragment4)
