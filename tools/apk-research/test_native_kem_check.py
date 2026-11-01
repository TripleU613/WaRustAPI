"""Small harness checks; the separate NIST matrix exercises native execution."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from native_kem_check import check
from native_kem_check import LIBRARIES


class NativeKemCheckTests(unittest.TestCase):
    def test_wrong_build_refuses_before_execution(self):
        with tempfile.TemporaryDirectory() as root:
            library = Path(root) / 'library.so'
            library.write_bytes(b'wrong build')
            with patch('native_kem_check.execute') as execute:
                with self.assertRaisesRegex(ValueError, 'pinned library hash mismatch'):
                    check(library, {'cases': []})
                execute.assert_not_called()

    def test_expected_fields_do_not_depend_on_json_order(self):
        case = dict(operation='keyGen', tcId=1, input={}, expected={'dk': '02', 'ek': '01'})
        with patch.object(Path, 'read_bytes', return_value=b'library'), \
             patch('native_kem_check.hashlib.sha256') as digest, \
             patch('native_kem_check.execute', return_value=(0, [b'\x01', b'\x02'], {})):
            digest.return_value.hexdigest.return_value = LIBRARIES['libwhatsappmerged.so']
            self.assertEqual(len(check(Path('unused'), {'cases': [case]})['cases']), 1)

    def test_mismatch_and_nonzero_native_return_fail(self):
        case = dict(operation='decapsulation', tcId=7, input={}, expected={'k': '01'})
        for result in ((0, [b'\x02'], {}), (1, [b'\x01'], {})):
            with patch.object(Path, 'read_bytes', return_value=b'library'), \
                 patch('native_kem_check.hashlib.sha256') as digest, \
                 patch('native_kem_check.execute', return_value=result):
                digest.return_value.hexdigest.return_value = LIBRARIES['libwhatsappmerged.so']
                with self.assertRaisesRegex(ValueError, 'NIST mismatch'):
                    check(Path('unused'), {'cases': [case]})


if __name__ == '__main__':
    unittest.main()
