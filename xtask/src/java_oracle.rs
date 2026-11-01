//! Drivers for the independent desktop Java protobuf oracles
//! (`docs/protocol/AdvDetailsProtoOracle.java`, `ProtocolEnumsOracle.java`).
//!
//! The case inputs are built here and piped to `java`; every expected value
//! comes from the pinned protobuf-javalite runtime, never from the Rust
//! parsers the fixtures check. Ports of the former Python generators: the
//! case sets, their order and the JSON layout are byte-identical.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;
use sha2::{Digest, Sha256};

/// protobuf-javalite 4.36.1 from Maven Central.
const JAR_SHA256: &str = "91d3dba2521322103230c509d41a017da3a5e6766610adcb10c5bb127cbb26c5";
const JAR_URL: &str = "https://repo.maven.apache.org/maven2/com/google/protobuf/protobuf-javalite/4.36.1/protobuf-javalite-4.36.1.jar";

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Oracle {
    /// GsB (ADVDeviceIdentity) parse/serialization vectors.
    AdvDetails,
    /// Enum field and presence vectors (BZb/BV3, CvY/C4F, Cv0/C2L).
    ProtocolEnums,
}

pub fn run(oracle: Oracle, classpath: &str, output: &Path) -> Result<(), String> {
    verify_jar(classpath)?;
    let json = match oracle {
        Oracle::AdvDetails => serde_json::to_string_pretty(&adv_details(classpath)?),
        Oracle::ProtocolEnums => serde_json::to_string_pretty(&protocol_enums(classpath)?),
    };
    let mut json = json.map_err(|e| e.to_string())?;
    json.push('\n');
    std::fs::write(output, json).map_err(|e| format!("{}: {e}", output.display()))?;
    println!("wrote {}", output.display());
    Ok(())
}

fn verify_jar(classpath: &str) -> Result<(), String> {
    let jars: Vec<_> = std::env::split_paths(classpath)
        .filter(|p| p.extension().is_some_and(|e| e == "jar"))
        .collect();
    let [jar] = jars.as_slice() else {
        return Err("classpath must contain exactly one .jar (protobuf-javalite 4.36.1)".into());
    };
    let bytes = std::fs::read(jar).map_err(|e| format!("{}: {e}", jar.display()))?;
    if hex::encode(Sha256::digest(&bytes)) != JAR_SHA256 {
        return Err(format!(
            "{} is not the pinned protobuf-javalite 4.36.1 ({JAR_URL})",
            jar.display()
        ));
    }
    Ok(())
}

/// Run `java -cp CLASSPATH CLASS` with one input line per case and require
/// exactly one output line per case.
fn java(classpath: &str, class: &str, input: &[String]) -> Result<Vec<String>, String> {
    let mut child = Command::new("java")
        .args(["-cp", classpath, class])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("java: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("java stdin")?;
    let text: String = input.iter().map(|line| format!("{line}\n")).collect();
    // Write on a thread so a large stdout cannot deadlock against stdin.
    let writer = std::thread::spawn(move || stdin.write_all(text.as_bytes()));
    let out = child.wait_with_output().map_err(|e| format!("java: {e}"))?;
    writer
        .join()
        .map_err(|_| "java stdin writer panicked")?
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("java {class} exited with {}", out.status));
    }
    let lines: Vec<String> = String::from_utf8(out.stdout)
        .map_err(|_| "java output is not UTF-8")?
        .lines()
        .map(str::to_owned)
        .collect();
    if lines.len() != input.len() {
        return Err(format!(
            "java {class}: {} outputs for {} cases",
            lines.len(),
            input.len()
        ));
    }
    Ok(lines)
}

fn integer(text: &str) -> Result<serde_json::Number, String> {
    text.parse::<i64>()
        .map(Into::into)
        .or_else(|_| text.parse::<u64>().map(Into::into))
        .map_err(|_| format!("oracle printed a non-integer: {text:?}"))
}

fn varint(mut n: u64) -> Vec<u8> {
    let mut out = Vec::new();
    while n > 127 {
        out.push((n & 127) as u8 | 128);
        n >>= 7;
    }
    out.push(n as u8);
    out
}

fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

