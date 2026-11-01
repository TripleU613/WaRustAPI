//! Native-library research tools for APK 2.26.37.71 (ports of the former
//! Python helpers). None of them loads WhatsApp code into this process:
//! `schema` and `kem-trace` read ELF bytes or `llvm-objdump` output, and
//! `qemu-check` runs the separate ARM64 decoder probe under QEMU in a
//! network-isolated bubblewrap sandbox. Evidence and scope:
//! docs/protocol/safety-number.md, native-kem.md and native-unpacking.md.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::ser::SerializeMap;
use sha2::{Digest, Sha256};

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

/// JSON object that keeps insertion order (serde_json's map would sort keys).
struct Ordered<T>(Vec<(String, T)>);

impl<T: Serialize> Serialize for Ordered<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

// ---------------------------------------------------------------- ELF64 -----

/// The few ELF64 little-endian ARM64 structures the schema reader needs:
/// file-backed PT_LOAD segments and R_AARCH64_RELATIVE addends from .rela.dyn.
struct Elf {
    segments: Vec<(u64, Vec<u8>)>,
    relative: BTreeMap<u64, u64>,
}

fn le_u16(d: &[u8], at: usize) -> Result<u16, String> {
    d.get(at..at + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| "truncated ELF".into())
}
fn le_u32(d: &[u8], at: usize) -> Result<u32, String> {
    d.get(at..at + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| "truncated ELF".into())
}
fn le_u64(d: &[u8], at: usize) -> Result<u64, String> {
    d.get(at..at + 8)
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| "truncated ELF".into())
}
fn usize_of(v: u64) -> Result<usize, String> {
    usize::try_from(v).map_err(|_| "ELF offset overflow".to_owned())
}

impl Elf {
    fn parse(d: &[u8]) -> Result<Self, String> {
        // e_ident: magic, ELFCLASS64 (2), ELFDATA2LSB (1); e_machine EM_AARCH64 (183).
        if d.get(..6) != Some(b"\x7fELF\x02\x01") || le_u16(d, 18)? != 183 {
            return Err("expected little-endian ELF64 ARM64".into());
        }
        let (phoff, phentsize, phnum) = (
            usize_of(le_u64(d, 32)?)?,
            le_u16(d, 54)? as usize,
            le_u16(d, 56)? as usize,
        );
        let mut segments = Vec::new();
        for i in 0..phnum {
            let at = phoff + i * phentsize;
            if le_u32(d, at)? == 1 {
                // PT_LOAD: file bytes [p_offset, p_offset + p_filesz) at p_vaddr.
                let (offset, vaddr, filesz) = (
                    usize_of(le_u64(d, at + 8)?)?,
                    le_u64(d, at + 16)?,
                    usize_of(le_u64(d, at + 32)?)?,
                );
                let bytes = d
                    .get(offset..offset + filesz)
                    .ok_or("PT_LOAD outside file")?;
                segments.push((vaddr, bytes.to_vec()));
            }
        }
        let (shoff, shentsize, shnum, shstrndx) = (
            usize_of(le_u64(d, 40)?)?,
            le_u16(d, 58)? as usize,
            le_u16(d, 60)? as usize,
            le_u16(d, 62)? as usize,
        );
        let section = |i: usize| -> Result<(u32, usize, usize), String> {
            let at = shoff + i * shentsize;
            Ok((
                le_u32(d, at)?,
                usize_of(le_u64(d, at + 24)?)?,
                usize_of(le_u64(d, at + 32)?)?,
            ))
        };
        let (_, strtab, _) = section(shstrndx)?;
        let mut relative = BTreeMap::new();
        let mut found = false;
        for i in 0..shnum {
            let (name, offset, size) = section(i)?;
            let name_at = strtab + name as usize;
            let end = d[name_at..]
                .iter()
                .position(|&b| b == 0)
                .ok_or("unterminated section name")?;
            if &d[name_at..name_at + end] == b".rela.dyn" {
                found = true;
                for entry in (offset..offset + size).step_by(24) {
                    // Elf64_Rela: r_offset, r_info (type in the low 32 bits), r_addend.
                    if le_u64(d, entry + 8)? & 0xffff_ffff == 1027 {
                        relative.insert(le_u64(d, entry)?, le_u64(d, entry + 16)?);
                    }
                }
            }
        }
        if !found {
            return Err("missing native relocation table".into());
        }
        Ok(Self { segments, relative })
    }

