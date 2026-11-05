"""Independent synthetic public-material digest vector; no account access."""
import hashlib
import json
from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey

def public(byte):
    return X25519PrivateKey.from_private_bytes(bytes([byte]) * 32).public_key().public_bytes_raw()

parts = [bytes([1]) * 32, public(2), bytes([7]) * 64, public(3), public(4)]
print(json.dumps({
    "scope": "synthetic structural hash vector, not an authenticated key fixture",
    "derivation": "X25519 scalar bytes repeated 2,3,4; identity bytes 1; signature bytes 7",
    "public_parts_hex": [p.hex() for p in parts],
    "sha1": hashlib.sha1(b"".join(parts)).hexdigest(),
}, indent=2))