#[derive(Serialize)]
struct AdvFixture {
    apk_build: &'static str,
    source: &'static str,
    oracle: &'static str,
    jar_sha256: &'static str,
    cases: Vec<AdvRow>,
}

#[derive(Serialize)]
struct AdvRow {
    hex: String,
    serialized: Option<String>,
    fields: Option<Vec<Option<serde_json::Number>>>,
}

fn adv_cases() -> BTreeSet<Vec<u8>> {
    let mut cases: BTreeSet<Vec<u8>> = [
        vec![],
        vec![0x08],
        vec![0x00],
        vec![0x80; 10],
        cat(&[&[0x08], &[0xff; 9], &[0x7f]]),
        cat(&[&[0x08], &[0x80; 10]]),
        cat(&[&[0x32], &varint(0xffff_ffff)]),
    ]
    .into();
    let fields: [&[u8]; 5] = [
        b"\x08\x07",
        b"\x10\x09",
        b"\x18\x02",
        b"\x20\x00",
        b"\x28\x00",
    ];
    permutations(&fields, &mut Vec::new(), &mut [false; 5], &mut cases);
    let base: &[u8] = b"\x08\x07\x10\x09\x18\x02";
    let values: [u64; 12] = [
        0,
        1,
        2,
        3,
        127,
        128,
        255,
        0xffff_ffff,
        0x1_0000_0000,
        0x1_0000_0001,
        0x7fff_ffff_ffff_ffff,
        u64::MAX,
    ];
    for tag in [8u8, 16, 24, 32, 40, 48] {
        for &n in &values {
            let field = cat(&[&[tag], &varint(n)]);
            for prefix in [&[][..], base, &cat(&[base, &field])] {
                cases.insert(cat(&[prefix, &field]));
            }
            if n < 128 {
                cases.insert(cat(&[base, &[tag], &[n as u8 | 128, 0]]));
            }
        }
    }
    let unknowns: Vec<Vec<u8>> = vec![
        b"\x30\x81\x00".to_vec(),
        cat(&[b"\x39", &(0..8).collect::<Vec<u8>>()]),
        b"\x42\x03abc".to_vec(),
        b"\x4b\x08\x87\x00\x4c".to_vec(),
        cat(&[b"\x55", &(0..4).collect::<Vec<u8>>()]),
        b"\x4c".to_vec(),
        b"\x4b\x54".to_vec(),
        b"\x4b".to_vec(),
        b"\x42\x05abc".to_vec(),
        b"\x0a\x01x".to_vec(),
    ];
    for u in &unknowns {
        for p in [&[][..], base, &cat(&[base, u])] {
            cases.insert(cat(&[p, u]));
        }
        for p in [&[][..], base] {
            cases.insert(cat(&[u, p]));
        }
    }
    for depth in [1, 2, 98, 99, 100] {
        cases.insert(cat(&[
            base,
            &vec![0x4b; depth],
            b"\x08\x07",
            &vec![0x4c; depth],
        ]));
    }
    // random.Random(3771) as in the former generator, call for call.
    let mut rng = PyRandom::new(3771);
    let tags = [8u8, 16, 24, 32, 40];
    for _ in 0..250 {
        let count = 1 + rng.below(8); // randrange(1, 9)
        let mut chunks = Vec::new();
        for _ in 0..count {
            // Python evaluates the inner choices before the outer choice.
            let tag = tags[rng.below(tags.len())];
            let value = values[rng.below(values.len())];
            let random_field = cat(&[&[tag], &varint(value)]);
            let pool: Vec<&[u8]> = fields
                .iter()
                .copied()
                .chain(unknowns.iter().map(Vec::as_slice))
                .chain([random_field.as_slice()])
                .collect();
            chunks.extend_from_slice(pool[rng.below(pool.len())]);
        }
        cases.insert(chunks);
    }
    cases
}

fn permutations(
    items: &[&[u8]],
    current: &mut Vec<u8>,
    used: &mut [bool; 5],
    out: &mut BTreeSet<Vec<u8>>,
) {
    if used.iter().all(|&u| u) {
        out.insert(current.clone());
        return;
    }
    for i in 0..items.len() {
        if !used[i] {
            used[i] = true;
            let len = current.len();
            current.extend_from_slice(items[i]);
            permutations(items, current, used, out);
            current.truncate(len);
            used[i] = false;
        }
    }
}