    fn data(&self, address: u64, length: usize) -> Result<&[u8], String> {
        for (start, contents) in &self.segments {
            if let Some(offset) = address
                .checked_sub(*start)
                .and_then(|o| usize::try_from(o).ok())
                && let Some(end) = offset
                    .checked_add(length)
                    .filter(|&end| end <= contents.len())
            {
                return Ok(&contents[offset..end]);
            }
        }
        Err(format!(
            "descriptor read outside file-backed ELF: {address:#x}"
        ))
    }

    fn u32(&self, address: u64) -> Result<u32, String> {
        le_u32(self.data(address, 4)?, 0)
    }

    fn pointer(&self, address: u64) -> Result<u64, String> {
        match self.relative.get(&address) {
            Some(&target) => Ok(target),
            None => le_u64(self.data(address, 8)?, 0),
        }
    }

    fn string(&self, address: u64) -> Result<String, String> {
        let mut out = Vec::new();
        for offset in 0..256 {
            match self.data(address + offset, 1)?[0] {
                0 => {
                    return String::from_utf8(out)
                        .map_err(|_| "non-ASCII descriptor string".into());
                }
                byte => out.push(byte),
            }
        }
        Err("unterminated descriptor string".into())
    }
}

// --------------------------------------------------------------- schema -----

const MERGED_SHA256: &str = "565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa";
// APK 2.26.37.71 libwhatsappmerged.so, initializer/unpack code at
// 0x2d3db0..0x2d3e54. ABI/source evidence: docs/protocol/safety-number.md.
const DESCRIPTORS: [u64; 2] = [0x740B60, 0x740C08];
// protobuf-c ProtobufCMessageDescriptor / ProtobufCEnumDescriptor magics.
const MESSAGE_MAGIC: u32 = 0x28AA_EEF9;
const ENUM_MAGIC: u32 = 0x1143_15AF;

#[derive(Serialize)]
struct Schema {
    apk_build: &'static str,
    library_sha256: &'static str,
    evidence: &'static str,
    messages: Vec<SchemaMessage>,
    enums: Ordered<SchemaEnum>,
}

#[derive(Serialize)]
struct SchemaMessage {
    name: String,
    address: String,
    size: u64,
    fields: Vec<SchemaField>,
}

#[derive(Serialize)]
struct SchemaField {
    name: String,
    tag: u32,
    label: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
    address: String,
    quantifier_offset: u32,
    offset: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    enumeration: Option<String>,
}

#[derive(Serialize)]
struct SchemaEnum {
    address: String,
    values: Vec<SchemaValue>,
}

#[derive(Serialize)]
struct SchemaValue {
    name: String,
    value: i32,
}

/// Read the two native fingerprint message descriptors and their enum
/// descriptors from the pinned reconstructed libwhatsappmerged.so.
pub fn schema(library: &Path) -> Result<(), String> {
    print!("{}", schema_json(library)?);
    Ok(())
}

