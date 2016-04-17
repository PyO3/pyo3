import random
import unittest
import btree

class TestBTreeSet(unittest.TestCase):

    def setUp(self):
        self.set = btree.BTreeSet()

    def test_empty(self):
        # make sure the initial set is empty
        # TODO self.assertEqual(bool(self.set), False)
        self.assertEqual(len(self.set), 0)

if __name__ == '__main__':
    unittest.main()