fn adv_details(classpath: &str) -> Result<AdvFixture, String> {
    let ordered: Vec<Vec<u8>> = adv_cases().into_iter().collect();
    let input: Vec<String> = ordered.iter().map(hex::encode).collect();
    let lines = java(classpath, "AdvDetailsProtoOracle", &input)?;
    let mut cases = Vec::new();
    for (hex, line) in input.into_iter().zip(lines) {
        let (serialized, fields) = if line == "malformed" {
            (None, None)
        } else {
            let mut parts = line.split('|');
            let serialized = parts.next().unwrap_or_default().to_owned();
            let fields = parts
                .map(|x| {
                    if x == "null" {
                        Ok(None)
                    } else {
                        integer(x).map(Some)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            (Some(serialized), Some(fields))
        };
        cases.push(AdvRow {
            hex,
            serialized,
            fields,
        });
    }
    Ok(AdvFixture {
        apk_build: "2.26.37.71",
        source: "GsB.dynamicMethod plus IQg/Azc enum 0..2",
        oracle: "desktop protobuf-javalite 4.36.1, not Android execution",
        jar_sha256: JAR_SHA256,
        cases,
    })
}

#[derive(Serialize)]
struct EnumFixture {
    apk_build: &'static str,
    apk_sha256: &'static str,
    scope: &'static str,
    source: &'static str,
    oracle: &'static str,
    runtime: &'static str,
    runtime_url: &'static str,
    runtime_sha256: &'static str,
    cases: Vec<EnumRow>,
}

#[derive(Serialize)]
struct EnumRow {
    mode: &'static str,
    hex: String,
    valid: bool,
    value: Option<serde_json::Number>,
}

fn enum_cases() -> BTreeSet<(&'static str, Vec<u8>)> {
    let field = |n: u64| cat(&[b"\x10", &varint(n)]);
    let wrap = |mode: &str, body: &[u8]| {
        let tag: u64 = if mode == "p" { 12 } else { 51 };
        cat(&[&varint(tag << 3 | 2), &varint(body.len() as u64), body])
    };
    let mut values: Vec<u64> = (0..65).collect();
    values.extend([
        127,
        128,
        255,
        0x8000_0000,
        0xffff_ffff,
        0x1_0000_0000,
        0x1_0000_0001,
        0x1_0000_0003,
        0x1_0000_0027,
        0x7fff_ffff_ffff_ffff,
        u64::MAX,
    ]);
    let mut cases = BTreeSet::new();
    // BTreeSet<(&str, Vec<u8>)> sorts like Python's sorted((mode, bytes)).
    for mode in ["p", "k"] {
        let mut add = |data: Vec<u8>| {
            cases.insert((mode, data));
        };
        add(vec![]);
        add(wrap(mode, &[]));
        for &value in &values {
            let current = field(value);
            add(wrap(mode, &current));
            let priors: [u64; 3] = if mode == "p" { [0, 3, 4] } else { [0, 1, 2] };
            for prior in priors {
                let previous = field(prior);
                add(wrap(mode, &cat(&[&previous, &current])));
                add(wrap(mode, &cat(&[&current, &previous])));
                add(cat(&[&wrap(mode, &previous), &wrap(mode, &current)]));
                add(cat(&[&wrap(mode, &current), &wrap(mode, &previous)]));
            }
        }
        let base = field(if mode == "p" { 3 } else { 1 });
        let unknowns: Vec<Vec<u8>> = vec![
            b"\x30\x01".to_vec(),
            cat(&[b"\x39", &[0; 8]]),
            b"\x42\x03abc".to_vec(),
            b"\x4b\x10\x27\x4c".to_vec(),
            cat(&[b"\x55", &[0; 4]]),
            b"\x4c".to_vec(),
            b"\x4b\x54".to_vec(),
            b"\x4b".to_vec(),
            b"\x42\x05abc".to_vec(),
            b"\x00".to_vec(),
            b"\x10".to_vec(),
            b"\x10\x80".to_vec(),
            b"\x0e".to_vec(),
            b"\x0f".to_vec(),
            vec![0x80; 10],
            b"\x12\x01\xff".to_vec(),
            cat(&[b"\x11", &[0; 8]]),
            cat(&[b"\x15", &[0; 4]]),
            b"\x13\x10\x00\x14".to_vec(),
        ];
        for unknown in &unknowns {
            add(wrap(mode, &cat(&[&base, unknown])));
            add(wrap(mode, &cat(&[unknown, &base])));
        }
        for depth in [1, 2, 97, 98, 99, 100] {
            add(wrap(
                mode,
                &cat(&[&base, &vec![0x4b; depth], &field(39), &vec![0x4c; depth]]),
            ));
        }
        for invalid in [&b"\x00"[..], b"\x80", b"\x0c", b"\x0b", b"\x0b\x14"] {
            add(invalid.to_vec());
        }
    }
    cases
}

fn protocol_enums(classpath: &str) -> Result<EnumFixture, String> {
    let ordered: Vec<(&str, Vec<u8>)> = enum_cases().into_iter().collect();
    let input: Vec<String> = ordered
        .iter()
        .map(|(mode, data)| format!("{mode}|{}", hex::encode(data)))
        .collect();
    let lines = java(classpath, "ProtocolEnumsOracle", &input)?;
    let mut cases = Vec::new();
    for ((mode, data), line) in ordered.into_iter().zip(lines) {
        let value = match line.as_str() {
            "absent" | "malformed" => None,
            other => Some(integer(other)?),
        };
        cases.push(EnumRow {
            mode,
            hex: hex::encode(data),
            valid: line != "malformed",
            value,
        });
    }
    Ok(EnumFixture {
        apk_build: "2.26.37.71",
        apk_sha256: "1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c",
        scope: "Projected enum fields/presence and singular-message merge, not full APK schemas or execution",
        source: "BZb/BV3 field 2 and CvY/C4F, Cv0/C2L enum verifiers",
        oracle: "docs/protocol/ProtocolEnumsOracle.java",
        runtime: "protobuf-javalite 4.36.1 on desktop Java; APK runtime version unknown",
        runtime_url: JAR_URL,
        runtime_sha256: JAR_SHA256,
        cases,
    })
}

/// CPython's `random.Random(int)`: MT19937 seeded by `init_by_array` with the
/// seed's 32-bit words, and `_randbelow` by rejection over `getrandbits`.
struct PyRandom {
    state: [u32; 624],
    index: usize,
}

impl PyRandom {
    fn new(seed: u32) -> Self {
        let mut mt = [0u32; 624];
        mt[0] = 19_650_218;
        for i in 1..624 {
            mt[i] = 1_812_433_253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        // init_by_array with key = [seed].
        let key = [seed];
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..624.max(key.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= 624 {
                mt[0] = mt[623];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..623 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_566_083_941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                mt[0] = mt[623];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
        Self {
            state: mt,
            index: 624,
        }
    }

    fn next_u32(&mut self) -> u32 {
        if self.index >= 624 {
            for k in 0..624 {
                let y = (self.state[k] & 0x8000_0000) | (self.state[(k + 1) % 624] & 0x7fff_ffff);
                let mut next = self.state[(k + 397) % 624] ^ (y >> 1);
                if y & 1 != 0 {
                    next ^= 0x9908_b0df;
                }
                self.state[k] = next;
            }
            self.index = 0;
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// `_randbelow_with_getrandbits(n)` for 0 < n < 2**32.
    fn below(&mut self, n: usize) -> usize {
        let k = usize::BITS - n.leading_zeros();
        loop {
            let r = (self.next_u32() >> (32 - k)) as usize;
            if r < n {
                return r;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn py_random_matches_cpython() {
        // python3 -c 'import random; r=random.Random(3771);
        //   print([r.getrandbits(32) for _ in range(3)])'
        let mut rng = PyRandom::new(3771);
        let words: Vec<u32> = (0..3).map(|_| rng.next_u32()).collect();
        assert_eq!(words, [2_911_557_941, 3_982_829_491, 1_146_958_065]);
        // ... r=random.Random(3771); print([r.randrange(1, 9) for _ in range(5)])
        let mut rng = PyRandom::new(3771);
        let draws: Vec<usize> = (0..5).map(|_| 1 + rng.below(8)).collect();
        assert_eq!(draws, [5, 2, 1, 6, 8]);
    }
}