fn schema_json(library: &Path) -> Result<String, String> {
    if !library.is_file() {
        return Err("missing reconstructed libwhatsappmerged.so".into());
    }
    if sha256_file(library)? != MERGED_SHA256 {
        return Err("requires the pinned 37.71 reconstructed libwhatsappmerged.so".into());
    }
    let elf = Elf::parse(&std::fs::read(library).map_err(|e| e.to_string())?)?;
    let mut messages = Vec::new();
    let mut enums = Vec::new();
    for address in DESCRIPTORS {
        if elf.u32(address)? != MESSAGE_MAGIC {
            return Err("wrong ProtobufCMessageDescriptor magic".into());
        }
        let (count, fields_at) = (elf.u32(address + 48)?, elf.pointer(address + 56)?);
        if !(1..=16).contains(&count) {
            return Err("unexpected fingerprint field count".into());
        }
        let size = elf.pointer(address + 40)?;
        let mut fields = Vec::new();
        let mut previous = 0;
        for index in 0..u64::from(count) {
            let at = fields_at + index * 72;
            let raw = elf.data(at + 8, 20)?;
            let word = |i: usize| le_u32(raw, i * 4);
            let (tag, label, kind, quantifier, offset) =
                (word(0)?, word(1)?, word(2)?, word(3)?, word(4)?);
            let kind_name = match kind {
                6 => "uint32",
                13 => "enum",
                15 => "bytes",
                16 => "message",
                _ => "",
            };
            if tag <= previous || label != 1 || kind_name.is_empty() {
                return Err("unsupported/unsorted fingerprint field descriptor".into());
            }
            previous = tag;
            if u64::from(offset) >= size || u64::from(quantifier) >= size {
                return Err("field offset exceeds native message size".into());
            }
            let mut field = SchemaField {
                name: elf.string(elf.pointer(at)?)?,
                tag,
                label: "optional",
                kind: kind_name,
                address: format!("{at:#x}"),
                quantifier_offset: quantifier,
                offset,
                message: None,
                enumeration: None,
            };
            let target = elf.pointer(at + 32)?;
            if kind == 16 {
                if !DESCRIPTORS.contains(&target) {
                    return Err("unexpected nested fingerprint descriptor".into());
                }
                field.message = Some(elf.string(elf.pointer(target + 8)?)?);
            } else if kind == 13 {
                if elf.u32(target)? != ENUM_MAGIC {
                    return Err("wrong ProtobufCEnumDescriptor magic".into());
                }
                let name = elf.string(elf.pointer(target + 8)?)?;
                let (enum_count, values_at) = (elf.u32(target + 40)?, elf.pointer(target + 48)?);
                if !(1..=16).contains(&enum_count) {
                    return Err("unexpected fingerprint enum count".into());
                }
                let values = (0..u64::from(enum_count))
                    .map(|i| {
                        let at = values_at + i * 24;
                        Ok(SchemaValue {
                            name: elf.string(elf.pointer(at)?)?,
                            value: le_u32(elf.data(at + 16, 4)?, 0)? as i32,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                // Python dict semantics: a repeated enum keeps its first position.
                let entry = SchemaEnum {
                    address: format!("{target:#x}"),
                    values,
                };
                match enums
                    .iter_mut()
                    .find(|(n, _): &&mut (String, SchemaEnum)| *n == name)
                {
                    Some(existing) => existing.1 = entry,
                    None => enums.push((name.clone(), entry)),
                }
                field.enumeration = Some(name);
            }
            fields.push(field);
        }
        messages.push(SchemaMessage {
            name: elf.string(elf.pointer(address + 8)?)?,
            address: format!("{address:#x}"),
            size,
            fields,
        });
    }
    let schema = Schema {
        apk_build: "2.26.37.71",
        library_sha256: MERGED_SHA256,
        evidence: "confirmed/static",
        messages,
        enums: Ordered(enums),
    };
    let mut json = serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())?;
    json.push('\n');
    Ok(json)
}

// ------------------------------------------------------------ kem-trace -----

const KEM_LIBRARIES: [(&str, &str); 2] = [
    (
        "libwhatsapp.so",
        "f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a",
    ),
    ("libwhatsappmerged.so", MERGED_SHA256),
];
const KEM_RANGES: [(&str, &str, u64, u64); 6] = [
    ("jni-encapsulate", "libwhatsapp.so", 0x622f70, 0x6232a0),
    ("jni-decapsulate", "libwhatsapp.so", 0x622cac, 0x622f0c),
    ("jni-dispatch", "libwhatsapp.so", 0x617050, 0x6170b0),
    ("ml-kem-exports", "libwhatsappmerged.so", 0x2baba0, 0x2bac08),
    (
        "ml-kem-dispatch",
        "libwhatsappmerged.so",
        0x70a6a8,
        0x70a6d0,
    ),
    (
        "ml-kem-encapsulate",
        "libwhatsappmerged.so",
        0x70a9dc,
        0x70aa78,
    ),
];

#[derive(Serialize)]
struct KemManifest {
    apk_build: &'static str,
    apk_sha256: &'static str,
    libraries: Ordered<&'static str>,
    ranges: Ordered<KemRange>,
    files: Ordered<KemFile>,
    coverage: &'static str,
}

#[derive(Serialize)]
struct KemRange {
    library: &'static str,
    start: String,
    end_exclusive: String,
}

#[derive(Serialize)]
struct KemFile {
    bytes: usize,
    sha256: String,
}

/// Disassemble the pinned KEM JNI/ML-KEM ranges with llvm-objdump and write
/// (or --check) the seven retained trace files. Nothing is executed.
pub fn kem_trace(libraries: &Path, output: &Path, check: bool) -> Result<(), String> {
    for (name, digest) in KEM_LIBRARIES {
        if sha256_file(&libraries.join(name))? != digest {
            return Err(format!("pinned library hash mismatch: {name}"));
        }
    }
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for (name, library, start, end) in KEM_RANGES {
        let out = Command::new("llvm-objdump")
            .args([
                "-d",
                &format!("--start-address={start}"),
                &format!("--stop-address={end}"),
                library,
            ])
            .current_dir(libraries)
            .output()
            .map_err(|e| format!("llvm-objdump: {e}"))?;
        if !out.status.success() {
            return Err(format!("llvm-objdump failed for {name}"));
        }
        files.push((format!("{name}.txt"), out.stdout));
    }
    let text = |name: &str| -> String {
        let bytes = &files.iter().find(|(n, _)| n == name).expect("collected").1;
        String::from_utf8_lossy(bytes).into_owned()
    };
    // Instruction anchors corroborate dispatch and widths, not primitive equivalence.
    let jni = text("jni-encapsulate.txt");
    for anchor in [
        "cmp\tw24, #0x320",
        "cmp\tw24, #0x4a0",
        "cmp\tw24, #0x620",
        "cmp\tw0, #0x20",
        "blr\tx28",
    ] {
        if !jni.contains(anchor) {
            return Err(format!("missing JNI anchor: {anchor}"));
        }
    }
    let dispatch = text("jni-dispatch.txt");
    for level in [512, 768, 1024] {
        if !dispatch.contains(&format!("OQS_KEM_ml_kem_{level}_encaps@plt")) {
            return Err(format!("missing ML-KEM dispatch: {level}"));
        }
    }
    let native = text("ml-kem-encapsulate.txt");
    if !native.contains("bl\t0x715a7c") || !native.contains("mov\tw1, #0x20") {
        return Err("missing native 32-byte randomness request".into());
    }
    let manifest = KemManifest {
        apk_build: "2.26.37.71",
        apk_sha256: "1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c",
        libraries: Ordered(
            KEM_LIBRARIES
                .iter()
                .map(|(n, d)| (n.to_string(), *d))
                .collect(),
        ),
        ranges: Ordered(
            KEM_RANGES
                .iter()
                .map(|(name, library, start, end)| {
                    (
                        name.to_string(),
                        KemRange {
                            library,
                            start: format!("{start:#x}"),
                            end_exclusive: format!("{end:#x}"),
                        },
                    )
                })
                .collect(),
        ),
        files: Ordered(
            files
                .iter()
                .map(|(name, data)| {
                    (
                        name.clone(),
                        KemFile {
                            bytes: data.len(),
                            sha256: hex::encode(Sha256::digest(data)),
                        },
                    )
                })
                .collect(),
        ),
        coverage: "Static instruction trace; no target execution or primitive equivalence test.",
    };
    let mut json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    json.push('\n');
    files.push(("manifest.json".into(), json.into_bytes()));
    if check {
        for (name, data) in &files {
            let path = output.join(name);
            if std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))? != *data {
                return Err(format!("trace differs: {name}"));
            }
        }
    } else {
        std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
        for (name, data) in &files {
            std::fs::write(output.join(name), data).map_err(|e| e.to_string())?;
        }
    }
    println!("Verified {} native KEM trace files", files.len());
    Ok(())
}

// ----------------------------------------------------------- qemu-check -----

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum QemuCheck {
    /// Load the decoder and call JNI_OnLoad (19 registered methods).
    DecoderInit,
    /// Run the ordinary full-archive extraction into an empty rootfs/decoded.
    DecoderFullArchive,
}

/// Run the local ARM64 decoder probe under QEMU in a network-isolated rootfs.
pub fn qemu_check(rootfs: &Path, check: QemuCheck, timeout_seconds: u64) -> Result<(), String> {
    if !(1..=600).contains(&timeout_seconds) {
        return Err("--timeout-seconds must be between 1 and 600".into());
    }
    let root = rootfs
        .canonicalize()
        .map_err(|e| format!("{}: {e}", rootfs.display()))?;
    let full = check == QemuCheck::DecoderFullArchive;
    let mut required = vec![
        "bin/decoder_probe",
        "system/bin/linker64",
        "system/lib64/libc.so",
        "system/lib64/libdl.so",
        "system/lib64/libsuperpack.so",
        "system/lib64/libandroid.so",
        "system/lib64/liblog.so",
    ];
    if full {
        required.push("system/lib64/wa_archive.so");
    }
    for name in &required {
        if !root.join(name).is_file() {
            return Err(format!(
                "missing decoder fixture: {}",
                root.join(name).display()
            ));
        }
    }
    // Exact inputs extracted from APK 2.26.37.71; do not silently reuse this
    // build's protocol/reconstruction conclusions for another decoder/archive.
    let mut pinned = vec![(
        "system/lib64/libsuperpack.so",
        "fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f",
    )];
    if full {
        pinned.push((
            "system/lib64/wa_archive.so",
            "24dfc7c0b9c3a13d1a9bd875304035a9f5ca6aa6f8ff904aef6ec1cfe3418c6b",
        ));
    }
    for (name, expected) in pinned {
        if sha256_file(&root.join(name))? != expected {
            return Err(format!("wrong 37.71 artifact hash: {name}"));
        }
    }
    let destination = root.join("decoded");
    if full
        && destination.exists()
        && (!destination.is_dir()
            || std::fs::read_dir(&destination)
                .map_err(|e| e.to_string())?
                .next()
                .is_some())
    {
        return Err("decoder-full-archive requires an empty rootfs/decoded directory".into());
    }
    let bwrap = which("bwrap").ok_or("requires bwrap and qemu-aarch64-static")?;
    let qemu = which("qemu-aarch64-static").ok_or("requires bwrap and qemu-aarch64-static")?;

    // QEMU's Linux ELF loader requires executable mode on the probe and
    // interpreter. Change only these local copies.
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
    for name in ["bin/decoder_probe", "system/bin/linker64"] {
        let path = root.join(name);
        let mode = path
            .metadata()
            .map_err(|e| e.to_string())?
            .permissions()
            .mode();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode | 0o100))
            .map_err(|e| e.to_string())?;
    }
    for name in ["proc", "dev"] {
        let dir = root.join(name);
        if !dir.exists() {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&dir)
                .map_err(|e| e.to_string())?;
        }
    }
    let placeholder = root.join("qemu");
    if !placeholder.exists() {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&placeholder)
            .map_err(|e| e.to_string())?;
    }
    let mut command = Command::new(bwrap);
    command
        .args([
            "--die-with-parent",
            "--unshare-net",
            "--unshare-pid",
            "--ro-bind",
        ])
        .arg(&root)
        .arg("/");
    if full {
        if !destination.exists() {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&destination)
                .map_err(|e| e.to_string())?;
        }
        command.arg("--bind").arg(&destination).arg("/decoded");
    }
    command.arg("--ro-bind").arg(&qemu).args([
        "/qemu",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--",
        "/qemu",
        "/bin/decoder_probe",
    ]);
    if full {
        command.arg("--full-archive");
    }
    let mut child = command
        .env_clear()
        .spawn()
        .map_err(|e| format!("bwrap: {e}"))?;
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if Instant::now() >= deadline {
            // --die-with-parent is not enough here (we stay alive), so kill it.
            let _ = child.kill();
            let _ = child.wait();
            return Err("QEMU check failed: wall-clock budget exhausted".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let code = status.code().unwrap_or(1);
    println!("QEMU process exit: {code}");
    if code == 0 {
        Ok(())
    } else {
        Err(format!("QEMU process exit: {code}"))
    }
}

fn which(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")?
        .to_str()?
        .split(':')
        .map(|dir| Path::new(dir).join(program))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROBE_INPUTS: [&str; 7] = [
        "bin/decoder_probe",
        "system/bin/linker64",
        "system/lib64/libc.so",
        "system/lib64/libdl.so",
        "system/lib64/libsuperpack.so",
        "system/lib64/libandroid.so",
        "system/lib64/liblog.so",
    ];

    #[test]
    fn qemu_missing_input_is_named() {
        let dir = tempfile::tempdir().unwrap();
        let error = qemu_check(dir.path(), QemuCheck::DecoderInit, 5).unwrap_err();
        assert!(
            error.contains("missing decoder fixture") && error.contains("bin/decoder_probe"),
            "{error}"
        );
    }

    #[test]
    fn qemu_wrong_build_refuses_before_preparation() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        for name in PROBE_INPUTS {
            let path = dir.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"unrecognized artifact").unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let error = qemu_check(dir.path(), QemuCheck::DecoderInit, 5).unwrap_err();
        assert!(error.contains("wrong 37.71 artifact hash"), "{error}");
        let mode = dir
            .path()
            .join("bin/decoder_probe")
            .metadata()
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "nothing is made executable before the hash check"
        );
        assert!(!dir.path().join("qemu").exists());
    }

    #[test]
    fn qemu_timeout_range_is_enforced() {
        let dir = tempfile::tempdir().unwrap();
        for seconds in [0, 601] {
            assert!(
                qemu_check(dir.path(), QemuCheck::DecoderInit, seconds)
                    .unwrap_err()
                    .contains("between 1 and 600")
            );
        }
    }

    #[test]
    fn schema_missing_and_wrong_build_refuse_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("libwhatsappmerged.so");
        assert!(
            schema_json(&path)
                .unwrap_err()
                .contains("missing reconstructed libwhatsappmerged.so")
        );
        std::fs::write(&path, b"\x7fELF not the pinned build").unwrap();
        assert!(
            schema_json(&path)
                .unwrap_err()
                .contains("requires the pinned 37.71")
        );
    }

    #[test]
    fn kem_trace_refuses_unpinned_libraries() {
        let dir = tempfile::tempdir().unwrap();
        for (name, _) in KEM_LIBRARIES {
            std::fs::write(dir.path().join(name), b"other build").unwrap();
        }
        let error = kem_trace(dir.path(), &dir.path().join("out"), true).unwrap_err();
        assert!(error.contains("pinned library hash mismatch"), "{error}");
    }

    /// Set WARUSTAPI_SCHEMA_LIBRARY to the reconstructed 37.71
    /// libwhatsappmerged.so to reproduce the committed schema exactly.
    #[test]
    fn pinned_schema_matches_committed_json() {
        let Some(library) = std::env::var_os("WARUSTAPI_SCHEMA_LIBRARY") else {
            eprintln!("skipped: WARUSTAPI_SCHEMA_LIBRARY not set");
            return;
        };
        let committed = include_str!("../../docs/protocol/fingerprint-native-schema.json");
        assert_eq!(schema_json(Path::new(&library)).unwrap(), committed);
    }
}
