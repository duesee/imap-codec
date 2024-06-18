import unittest

from imap_codec import sum_as_string


class TestSumAsString(unittest.TestCase):
    def test_1(self):
        self.assertEqual(sum_as_string(0, 1), "1")
        self.assertEqual(sum_as_string(1, 0), "1")

    def test_2(self):
        self.assertEqual(sum_as_string(0, 2), "2")
        self.assertEqual(sum_as_string(1, 1), "2")
        self.assertEqual(sum_as_string(2, 0), "2")

    def test_3(self):
        self.assertEqual(sum_as_string(0, 3), "3")
        self.assertEqual(sum_as_string(1, 2), "3")
        self.assertEqual(sum_as_string(2, 1), "3")
        self.assertEqual(sum_as_string(3, 0), "3")

    def test_100(self):
        self.assertEqual(sum_as_string(50, 50), "100")
