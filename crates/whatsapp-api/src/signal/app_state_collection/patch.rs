//! Comparison BVq/BQ6 patch protobuf. External mutations require a separate
//! authenticated retrieval path and are not silently discarded here.
use super::{
    snapshot::{Blob, Record, Version, blob, fixed},
    *,
};
use prost::Message;

#[derive(Message)]
struct WirePatch {
    #[prost(message, optional, tag = "1")]
    version: Option<Version>,
    #[prost(message, repeated, tag = "2")]
    mutations: Vec<WireMutation>,
    #[prost(bytes = "vec", optional, tag = "4")]
    snapshot_mac: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "5")]
    patch_mac: Option<Vec<u8>>,
    #[prost(message, optional, tag = "6")]
    key_id: Option<Blob>,
    #[prost(uint32, optional, tag = "8")]
    device_index: Option<u32>,
    #[prost(bytes = "vec", optional, tag = "9")]
    client_debug_data: Option<Vec<u8>>,
}

#[derive(Message)]
struct WireMutation {
    #[prost(int32, optional, tag = "1")]
    operation: Option<i32>,
    #[prost(message, optional, tag = "2")]
    record: Option<Record>,
}

impl CollectionPatch {
    /// Decode an inline patch, without trusting its MACs or activating any keys.
    /// External mutation references and exit-code branches are not yet supported
    /// and fail explicitly, even if inline mutations are also present. Bounds
    /// and duplicate-singleton refusal are local defensive parser policy.
    pub fn decode(mut bytes: &[u8]) -> Result<Self> {
        use prost::encoding::{DecodeContext, decode_key, skip_field};
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid("patch exceeds local byte bound"));
        }
        let original = bytes;
        let mut fields = 0;
        let mut mutations = 0;
        let mut singletons = BTreeSet::new();
        while !bytes.is_empty() {
            let (tag, wire) =
                decode_key(&mut bytes).map_err(|_| invalid("invalid patch protobuf"))?;
            fields += 1;
            if tag == 2 {
                mutations += 1;
            }
            if fields > 8192 || mutations > 4096 {
                return Err(invalid("patch exceeds local field bound"));
            }
            match tag {
                3 => {
                    return Err(invalid(
                        "external mutations require authenticated retrieval",
                    ));
                }
                7 => return Err(invalid("patch exit-code handling is not implemented")),
                1 | 4..=6 | 8 | 9 if !singletons.insert(tag) => {
                    return Err(invalid("ambiguous duplicate patch field"));
                }
                _ => {}
            }
            skip_field(wire, tag, &mut bytes, DecodeContext::default())
                .map_err(|_| invalid("invalid patch protobuf"))?;
        }
        let patch = WirePatch::decode(original).map_err(|_| invalid("invalid patch protobuf"))?;
        let version = patch
            .version
            .and_then(|v| v.value)
            .ok_or_else(|| invalid("missing patch version"))?;
        let key_id = fixed(blob(patch.key_id)?)?;
        let snapshot_mac = fixed(
            patch
                .snapshot_mac
                .ok_or_else(|| invalid("missing patch snapshot MAC"))?,
        )?;
        let patch_mac = fixed(
            patch
                .patch_mac
                .ok_or_else(|| invalid("missing patch MAC"))?,
        )?;
        let mut mutations = Vec::new();
        for mutation in patch.mutations {
            // Protobuf enum default is SET (0); explicit unknown values fail.
            let operation = match mutation.operation.unwrap_or(0) {
                0 => MutationOperation::Set,
                1 => MutationOperation::Remove,
                _ => return Err(invalid("unknown mutation operation")),
            };
            let record = mutation
                .record
                .ok_or_else(|| invalid("missing mutation record"))?;
            let body = blob(record.value)?;
            if !(64..=1_048_576).contains(&body.len()) {
                return Err(invalid("invalid mutation record size"));
            }
            mutations.push(EncryptedMutation {
                operation,
                key_id: fixed(blob(record.key_id)?)?,
                index_mac: fixed(blob(record.index)?)?,
                body,
            });
        }
        Ok(Self {
            version,
            key_id,
            snapshot_mac,
            patch_mac,
            mutations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal_patch(remove: bool) -> Vec<u8> {
        // Hand-written protobuf tags/lengths, not WirePatch::encode. Crypto
        // values are the independent Python/OpenSSL SET/REMOVE fixtures.
        let fixture = super::super::tests::patch(remove);
        let mut bytes = vec![
            0x0a,
            2,
            8,
            fixture.version as u8,
            0x12,
            0x87,
            1,
            8,
            u8::from(remove),
            0x12,
            0x82,
            1,
            0x0a,
            0x22,
            0x0a,
            0x20,
        ];
        bytes.extend(fixture.mutations[0].index_mac);
        bytes.extend([0x12, 0x52, 0x0a, 0x50]);
        bytes.extend(&fixture.mutations[0].body);
        bytes.extend([0x1a, 8, 0x0a, 6]);
        bytes.extend(fixture.key_id);
        bytes.extend([0x22, 32]);
        bytes.extend(fixture.snapshot_mac);
        bytes.extend([0x2a, 32]);
        bytes.extend(fixture.patch_mac);
        bytes.extend([0x32, 8, 0x0a, 6]);
        bytes.extend(fixture.key_id);
        bytes
    }

    #[test]
    fn literal_wire_set_remove_verifies() {
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        let set = CollectionPatch::decode(&literal_patch(false)).unwrap();
        let state = CollectionState::empty("regular")
            .unwrap()
            .verify_patch(&set, |_| Ok(&keys))
            .unwrap()
            .state;
        assert_eq!(state.len(), 1);
        let remove = CollectionPatch::decode(&literal_patch(true)).unwrap();
        let state = state.verify_patch(&remove, |_| Ok(&keys)).unwrap().state;
        assert_eq!(state.version(), 2);
        assert!(state.is_empty());
        assert_eq!(state.hash(), &StateHash::default());
    }

    #[test]
    fn unsupported_branches_and_malformed_fields_fail() {
        for suffix in [&[0x1a, 0][..], &[0x3a, 0], &[0x0a, 2, 8, 1]] {
            let mut bytes = literal_patch(false);
            bytes.extend(suffix);
            assert!(CollectionPatch::decode(&bytes).is_err());
        }
        let mut unknown = literal_patch(false);
        unknown[8] = 2;
        assert!(CollectionPatch::decode(&unknown).is_err());
        assert!(CollectionPatch::decode(&[]).is_err());
        assert!(CollectionPatch::decode(&[0x12, 0].repeat(4097)).is_err());
        let bytes = literal_patch(false);
        for end in 0..bytes.len() {
            assert!(CollectionPatch::decode(&bytes[..end]).is_err());
        }
    }

    #[test]
    fn omitted_set_enum_uses_protobuf_default_and_metadata_does_not_authorize() {
        let mut bytes = literal_patch(false);
        bytes.drain(7..9);
        bytes[5] = 0x85; // Mutation shrinks by two bytes, still length-delimited.
        bytes.extend([0x40, 3, 0x4a, 2, 1, 2]); // deviceIndex/clientDebugData
        let patch = CollectionPatch::decode(&bytes).unwrap();
        assert!(matches!(
            patch.mutations[0].operation,
            MutationOperation::Set
        ));
        let state = CollectionState::empty("regular").unwrap();
        assert!(
            state
                .verify_patch(&patch, |_| Err(Error::Cancelled))
                .is_err()
        );
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        assert!(state.verify_patch(&patch, |_| Ok(&keys)).is_ok());
    }
}
