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

/// One mutation to place in an outgoing patch.
pub struct OutgoingMutation {
    /// SET replaces the value at this index; REMOVE deletes it and requires the
    /// index to be present in the current state.
    pub operation: MutationOperation,
    /// Exact index JSON array bytes. These are MAC'd verbatim, so the caller
    /// must pass the same bytes it wants authenticated, never a reserialization.
    pub index: Vec<u8>,
    /// `SyncActionValue` protobuf. Required for SET, which must carry its
    /// timestamp (field 1); optional for REMOVE.
    pub action: Option<Vec<u8>>,
    /// Action schema version, distinct from the collection's ordering counter.
    pub version: i32,
}
impl std::fmt::Debug for OutgoingMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OutgoingMutation(<redacted>)")
    }
}

/// A patch ready to send, plus the state to persist **only** once the server
/// has accepted it. Applying the new state before acceptance would desynchronize
/// the collection version from the server's.
pub struct BuiltPatch {
    /// `SyncdPatch` wire bytes for the `<patch>` child.
    pub bytes: Vec<u8>,
    /// Candidate state at `version`; the source state is unchanged.
    pub state: CollectionState,
    /// The version this patch claims, always the current version plus one.
    pub version: u64,
}
impl std::fmt::Debug for BuiltPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BuiltPatch(<redacted>)")
    }
}

impl CollectionState {
    /// Build the next patch for this collection. This is the exact inverse of
    /// [`CollectionState::verify_patch`]: same version rule, same duplicate and
    /// missing-REMOVE refusals, same LT-hash accounting and the same MAC inputs,
    /// so a built patch re-verifies against the state it was built from.
    ///
    /// Nothing is persisted or sent here, and the returned state must not be
    /// saved until the server accepts the patch.
    pub fn build_patch(
        &self,
        keys: &MutationKeys,
        key_id: [u8; 6],
        device_index: u32,
        mutations: &[OutgoingMutation],
    ) -> Result<BuiltPatch> {
        if mutations.is_empty() {
            return Err(invalid("a patch needs at least one mutation"));
        }
        if mutations.len() > 4096 {
            return Err(invalid("patch exceeds local mutation bound"));
        }
        let version = self
            .version
            .checked_add(1)
            .ok_or_else(|| invalid("collection version overflow"))?;
        let mut next = self.clone();
        let mut wire_macs: Vec<[u8; 32]> = Vec::new();
        let mut wire_mutations = Vec::new();
        let mut removed = Vec::new();
        let mut added = Vec::new();
        let mut indexes = BTreeSet::new();
        for mutation in mutations {
            let mut iv = [0; 16];
            getrandom::fill(&mut iv).map_err(|_| invalid("mutation IV entropy unavailable"))?;
            let encrypted = keys.encrypt_mutation(
                mutation.operation,
                &key_id,
                &mutation.index,
                mutation.action.as_deref(),
                mutation.version,
                iv,
            )?;
            if !indexes.insert(encrypted.index_mac) {
                return Err(invalid("duplicate mutation index"));
            }
            match mutation.operation {
                MutationOperation::Set => {
                    if let Some(previous) =
                        next.values.insert(encrypted.index_mac, encrypted.value_mac)
                    {
                        removed.push(previous);
                    }
                    added.push(encrypted.value_mac);
                }
                MutationOperation::Remove => {
                    let previous = next
                        .values
                        .remove(&encrypted.index_mac)
                        .ok_or_else(|| invalid("REMOVE has no saved index in this collection"))?;
                    removed.push(previous);
                }
            }
            wire_macs.push(encrypted.value_mac);
            wire_mutations.push(WireMutation {
                operation: Some(match mutation.operation {
                    MutationOperation::Set => 0,
                    MutationOperation::Remove => 1,
                }),
                record: Some(Record {
                    index: Some(Blob {
                        value: Some(encrypted.index_mac.to_vec()),
                    }),
                    value: Some(Blob {
                        value: Some(encrypted.body.to_vec()),
                    }),
                    key_id: Some(Blob {
                        value: Some(key_id.to_vec()),
                    }),
                }),
            });
        }
        next.hash = self.hash.updated(&removed, &added)?;
        let snapshot_mac = keys.snapshot_mac(next.hash.as_bytes(), version, &self.collection)?;
        let patch_mac = keys.patch_mac(&snapshot_mac, &wire_macs, version, &self.collection)?;
        next.version = version;
        let wire = WirePatch {
            // `version` (field 1) is deliberately absent. **Confirmed**
            // (2.26.37.71): `LX/BYK->version_` has nine references in the whole
            // APK and every one is an `iget-object` — nothing ever writes it.
            // The outgoing patch is assembled by `LX/CRA->A00`, which sets
            // `mutations`, `keyId`, `snapshotMac`, `patchMac`, `deviceIndex`,
            // `clientDebugData` and (conditionally) `externalMutations`, and
            // never touches `version`. The version the patch claims still feeds
            // both MACs and the local state; it is simply not on the wire.
            //
            // Sending it is what our two refused live attempts did.
            version: None,
            mutations: wire_mutations,
            snapshot_mac: Some(snapshot_mac.to_vec()),
            patch_mac: Some(patch_mac.to_vec()),
            key_id: Some(Blob {
                value: Some(key_id.to_vec()),
            }),
            device_index: Some(device_index),
            // `clientDebugData` (field 9) is traced but **not implemented**.
            // `LX/CRA->A00` builds it only when it holds all three of a current
            // LT hash, a version and a new LT hash, and it is a nested `LX/BYb`
            // whose field numbers are, from its `newMessageInfo` descriptor:
            // 1 `currentLthash` (bytes), 2 `newLthash` (bytes),
            // 3 `patchVersion` (8-byte big-endian of the version),
            // 4 `collectionName` (UTF-8 bytes),
            // 5 `firstFourBytesFromAHashOfSnapshotMacKey` (never set there),
            // 6 `newLthashSubtract` (bytes, optional), 7 `numberAdd`,
            // 8 `numberRemove`, 9 `numberOverride`, 10 `senderPlatform`
            // (Android), 11 `isSenderPrimary`.
            //
            // **Unknown:** whether `newLthash`/`newLthashSubtract` are the full
            // post-patch hash and a delta, or the addition and subtraction
            // terms — `LX/CRA`'s constructor leaves `currentLthash` null and
            // later setters fill it, and those were not read. Guessing would
            // put wrong integrity data in a field the server may check, so it
            // stays absent until that is traced.
            client_debug_data: None,
        };
        Ok(BuiltPatch {
            bytes: wire.encode_to_vec(),
            state: next,
            version,
        })
    }
}

