//! Pure structural headers, before keys, authentication or state application.
//! APK 2.26.43.72 F2T.A00 and Dh0/Dg5 descriptors; see the retained
//! app-state-patch-header-audit evidence. Local guards are not server rules.
use super::response::CollectionData;
use std::{collections::BTreeSet, fmt};

/// A structural verdict never establishes authenticity or permission to apply.
pub struct CollectionHeaders<'a> {
    /// Original snapshot reference; authenticated retrieval remains required.
    pub snapshot_ref: Option<&'a [u8]>,
    /// Original patch order, after whole-collection structural validation.
    pub patches: Vec<PatchHeader<'a>>,
    /// Original continuation indication, not completion proof.
    pub has_more_patches: bool,
}
/// Borrowed patch header. Body contents are deliberately absent from Debug.
pub struct PatchHeader<'a> {
    /// Unmodified original protobuf, including unknown fields.
    pub original: &'a [u8],
    /// Structurally present version, not an authenticated version.
    pub version: u64,
    /// Number of inline mutation messages, whose nested syntax was checked.
    pub inline_mutations: usize,
    /// External-only data is deferred, never treated as an empty inline patch.
    pub source: PatchSource,
}
/// Structural source only; neither variant authorizes application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchSource {
    /// Inline mutation source, still unauthenticated.
    Inline,
    /// External reference requires authenticated retrieval.
    ExternalDeferred,
}
/// Failure origin remains explicit when parser parity is unproved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderFailure {
    /// Client admission restriction, with no invented native reason.
    LocalPolicy(LocalHeaderFailure),
    /// Corresponding native F2T fatal reason, without private payload text.
    NativeFatal {
        /// Native F2T diagnostic number.
        reason: u32,
        /// Original zero-based patch position, absent for snapshot syntax.
        patch_index: Option<usize>,
    },
}
/// Defensive restrictions; these are not recovered native protocol rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalHeaderFailure {
    /// Local byte, item or field budget exceeded.
    Limit,
    /// Recognized singleton repeated; native merge parity is unproved.
    DuplicateSingleton,
    /// Encoding outside the locally supported protobuf subset.
    UnsupportedEncoding,
}
impl HeaderFailure {
    /// APK diagnostic reason, when parity is established for this failure.
    pub fn native_reason(&self) -> Option<u32> {
        match self {
            Self::NativeFatal { reason, .. } => Some(*reason),
            _ => None,
        }
    }
}
impl fmt::Debug for CollectionHeaders<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollectionHeaders")
            .field("has_snapshot", &self.snapshot_ref.is_some())
            .field("patches", &self.patches)
            .field("has_more_patches", &self.has_more_patches)
            .finish()
    }
}
impl fmt::Debug for PatchHeader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PatchHeader")
            .field("inline_mutations", &self.inline_mutations)
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Copy)]
enum Shape {
    Patch,
    Version,
    Reference,
    Mutation,
    Record,
    BytesWrapper,
    Exit,
}
#[derive(Default)]
struct Parsed {
    version: Option<u64>,
    count: usize,
    external: bool,
    exit: Option<Option<u64>>,
}
enum ParseFailure {
    Malformed,
    Local(LocalHeaderFailure),
}
fn local(v: LocalHeaderFailure) -> ParseFailure {
    ParseFailure::Local(v)
}
fn varint(b: &[u8], at: &mut usize) -> Result<u64, ParseFailure> {
    let mut value = 0;
    for i in 0..10 {
        let x = *b.get(*at).ok_or(ParseFailure::Malformed)?;
        *at += 1;
        if i == 9 && x > 1 {
            return Err(local(LocalHeaderFailure::UnsupportedEncoding));
        }
        value |= u64::from(x & 127) << (7 * i);
        if x < 128 {
            return Ok(value);
        }
    }
    Err(local(LocalHeaderFailure::UnsupportedEncoding))
}
// Exact recognized nested types come from the current APK descriptors, not
// guessed field numbers. Strings are checked locally for UTF-8; native invalid
// UTF-8 replacement behavior has not been established.
fn parse(b: &[u8], shape: Shape, budget: &mut usize) -> Result<Parsed, ParseFailure> {
    let mut out = Parsed::default();
    let mut at = 0;
    let mut seen = BTreeSet::new();
    while at < b.len() {
        *budget = budget
            .checked_sub(1)
            .ok_or(local(LocalHeaderFailure::Limit))?;
        let key = varint(b, &mut at)?;
        let tag = key >> 3;
        let wire = key & 7;
        if key > u64::from(u32::MAX) {
            return Err(local(LocalHeaderFailure::UnsupportedEncoding));
        }
        if tag == 0 {
            return Err(ParseFailure::Malformed);
        }
        let (expected, nested, repeated, string) = match (shape, tag) {
            (Shape::Patch, 1) => (Some(2), Some(Shape::Version), false, false),
            (Shape::Patch, 2) => (Some(2), Some(Shape::Mutation), true, false),
            (Shape::Patch, 3) => (Some(2), Some(Shape::Reference), false, false),
            (Shape::Patch, 6) | (Shape::Record, 1..=3) => {
                (Some(2), Some(Shape::BytesWrapper), false, false)
            }
            (Shape::Patch, 7) => (Some(2), Some(Shape::Exit), false, false),
            (Shape::Patch, 8)
            | (Shape::Version, 1)
            | (Shape::Exit, 1)
            | (Shape::Reference, 4)
            | (Shape::Mutation, 1) => (Some(0), None, false, false),
            (Shape::Mutation, 2) => (Some(2), Some(Shape::Record), false, false),
            (Shape::Reference, 2..=3) | (Shape::Exit, 2) => (Some(2), None, false, true),
            (Shape::Patch, 4..=5 | 9)
            | (Shape::Reference, 1 | 5..=6)
            | (Shape::BytesWrapper, 1) => (Some(2), None, false, false),
            _ => (None, None, false, false),
        };
        if expected.is_some_and(|w| w != wire) {
            return Err(local(LocalHeaderFailure::UnsupportedEncoding));
        }
        if expected.is_some() && !repeated && !seen.insert(tag) {
            return Err(local(LocalHeaderFailure::DuplicateSingleton));
        }
        let mut scalar = None;
        let body = match wire {
            0 => {
                scalar = Some(varint(b, &mut at)?);
                &b[at..at]
            }
            1 | 5 => {
                let end = at
                    .checked_add(if wire == 1 { 8 } else { 4 })
                    .ok_or(ParseFailure::Malformed)?;
                let value = b.get(at..end).ok_or(ParseFailure::Malformed)?;
                at = end;
                value
            }
            2 => {
                let len = usize::try_from(varint(b, &mut at)?)
                    .map_err(|_| local(LocalHeaderFailure::Limit))?;
                let end = at.checked_add(len).ok_or(ParseFailure::Malformed)?;
                let value = b.get(at..end).ok_or(ParseFailure::Malformed)?;
                at = end;
                value
            }
            _ => return Err(local(LocalHeaderFailure::UnsupportedEncoding)),
        };
        if string && std::str::from_utf8(body).is_err() {
            return Err(local(LocalHeaderFailure::UnsupportedEncoding));
        }
        let sub = nested.map(|s| parse(body, s, budget)).transpose()?;
        match (shape, tag) {
            (Shape::Patch, 1) => out.version = sub.and_then(|s| s.version),
            (Shape::Version, 1) | (Shape::Exit, 1) => out.version = scalar,
            (Shape::Patch, 2) => out.count += 1,
            (Shape::Patch, 3) => out.external = true,
            (Shape::Patch, 7) => out.exit = Some(sub.and_then(|s| s.version)),
            _ => {}
        }
    }
    Ok(out)
}
fn failure(e: ParseFailure, reason: u32, patch_index: Option<usize>) -> HeaderFailure {
    match e {
        ParseFailure::Malformed => HeaderFailure::NativeFatal {
            reason,
            patch_index,
        },
        ParseFailure::Local(v) => HeaderFailure::LocalPolicy(v),
    }
}
/// Check snapshot-reference syntax first, then every selected patch's recognized
/// nested protobuf before semantic checks. No keys, downloads or state writes.
/// Unknown scalar fields survive in the borrowed originals. Unsupported parser
/// encodings are local refusals, not claims of exact native parser equivalence.
pub fn inspect_collection_headers<'a>(
    data: &CollectionData<'a>,
) -> Result<CollectionHeaders<'a>, HeaderFailure> {
    let bytes = data
        .patches
        .iter()
        .try_fold(data.snapshot_ref.map_or(0, <[u8]>::len), |n, b| {
            n.checked_add(b.len())
        });
    if data.patches.len() > 4096 || bytes.is_none_or(|n| n > 16 * 1024 * 1024) {
        return Err(HeaderFailure::LocalPolicy(LocalHeaderFailure::Limit));
    }
    let mut budget = 262144;
    if let Some(reference) = data.snapshot_ref {
        parse(reference, Shape::Reference, &mut budget).map_err(|e| failure(e, 69, None))?;
    }
    let mut parsed = Vec::with_capacity(data.patches.len());
    for (i, b) in data.patches.iter().enumerate() {
        parsed.push(parse(b, Shape::Patch, &mut budget).map_err(|e| failure(e, 18, Some(i)))?);
    }
    let mut versions = BTreeSet::new();
    let mut patches = Vec::with_capacity(parsed.len());
    for (i, p) in parsed.into_iter().enumerate() {
        let fatal = |reason| HeaderFailure::NativeFatal {
            reason,
            patch_index: Some(i),
        };
        if let Some(code) = p.exit {
            return Err(fatal(match code {
                Some(100) => 36,
                Some(101) => 37,
                _ => 38,
            }));
        }
        let version = p.version.ok_or_else(|| fatal(8))?;
        if p.external && p.count > 0 {
            return Err(fatal(14));
        }
        if !versions.insert(version) {
            return Err(fatal(9));
        }
        patches.push(PatchHeader {
            original: data.patches[i],
            version,
            inline_mutations: p.count,
            source: if p.external {
                PatchSource::ExternalDeferred
            } else {
                PatchSource::Inline
            },
        });
    }
    Ok(CollectionHeaders {
        snapshot_ref: data.snapshot_ref,
        patches,
        has_more_patches: data.has_more_patches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inspect<'a>(
        patches: Vec<&'a [u8]>,
        snapshot_ref: Option<&'a [u8]>,
    ) -> Result<CollectionHeaders<'a>, HeaderFailure> {
        inspect_collection_headers(&CollectionData {
            patches,
            snapshot_ref,
            has_more_patches: false,
        })
    }
    fn reason(patches: Vec<&[u8]>) -> Option<u32> {
        inspect(patches, None).unwrap_err().native_reason()
    }
    #[test]
    fn native_semantics_and_parse_first_priority() {
        let good = [10, 2, 8, 1];
        assert_eq!(inspect(vec![&good], None).unwrap().patches[0].version, 1);
        assert_eq!(reason(vec![&good, &good]), Some(9));
        assert_eq!(reason(vec![&[]]), Some(8));
        for (code, want) in [(100, 36), (101, 37), (99, 38)] {
            let exit = [58, 2, 8, code];
            assert_eq!(reason(vec![&exit]), Some(want));
            // Native parses the entire list before inspecting any exit header.
            assert_eq!(reason(vec![&exit, &[10, 1, 8]]), Some(18));
        }
        assert_eq!(reason(vec![&[58, 0]]), Some(38));
        assert_eq!(reason(vec![&[10, 0]]), Some(8));
    }
    #[test]
    fn recursively_checks_every_recognized_message_and_snapshot_first() {
        // version + mutation(record(index(blob length=2, but truncated))).
        let broken = [10, 2, 8, 1, 18, 7, 18, 5, 10, 3, 10, 2, 1];
        assert_eq!(reason(vec![&broken]), Some(18));
        assert_eq!(
            inspect(vec![&broken], Some(&[10, 2, 1]))
                .unwrap_err()
                .native_reason(),
            Some(69)
        );
        // clientDebugData is opaque bytes, not another protobuf message.
        assert!(inspect(vec![&[10, 2, 8, 1, 74, 1, 255]], None).is_ok());
        // Dg5 file size is uint64; malformed nested reference must be parsed.
        assert_eq!(reason(vec![&[10, 2, 8, 1, 26, 1, 32]]), Some(18));
    }
    #[test]
    fn external_is_deferred_and_mixed_is_fatal() {
        let external = [10, 2, 8, 1, 26, 0];
        assert_eq!(
            inspect(vec![&external], None).unwrap().patches[0].source,
            PatchSource::ExternalDeferred
        );
        assert_eq!(reason(vec![&[10, 2, 8, 1, 26, 0, 18, 0]]), Some(14));
    }
    #[test]
    fn local_guards_do_not_invent_native_reasons() {
        for bytes in [
            &[10, 2, 8, 1, 10, 2, 8, 2][..],
            &[8, 1][..],
            &[10, 2, 8, 1, 26, 3, 18, 1, 255][..],
            &[11, 12][..],
        ] {
            let err = inspect(vec![bytes], None).unwrap_err();
            assert!(matches!(err, HeaderFailure::LocalPolicy(_)));
            assert_eq!(err.native_reason(), None);
        }
    }
    #[test]
    fn originals_unknowns_and_debug_are_preserved_and_redacted() {
        let body = [10, 2, 8, 77, 162, 6, 6, b'S', b'E', b'C', b'R', b'E', b'T'];
        let out = inspect(
            vec![&body],
            Some(&[18, 6, b'S', b'E', b'C', b'R', b'E', b'T']),
        )
        .unwrap();
        assert_eq!(out.patches[0].original.as_ptr(), body.as_ptr());
        assert!(!format!("{out:?}").contains("SECRET"));
        assert!(!format!("{out:?}").contains("77"));
    }
}