#[cfg(test)]
mod build_tests {
    use super::*;

    /// The APK never writes `SyncdPatch.version`: all nine references to
    /// `LX/BYK->version_` are reads. Sending it is what our two refused live
    /// attempts did, so its absence is asserted on the wire rather than left to
    /// a comment.
    #[test]
    fn an_outgoing_patch_carries_no_version_field() {
        use prost::encoding::{DecodeContext, decode_key, skip_field};
        let keys = MutationKeys::derive(&[9u8; 32]).unwrap();
        let built = CollectionState::empty("regular_high")
            .unwrap()
            .build_patch(
                &keys,
                [0, 1, 0, 0, 0, 2],
                0,
                &[set(r#"["contact","a@s.whatsapp.net"]"#)],
            )
            .unwrap();
        let mut bytes = built.bytes.as_slice();
        let mut tags = Vec::new();
        while !bytes.is_empty() {
            let (tag, wire) = decode_key(&mut bytes).unwrap();
            tags.push(tag);
            skip_field(wire, tag, &mut bytes, DecodeContext::default()).unwrap();
        }
        assert!(!tags.contains(&1), "field 1 (version) must not be emitted");
        // The fields LX/CRA->A00 does set are all present.
        for tag in [2, 4, 5, 6, 8] {
            assert!(tags.contains(&tag), "field {tag} must be emitted");
        }
        // The version still reaches the MACs and the local state.
        assert_eq!(built.version, 1);
        assert_eq!(built.state.version, 1);
    }

    /// Our own outgoing patch deliberately omits `version` (field 1), because
    /// the APK's builder never writes it. The incoming parser still requires it,
    /// since a server-sent patch always carries one — so to verify our output
    /// with that same parser the tests put the version back on the wire, which
    /// is exactly what the server would be doing on its side.
    fn with_version(bytes: &[u8], version: u64) -> Vec<u8> {
        let mut inner = Vec::new();
        prost::encoding::uint64::encode(1, &version, &mut inner);
        let mut out = Vec::new();
        prost::encoding::encode_key(1, prost::encoding::WireType::LengthDelimited, &mut out);
        prost::encoding::encode_varint(inner.len() as u64, &mut out);
        out.extend_from_slice(&inner);
        out.extend_from_slice(bytes);
        out
    }

    fn action(ts: i64) -> Vec<u8> {
        #[derive(Message)]
        struct Header {
            #[prost(int64, optional, tag = "1")]
            timestamp: Option<i64>,
        }
        Header {
            timestamp: Some(ts),
        }
        .encode_to_vec()
    }

    fn set(index: &str) -> OutgoingMutation {
        OutgoingMutation {
            operation: MutationOperation::Set,
            index: index.as_bytes().to_vec(),
            action: Some(action(1_700_000_000_000)),
            version: 7,
        }
    }

    /// A built patch must satisfy the verifier it was modelled on, including the
    /// LT-hash-derived snapshot MAC and the ordered patch MAC. `verify_patch`
    /// was written and vectored before any of the write path existed, so this is
    /// a real cross-check rather than the builder agreeing with itself.
    #[test]
    fn a_built_patch_verifies_against_the_state_it_was_built_from() {
        let keys = MutationKeys::derive(&[9u8; 32]).unwrap();
        let key_id = [0, 1, 0, 0, 0, 2];
        let state = CollectionState::empty("regular_high").unwrap();

        let built = state
            .build_patch(
                &keys,
                key_id,
                0,
                &[set(r#"["contact","a@s.whatsapp.net"]"#)],
            )
            .unwrap();
        assert_eq!(built.version, 1);

        let decoded = CollectionPatch::decode(&with_version(&built.bytes, built.version)).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.key_id, key_id);
        let verified = state
            .verify_patch(&decoded, |id| {
                assert_eq!(id, &key_id);
                Ok(&keys)
            })
            .unwrap();
        // The verifier's resulting state matches what the builder predicted.
        assert_eq!(verified.state, built.state);
        assert_eq!(verified.state.version(), 1);
        assert_eq!(verified.state.len(), 1);
        assert_eq!(verified.mutations.len(), 1);
        assert_eq!(
            verified.mutations[0].index(),
            br#"["contact","a@s.whatsapp.net"]"#
        );
    }

    /// Several mutations in one patch, then a REMOVE in a following patch, so
    /// the replace-and-remove LT-hash accounting is exercised end to end.
    #[test]
    fn sequential_patches_replace_and_remove_consistently() {
        let keys = MutationKeys::derive(&[4u8; 32]).unwrap();
        let key_id = [0, 1, 0, 0, 0, 3];
        let empty = CollectionState::empty("regular").unwrap();
        let verify = |state: &CollectionState, built: &BuiltPatch| {
            let decoded =
                CollectionPatch::decode(&with_version(&built.bytes, built.version)).unwrap();
            let verified = state.verify_patch(&decoded, |_| Ok(&keys)).unwrap();
            assert_eq!(verified.state, built.state);
            verified.state
        };

        let first = empty
            .build_patch(
                &keys,
                key_id,
                0,
                &[set(r#"["c","a"]"#), set(r#"["c","b"]"#)],
            )
            .unwrap();
        let after_first = verify(&empty, &first);
        assert_eq!(after_first.len(), 2);

        // Replacing an existing index keeps the count and still verifies: the
        // old value MAC must be subtracted from the hash.
        let second = after_first
            .build_patch(&keys, key_id, 0, &[set(r#"["c","a"]"#)])
            .unwrap();
        let after_second = verify(&after_first, &second);
        assert_eq!(after_second.len(), 2);
        assert_eq!(after_second.version(), 2);

        let third = after_second
            .build_patch(
                &keys,
                key_id,
                0,
                &[OutgoingMutation {
                    operation: MutationOperation::Remove,
                    index: br#"["c","b"]"#.to_vec(),
                    action: None,
                    version: 7,
                }],
            )
            .unwrap();
        let after_third = verify(&after_second, &third);
        assert_eq!(after_third.len(), 1);
        assert_eq!(after_third.version(), 3);
    }

    #[test]
    fn the_builder_refuses_what_the_verifier_would_reject() {
        let keys = MutationKeys::derive(&[2u8; 32]).unwrap();
        let key_id = [0; 6];
        let state = CollectionState::empty("regular").unwrap();
        // Empty patch.
        assert!(state.build_patch(&keys, key_id, 0, &[]).is_err());
        // Duplicate index within one patch.
        assert!(
            state
                .build_patch(
                    &keys,
                    key_id,
                    0,
                    &[set(r#"["c","a"]"#), set(r#"["c","a"]"#)]
                )
                .is_err()
        );
        // REMOVE of an index this collection does not hold.
        assert!(
            state
                .build_patch(
                    &keys,
                    key_id,
                    0,
                    &[OutgoingMutation {
                        operation: MutationOperation::Remove,
                        index: br#"["c","missing"]"#.to_vec(),
                        action: None,
                        version: 7,
                    }]
                )
                .is_err()
        );
        // A SET without an action.
        assert!(
            state
                .build_patch(
                    &keys,
                    key_id,
                    0,
                    &[OutgoingMutation {
                        operation: MutationOperation::Set,
                        index: br#"["c","a"]"#.to_vec(),
                        action: None,
                        version: 7,
                    }]
                )
                .is_err()
        );
    }

    /// A patch is bound to its collection: the same bytes must not verify as
    /// another collection's patch, since the name is inside both MACs.
    #[test]
    fn a_patch_does_not_verify_under_another_collection() {
        let keys = MutationKeys::derive(&[6u8; 32]).unwrap();
        let key_id = [0; 6];
        let built = CollectionState::empty("regular")
            .unwrap()
            .build_patch(&keys, key_id, 0, &[set(r#"["c","a"]"#)])
            .unwrap();
        let decoded = CollectionPatch::decode(&with_version(&built.bytes, built.version)).unwrap();
        let other = CollectionState::empty("regular_high").unwrap();
        assert!(other.verify_patch(&decoded, |_| Ok(&keys)).is_err());
    }
}
