//! dex class/method cross-reference and smali-level instruction dump.
//!
//! `apk-xref` answers "which class and method references this string constant?",
//! enabling targeted decompilation. `apk-smali` dumps the Dalvik instruction
//! stream for classes whose names match a filter, with string/type/method
//! references resolved symbolically. Both commands work directly from the dex
//! files inside the APK — no Java decompiler is needed.
//!
//! Structural tables parsed: string_id_item, type_id_item, method_id_item,
//! field_id_item, proto_id_item, class_def_item, class_data_item, code_item.
//! Instruction widths follow the Dalvik bytecode spec (AOSP docs/dalvik/README).

use crate::dex::{DEX_MAGIC, mutf8, uleb128};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

// ── low-level byte helpers ────────────────────────────────────────────────────

fn u16le(d: &[u8], o: usize) -> Result<u16, String> {
    d.get(o..o + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| format!("dex: u16 read at 0x{o:x} out of bounds"))
}

fn u32le(d: &[u8], o: usize) -> Result<u32, String> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| format!("dex: u32 read at 0x{o:x} out of bounds"))
}

// ── dex file view ─────────────────────────────────────────────────────────────

/// Lightweight view into one dex blob. Borrows the underlying bytes.
struct DexFile<'a> {
    d: &'a [u8],
    str_count: u32,
    str_off: u32,
    type_count: u32,
    type_off: u32,
    proto_count: u32,
    proto_off: u32,
    field_count: u32,
    field_off: u32,
    method_count: u32,
    method_off: u32,
    class_count: u32,
    class_off: u32,
}

impl<'a> DexFile<'a> {
    fn parse(d: &'a [u8]) -> Result<Self, String> {
        if d.len() < 0x70 || &d[..4] != DEX_MAGIC {
            return Err("not a dex file".into());
        }
        Ok(Self {
            d,
            str_count: u32le(d, 0x38)?,
            str_off: u32le(d, 0x3c)?,
            type_count: u32le(d, 0x40)?,
            type_off: u32le(d, 0x44)?,
            proto_count: u32le(d, 0x48)?,
            proto_off: u32le(d, 0x4c)?,
            field_count: u32le(d, 0x50)?,
            field_off: u32le(d, 0x54)?,
            method_count: u32le(d, 0x58)?,
            method_off: u32le(d, 0x5c)?,
            class_count: u32le(d, 0x60)?,
            class_off: u32le(d, 0x64)?,
        })
    }

    fn string(&self, idx: u32) -> Result<String, String> {
        if idx >= self.str_count {
            return Err(format!("string index {idx} >= {}", self.str_count));
        }
        let item_off = self.str_off as usize + 4 * idx as usize;
        let data_off = u32le(self.d, item_off)? as usize;
        let (len_units, body) = uleb128(self.d, data_off)?;
        let max_bytes = body + len_units as usize * 3 + 1;
        let end = self.d[body..max_bytes.min(self.d.len())]
            .iter()
            .position(|&b| b == 0)
            .map(|p| body + p)
            .unwrap_or(body);
        Ok(mutf8(&self.d[body..end]))
    }

    fn type_name(&self, type_idx: u32) -> Result<String, String> {
        if type_idx == 0xFFFF_FFFF {
            return Ok("<none>".into());
        }
        if type_idx >= self.type_count {
            return Err(format!("type index {type_idx} >= {}", self.type_count));
        }
        let str_idx = u32le(self.d, self.type_off as usize + 4 * type_idx as usize)?;
        self.string(str_idx)
    }

    /// `(class_descriptor, method_name, shorty)` for a method_id_item.
    fn method_info(&self, method_idx: u32) -> Result<(String, String, String), String> {
        if method_idx >= self.method_count {
            return Err(format!(
                "method index {method_idx} >= {}",
                self.method_count
            ));
        }
        let off = self.method_off as usize + 8 * method_idx as usize;
        let class_idx = u16le(self.d, off)? as u32;
        let proto_idx = u16le(self.d, off + 2)? as u32;
        let name_idx = u32le(self.d, off + 4)?;
        let class = self.type_name(class_idx)?;
        let name = self.string(name_idx)?;
        let shorty = self.proto_shorty(proto_idx).unwrap_or_default();
        Ok((class, name, shorty))
    }

    fn proto_shorty(&self, proto_idx: u32) -> Result<String, String> {
        if proto_idx >= self.proto_count {
            return Err(format!("proto index {proto_idx} >= {}", self.proto_count));
        }
        let off = self.proto_off as usize + 12 * proto_idx as usize;
        let shorty_idx = u32le(self.d, off)?;
        self.string(shorty_idx)
    }

    /// `(class_descriptor, field_name, type_descriptor)` for a field_id_item.
    fn field_info(&self, field_idx: u32) -> Result<(String, String, String), String> {
        if field_idx >= self.field_count {
            return Err(format!("field index {field_idx} >= {}", self.field_count));
        }
        let off = self.field_off as usize + 8 * field_idx as usize;
        let class_idx = u16le(self.d, off)? as u32;
        let type_idx = u16le(self.d, off + 2)? as u32;
        let name_idx = u32le(self.d, off + 4)?;
        let class = self.type_name(class_idx)?;
        let name = self.string(name_idx)?;
        let ty = self.type_name(type_idx)?;
        Ok((class, name, ty))
    }
}

// ── Dalvik instruction width table ────────────────────────────────────────────

/// Width in 16-bit code units for the instruction whose opcode is `op`.
/// Follows AOSP `docs/dalvik/dalvik-bytecode.html` table; unknown opcodes
/// return 1 (conservative — may misalign, won't panic).
fn insn_width(op: u8) -> usize {
    match op {
        // 1-unit instructions (10x, 11x, 11n, 12x, 10t formats)
        0x00         // nop
        | 0x01       // move
        | 0x04       // move-wide
        | 0x07       // move-object
        | 0x0a..=0x12// move-result, move-exception, return-*, const/4
        | 0x1d..=0x1e// monitor-enter/exit
        | 0x21       // array-length
        | 0x27       // throw
        | 0x28       // goto
        | 0x3e..=0x43// unused block
        | 0x73       // unused / return-void-no-barrier
        | 0x79..=0x7a// unused
        | 0x7b..=0x8f// neg/not/int-to-* conversions (12x)
        | 0xb0..=0xcf// binary-2addr ops (12x)
        | 0xe3..=0xf9// unused block
        => 1,

        // 2-unit instructions (21x, 22x, 21c, 22c, 21t, 21s, 21h, 23x, 22t, 22s, 22b)
        0x02         // move/from16
        | 0x05       // move-wide/from16
        | 0x08       // move-object/from16
        | 0x13       // const/16
        | 0x15       // const/high16
        | 0x16       // const-wide/16
        | 0x19       // const-wide/high16
        | 0x1a       // const-string  ← key
        | 0x1c       // const-class
        | 0x1f       // check-cast
        | 0x20       // instance-of
        | 0x22       // new-instance
        | 0x23       // new-array
        | 0x29       // goto/16
        | 0x2d..=0x37// cmp*, if-test*
        | 0x38..=0x3d// if-testz*
        | 0x44..=0x51// aget/aput (23x)
        | 0x52..=0x5f// iget/iput (22c)
        | 0x60..=0x6d// sget/sput (21c)
        | 0x90..=0xaf// binary ops (23x)
        | 0xd0..=0xe2// lit16/lit8 binary ops
        | 0xfe..=0xff// const-method-handle/type (21c)
        => 2,

        // 3-unit instructions (31i, 31c, 31t, 32x, 35c, 3rc)
        0x03         // move/16
        | 0x06       // move-wide/16
        | 0x09       // move-object/16
        | 0x14       // const (31i)
        | 0x17       // const-wide/32
        | 0x1b       // const-string/jumbo ← key
        | 0x26       // fill-array-data (31t)
        | 0x2a       // goto/32
        | 0x2b..=0x2c// packed/sparse-switch
        | 0x6e..=0x72// invoke-* (35c) ← key
        | 0x74..=0x78// invoke-*/range (3rc)
        | 0xfc       // invoke-custom
        | 0xfd       // invoke-custom/range
        => 3,

        // 4-unit instructions (45cc, 4rcc)
        0xfa..=0xfb => 4,

        // 5-unit: const-wide (51l)
        0x18 => 5,

        // Anything not listed: conservatively advance 1 unit.
        _ => 1,
    }
}

// ── instruction decoding ──────────────────────────────────────────────────────

/// Human-readable name for common opcodes.
fn opcode_name(op: u8) -> &'static str {
    match op {
        0x00 => "nop",
        0x01 => "move",
        0x02 => "move/from16",
        0x03 => "move/16",
        0x04 => "move-wide",
        0x05 => "move-wide/from16",
        0x06 => "move-wide/16",
        0x07 => "move-object",
        0x08 => "move-object/from16",
        0x09 => "move-object/16",
        0x0a => "move-result",
        0x0b => "move-result-wide",
        0x0c => "move-result-object",
        0x0d => "move-exception",
        0x0e => "return-void",
        0x0f => "return",
        0x10 => "return-wide",
        0x11 => "return-object",
        0x12 => "const/4",
        0x13 => "const/16",
        0x14 => "const",
        0x15 => "const/high16",
        0x16 => "const-wide/16",
        0x17 => "const-wide/32",
        0x18 => "const-wide",
        0x19 => "const-wide/high16",
        0x1a => "const-string",
        0x1b => "const-string/jumbo",
        0x1c => "const-class",
        0x1d => "monitor-enter",
        0x1e => "monitor-exit",
        0x1f => "check-cast",
        0x20 => "instance-of",
        0x21 => "array-length",
        0x22 => "new-instance",
        0x23 => "new-array",
        0x24 => "filled-new-array",
        0x25 => "filled-new-array/range",
        0x26 => "fill-array-data",
        0x27 => "throw",
        0x28 => "goto",
        0x29 => "goto/16",
        0x2a => "goto/32",
        0x2b => "packed-switch",
        0x2c => "sparse-switch",
        0x2d => "cmpl-float",
        0x2e => "cmpg-float",
        0x2f => "cmpl-double",
        0x30 => "cmpg-double",
        0x31 => "cmp-long",
        0x32 => "if-eq",
        0x33 => "if-ne",
        0x34 => "if-lt",
        0x35 => "if-ge",
        0x36 => "if-gt",
        0x37 => "if-le",
        0x38 => "if-eqz",
        0x39 => "if-nez",
        0x3a => "if-ltz",
        0x3b => "if-gez",
        0x3c => "if-gtz",
        0x3d => "if-lez",
        0x44 => "aget",
        0x45 => "aget-wide",
        0x46 => "aget-object",
        0x4b => "aput",
        0x4d => "aput-object",
        0x52 => "iget",
        0x54 => "iget-object",
        0x59 => "iput",
        0x5b => "iput-object",
        0x60 => "sget",
        0x62 => "sget-object",
        0x67 => "sput",
        0x69 => "sput-object",
        0x6e => "invoke-virtual",
        0x6f => "invoke-super",
        0x70 => "invoke-direct",
        0x71 => "invoke-static",
        0x72 => "invoke-interface",
        0x74 => "invoke-virtual/range",
        0x75 => "invoke-super/range",
        0x76 => "invoke-direct/range",
        0x77 => "invoke-static/range",
        0x78 => "invoke-interface/range",
        0x90 => "add-int",
        0x91 => "sub-int",
        0x92 => "mul-int",
        0x93 => "div-int",
        0x94 => "rem-int",
        0x95 => "and-int",
        0x96 => "or-int",
        0x97 => "xor-int",
        0xb0 => "add-int/2addr",
        0xb1 => "sub-int/2addr",
        0xb4 => "rem-int/2addr",
        0xb5 => "and-int/2addr",
        0xb6 => "or-int/2addr",
        0xb7 => "xor-int/2addr",
        0xd8 => "add-int/lit8",
        0xd9 => "rsub-int/lit8",
        0xda => "mul-int/lit8",
        0xfe => "const-method-handle",
        0xff => "const-method-type",
        _ => "??",
    }
}

/// Decoded form of a single Dalvik instruction.
#[derive(Debug)]
pub struct Insn {
    /// Byte offset from the start of the code unit array.
    pub byte_off: usize,
    /// Opcode byte.
    pub op: u8,
    /// Width in 16-bit code units (byte width = width * 2).
    pub width: usize,
    /// Decoded annotation (empty for instructions without interesting operands).
    pub annotation: String,
}

/// Decode all instructions in `insns` (a slice of raw code-unit bytes,
/// length = insns_size * 2). References are resolved via `dex`.
fn decode_insns(insns: &[u8], dex: &DexFile<'_>) -> Vec<Insn> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos + 1 < insns.len() {
        let op = insns[pos];
        let reg = insns[pos + 1]; // high byte of first code unit (register / other)
        let width = insn_width(op);
        let byte_width = width * 2;

        let annotation = if pos + byte_width > insns.len() {
            // truncated — stop
            out.push(Insn {
                byte_off: pos,
                op,
                width,
                annotation: "# truncated".into(),
            });
            break;
        } else {
            decode_annotation(op, reg, &insns[pos..pos + byte_width], dex)
        };

        out.push(Insn {
            byte_off: pos,
            op,
            width,
            annotation,
        });
        pos += byte_width;
    }
    out
}

fn decode_annotation(op: u8, reg: u8, insn_bytes: &[u8], dex: &DexFile<'_>) -> String {
    match op {
        // const-string vAA, string@BBBB
        0x1a if insn_bytes.len() >= 4 => {
            let str_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let s = dex
                .string(str_idx)
                .unwrap_or_else(|_| format!("?{str_idx}"));
            format!("v{reg}, {:?}", s)
        }
        // const-string/jumbo vAA, string@BBBBBBBB
        0x1b if insn_bytes.len() >= 6 => {
            let str_idx =
                u32::from_le_bytes([insn_bytes[2], insn_bytes[3], insn_bytes[4], insn_bytes[5]]);
            let s = dex
                .string(str_idx)
                .unwrap_or_else(|_| format!("?{str_idx}"));
            format!("v{reg}, {:?}", s)
        }
        // const-class vAA, type@BBBB
        // new-instance vAA, type@BBBB
        // check-cast vAA, type@BBBB
        0x1c | 0x1f | 0x22 if insn_bytes.len() >= 4 => {
            let type_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let t = dex
                .type_name(type_idx)
                .unwrap_or_else(|_| format!("?{type_idx}"));
            format!("v{reg}, {t}")
        }
        // invoke-virtual/super/direct/static/interface {vC,..vG}, meth@BBBB
        // Format 35c: [A|G|op] [BBBB] [F|E|D|C]
        0x6e..=0x72 if insn_bytes.len() >= 6 => {
            let meth_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let arg_count = (reg >> 4) & 0xf;
            let g_reg = reg & 0xf;
            let d4 = insn_bytes[4];
            let d5 = insn_bytes[5];
            let c = d4 & 0xf;
            let d = (d4 >> 4) & 0xf;
            let e = d5 & 0xf;
            let f = (d5 >> 4) & 0xf;
            let args: Vec<String> = (0..arg_count as usize)
                .map(|i| format!("v{}", [c, d, e, f, g_reg][i]))
                .collect();
            let meth = dex
                .method_info(meth_idx)
                .map(|(cls, name, shorty)| format!("{cls}->{name}({shorty})"))
                .unwrap_or_else(|_| format!("?{meth_idx}"));
            format!("{{{}}}, {meth}", args.join(", "))
        }
        // invoke-*/range {vCCCC..vNNNN}, meth@BBBB
        // Format 3rc: [AA|op] [BBBB] [CCCC]
        0x74..=0x78 if insn_bytes.len() >= 6 => {
            let meth_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let first = u16::from_le_bytes([insn_bytes[4], insn_bytes[5]]);
            let meth = dex
                .method_info(meth_idx)
                .map(|(cls, name, shorty)| format!("{cls}->{name}({shorty})"))
                .unwrap_or_else(|_| format!("?{meth_idx}"));
            format!("{{v{first}..v{}}}, {meth}", first + reg as u16)
        }
        // iget/iput variants: field@CCCC, format 22c: [B|A|op] [CCCC]
        0x52..=0x5f if insn_bytes.len() >= 4 => {
            let field_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let dest = reg & 0xf;
            let src = (reg >> 4) & 0xf;
            let field = dex
                .field_info(field_idx)
                .map(|(cls, name, ty)| format!("{cls}->{name}:{ty}"))
                .unwrap_or_else(|_| format!("?{field_idx}"));
            format!("v{dest}, v{src}, {field}")
        }
        // sget/sput variants: field@BBBB, format 21c
        0x60..=0x6d if insn_bytes.len() >= 4 => {
            let field_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let field = dex
                .field_info(field_idx)
                .map(|(cls, name, ty)| format!("{cls}->{name}:{ty}"))
                .unwrap_or_else(|_| format!("?{field_idx}"));
            format!("v{reg}, {field}")
        }
        // const/4 vA, #+B — 11n: high nibble of reg byte = register, low nibble of that = value
        0x12 => {
            let v = (reg >> 4) & 0xf;
            let lit = (reg & 0xf) as i8;
            format!("v{v}, #{lit:#x}")
        }
        // const/16 vAA, #+BBBB — 21s
        0x13 if insn_bytes.len() >= 4 => {
            let lit = i16::from_le_bytes([insn_bytes[2], insn_bytes[3]]);
            format!("v{reg}, #{lit:#x}")
        }
        // const vAA, #+BBBBBBBB — 31i
        0x14 if insn_bytes.len() >= 6 => {
            let lit =
                i32::from_le_bytes([insn_bytes[2], insn_bytes[3], insn_bytes[4], insn_bytes[5]]);
            format!("v{reg}, #{lit:#x}")
        }
        // return / goto / comparisons — just show register
        0x0f | 0x10 | 0x11 | 0x27 => format!("v{reg}"),
        0x28 => {
            let offset = insn_bytes[1] as i8;
            format!("+{offset:#x}")
        }
        0x29 if insn_bytes.len() >= 4 => {
            let offset = i16::from_le_bytes([insn_bytes[2], insn_bytes[3]]);
            format!("+{offset:#x}")
        }
        // if-test vA, vB, +CCCC — 22t
        0x32..=0x37 if insn_bytes.len() >= 4 => {
            let va = reg & 0xf;
            let vb = (reg >> 4) & 0xf;
            let offset = i16::from_le_bytes([insn_bytes[2], insn_bytes[3]]);
            format!("v{va}, v{vb}, +{offset:#x}")
        }
        // if-testz vAA, +BBBB — 21t
        0x38..=0x3d if insn_bytes.len() >= 4 => {
            let offset = i16::from_le_bytes([insn_bytes[2], insn_bytes[3]]);
            format!("v{reg}, +{offset:#x}")
        }
        // new-array vA, vB, type@CCCC — 22c
        0x23 if insn_bytes.len() >= 4 => {
            let type_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let va = reg & 0xf;
            let vb = (reg >> 4) & 0xf;
            let t = dex
                .type_name(type_idx)
                .unwrap_or_else(|_| format!("?{type_idx}"));
            format!("v{va}, v{vb}, {t}")
        }
        // instance-of vA, vB, type@CCCC — 22c
        0x20 if insn_bytes.len() >= 4 => {
            let type_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let va = reg & 0xf;
            let vb = (reg >> 4) & 0xf;
            let t = dex
                .type_name(type_idx)
                .unwrap_or_else(|_| format!("?{type_idx}"));
            format!("v{va}, v{vb}, {t}")
        }
        _ => String::new(),
    }
}

// ── cross-reference result ────────────────────────────────────────────────────

/// One occurrence of an anchor string in the APK bytecode.
#[derive(Debug)]
pub struct XRef {
    /// Dex file where this was found (e.g. `classes2.dex`).
    pub dex_file: String,
    /// Dalvik class descriptor of the enclosing class (e.g. `Lcom/whatsapp/Foo;`).
    pub class_name: String,
    /// Name of the enclosing method.
    pub method_name: String,
    /// Proto shorty string of the method.
    pub method_proto: String,
    /// Byte offset of the const-string instruction from the start of the dex blob.
    pub dex_byte_off: usize,
}

// ── per-dex xref scan ─────────────────────────────────────────────────────────

fn xref_one(
    dex_name: &str,
    d: &[u8],
    anchors: &[String],
    out: &mut BTreeMap<String, Vec<XRef>>,
) -> Result<(), String> {
    let dex = DexFile::parse(d)?;

    // Build anchor-string → index set for fast lookups.
    let mut anchor_indices: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for idx in 0..dex.str_count {
        let s = dex.string(idx).unwrap_or_default();
        if anchors.contains(&s) {
            anchor_indices.insert(idx);
        }
    }
    if anchor_indices.is_empty() {
        return Ok(());
    }

    // Walk class_def_item list.
    for ci in 0..dex.class_count {
        let cdef_off = dex.class_off as usize + 32 * ci as usize;
        let class_idx = match u32le(d, cdef_off) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let class_data_off = match u32le(d, cdef_off + 24) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if class_data_off == 0 {
            continue;
        }
        let class_name = dex.type_name(class_idx).unwrap_or_default();

        // Parse class_data_item.
        let mut pos = class_data_off as usize;
        let (sf, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;
        let (inf, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;
        let (dm, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;
        let (vm, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;
        let field_count = sf + inf;
        let method_count = dm + vm;

        // Skip encoded_field entries: each is two ULEBs.
        for _ in 0..field_count {
            let (_, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            let (_, p2) = uleb128(d, p).unwrap_or((0, p + 1));
            pos = p2;
        }

        // Walk encoded_method entries.
        let mut method_idx_acc: u32 = 0;
        for _ in 0..method_count {
            let (moff, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            pos = p;
            let (_, p) = uleb128(d, pos).unwrap_or((0, pos + 1)); // access_flags
            pos = p;
            let (code_off, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            pos = p;

            method_idx_acc += moff;
            if code_off == 0 {
                continue;
            }

            let (method_class, method_name, method_proto) =
                dex.method_info(method_idx_acc).unwrap_or_default();
            let _ = method_class; // same as class_name

            // Parse code_item header (16 bytes).
            let ci_off = code_off as usize;
            let insns_size = match u32le(d, ci_off + 12) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let insns_start = ci_off + 16;
            let insns_end = insns_start + insns_size as usize * 2;
            let insns = match d.get(insns_start..insns_end) {
                Some(s) => s,
                None => continue,
            };

            // Scan code units for const-string patterns.
            let mut i = 0usize;
            while i + 1 < insns.len() {
                let op = insns[i];
                let width = insn_width(op);
                let byte_width = width * 2;

                let str_idx: Option<u32> = match op {
                    0x1a if i + 3 < insns.len() => {
                        Some(u16::from_le_bytes([insns[i + 2], insns[i + 3]]) as u32)
                    }
                    0x1b if i + 5 < insns.len() => Some(u32::from_le_bytes([
                        insns[i + 2],
                        insns[i + 3],
                        insns[i + 4],
                        insns[i + 5],
                    ])),
                    _ => None,
                };

                if let Some(sidx) = str_idx
                    && anchor_indices.contains(&sidx)
                {
                    let anchor = dex.string(sidx).unwrap_or_default();
                    out.entry(anchor).or_default().push(XRef {
                        dex_file: dex_name.to_owned(),
                        class_name: class_name.clone(),
                        method_name: method_name.clone(),
                        method_proto: method_proto.clone(),
                        dex_byte_off: insns_start + i,
                    });
                }

                i += byte_width.max(2); // always advance at least one code unit
            }
        }
    }
    Ok(())
}

// ── public apk-xref entry point ───────────────────────────────────────────────

/// For each anchor in `anchors`, find every class+method that loads it via
/// const-string. Returns a map anchor → list of XRef records.
pub fn apk_xref(apk: &Path, anchors: &[String]) -> Result<BTreeMap<String, Vec<XRef>>, String> {
    let f = std::fs::File::open(apk).map_err(|e| format!("{}: {e}", apk.display()))?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| format!("zip: {e}"))?;
    let names: Vec<String> = zip
        .file_names()
        .filter(|n| n.starts_with("classes") && n.ends_with(".dex"))
        .map(str::to_owned)
        .collect();

    let mut out: BTreeMap<String, Vec<XRef>> = BTreeMap::new();
    for name in &names {
        let mut buf = Vec::new();
        zip.by_name(name)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        if let Err(e) = xref_one(name, &buf, anchors, &mut out) {
            eprintln!("warning: {name}: {e}");
        }
    }
    Ok(out)
}

pub fn run_xref(apk: &Path, anchors: &[String]) -> Result<(), String> {
    let results = apk_xref(apk, anchors)?;
    let total: usize = results.values().map(|v| v.len()).sum();
    if results.is_empty() {
        eprintln!("no references found for any anchor");
        return Ok(());
    }
    for (anchor, refs) in &results {
        println!("[{anchor:?}]  ({} reference(s))", refs.len());
        for r in refs {
            println!(
                "  {}  {}->{}({})",
                r.dex_file, r.class_name, r.method_name, r.method_proto
            );
            println!("    dex byte offset: 0x{:x}", r.dex_byte_off);
        }
    }
    eprintln!(
        "{total} total reference(s) across {} anchor(s)",
        results.len()
    );
    Ok(())
}

// ── smali dump ────────────────────────────────────────────────────────────────

/// One decoded method from a single class.
pub struct MethodDump {
    pub class_name: String,
    pub method_name: String,
    pub method_proto: String,
    pub access_flags: u32,
    pub registers: u16,
    pub insns: Vec<Insn>,
}

fn smali_one(
    d: &[u8],
    class_filter: &str,
    method_filter: Option<&str>,
    out: &mut Vec<MethodDump>,
) -> Result<(), String> {
    let dex = DexFile::parse(d)?;

    for ci in 0..dex.class_count {
        let cdef_off = dex.class_off as usize + 32 * ci as usize;
        let class_idx = u32le(d, cdef_off)?;
        let class_name = dex.type_name(class_idx).unwrap_or_default();

        if !class_name.contains(class_filter) {
            continue;
        }

        let class_data_off = u32le(d, cdef_off + 24)?;
        if class_data_off == 0 {
            continue;
        }

        let mut pos = class_data_off as usize;
        let (sf, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;
        let (inf, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;
        let (dm, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;
        let (vm, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
        pos = p;

        for _ in 0..(sf + inf) {
            let (_, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            let (_, p2) = uleb128(d, p).unwrap_or((0, p + 1));
            pos = p2;
        }

        let mut method_idx_acc: u32 = 0;
        for _ in 0..(dm + vm) {
            let (moff, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            pos = p;
            let (access_flags, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            pos = p;
            let (code_off, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            pos = p;
            method_idx_acc += moff;

            let (_, method_name, method_proto) =
                dex.method_info(method_idx_acc).unwrap_or_default();

            if let Some(mf) = method_filter
                && !method_name.contains(mf)
            {
                continue;
            }

            if code_off == 0 {
                out.push(MethodDump {
                    class_name: class_name.clone(),
                    method_name,
                    method_proto,
                    access_flags,
                    registers: 0,
                    insns: vec![],
                });
                continue;
            }

            let ci_off = code_off as usize;
            let registers = u16le(d, ci_off).unwrap_or(0);
            let insns_size = u32le(d, ci_off + 12).unwrap_or(0);
            let insns_start = ci_off + 16;
            let insns_end = insns_start + insns_size as usize * 2;
            let raw = d.get(insns_start..insns_end).unwrap_or(&[]);
            let insns = decode_insns(raw, &dex);

            out.push(MethodDump {
                class_name: class_name.clone(),
                method_name,
                method_proto,
                access_flags,
                registers,
                insns,
            });
        }
    }
    Ok(())
}

pub fn apk_smali(
    apk: &Path,
    class_filter: &str,
    method_filter: Option<&str>,
) -> Result<Vec<MethodDump>, String> {
    let f = std::fs::File::open(apk).map_err(|e| format!("{}: {e}", apk.display()))?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| format!("zip: {e}"))?;
    let names: Vec<String> = zip
        .file_names()
        .filter(|n| n.starts_with("classes") && n.ends_with(".dex"))
        .map(str::to_owned)
        .collect();

    let mut out = Vec::new();
    for name in &names {
        let mut buf = Vec::new();
        zip.by_name(name)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        if let Err(e) = smali_one(&buf, class_filter, method_filter, &mut out) {
            eprintln!("warning: {name}: {e}");
        }
    }
    Ok(out)
}

pub fn run_smali(
    apk: &Path,
    class_filter: &str,
    method_filter: Option<&str>,
) -> Result<(), String> {
    let methods = apk_smali(apk, class_filter, method_filter)?;
    if methods.is_empty() {
        eprintln!("no classes match filter {:?}", class_filter);
        return Ok(());
    }
    let mut last_class = String::new();
    for m in &methods {
        if m.class_name != last_class {
            println!("\n# class: {}", m.class_name);
            last_class = m.class_name.clone();
        }
        let flags = access_flags_str(m.access_flags);
        println!(
            "\n.method {flags}{}({})  # .registers {}",
            m.method_name, m.method_proto, m.registers
        );
        if m.insns.is_empty() {
            println!("    # (abstract / native)");
        }
        for insn in &m.insns {
            let name = opcode_name(insn.op);
            let end = insn.byte_off + insn.width * 2;
            if insn.annotation.is_empty() {
                println!("    {:04x}..{:04x}  {name}", insn.byte_off, end);
            } else {
                println!(
                    "    {:04x}..{:04x}  {name}  {}",
                    insn.byte_off, end, insn.annotation
                );
            }
        }
        println!(".end method");
    }
    eprintln!("{} method(s) in {} class(es)", methods.len(), {
        let mut seen = std::collections::HashSet::new();
        methods.iter().for_each(|m| {
            seen.insert(&m.class_name);
        });
        seen.len()
    });
    Ok(())
}

fn access_flags_str(flags: u32) -> &'static str {
    match flags & 0x7 {
        0x1 => "public ",
        0x2 => "private ",
        0x4 => "protected ",
        _ => "",
    }
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid dex header scaffold for tests.
    /// Supplies only the header; content tables are empty.
    fn minimal_dex_header() -> Vec<u8> {
        let mut d = vec![0u8; 0x70];
        d[0..4].copy_from_slice(b"dex\n");
        d[4..8].copy_from_slice(b"035\0");
        // endian tag
        d[0x3c..0x40].copy_from_slice(&0x12345678u32.to_le_bytes());
        // header_size = 0x70
        d[0x08..0x0c].copy_from_slice(&0x70u32.to_le_bytes());
        // All counts and offsets remain zero (empty tables).
        d
    }

    #[test]
    fn insn_width_covers_all_opcodes() {
        // Every opcode must return width >= 1 and <= 5.
        for op in 0u8..=255 {
            let w = insn_width(op);
            assert!(
                w >= 1 && w <= 5,
                "opcode 0x{op:02x} width {w} out of expected range"
            );
        }
    }

    #[test]
    fn insn_width_key_opcodes() {
        assert_eq!(insn_width(0x1a), 2, "const-string");
        assert_eq!(insn_width(0x1b), 3, "const-string/jumbo");
        assert_eq!(insn_width(0x6e), 3, "invoke-virtual");
        assert_eq!(insn_width(0x71), 3, "invoke-static");
        assert_eq!(insn_width(0x22), 2, "new-instance");
        assert_eq!(insn_width(0x18), 5, "const-wide");
        assert_eq!(insn_width(0xfa), 4, "invoke-polymorphic");
        assert_eq!(insn_width(0x0e), 1, "return-void");
    }

    #[test]
    fn decode_insns_const_string_annotation() {
        // Craft a minimal dex with a single string "hello" and one const-string insn.
        // String table: 1 string, at offset 0x80 (just after header).
        let mut d = minimal_dex_header();
        // str_count=1 at 0x38, str_off=0x78 (right after header)
        let str_ids_off = 0x70usize;
        let str_data_off = 0x74usize;
        d.resize(0x90, 0);

        // string_ids_size=1, string_ids_off=str_ids_off
        d[0x38..0x3c].copy_from_slice(&1u32.to_le_bytes());
        d[0x3c..0x40].copy_from_slice(&(str_ids_off as u32).to_le_bytes());
        // string_id_item[0] → str_data_off
        d[str_ids_off..str_ids_off + 4].copy_from_slice(&(str_data_off as u32).to_le_bytes());
        // string_data_item: uleb128 len=5, then "hello\0"
        d[str_data_off] = 5; // uleb128 length
        d[str_data_off + 1..str_data_off + 7].copy_from_slice(b"hello\0");

        let dex = DexFile::parse(&d).unwrap();

        // Build a const-string instruction for string 0 into register 1.
        // Bytes: [0x1a, 0x01, 0x00, 0x00]
        let insns = &[0x1au8, 0x01, 0x00, 0x00];
        let decoded = decode_insns(insns, &dex);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].op, 0x1a);
        assert!(
            decoded[0].annotation.contains("hello"),
            "annotation = {:?}",
            decoded[0].annotation
        );
    }

    #[test]
    fn decode_insns_never_panics_on_truncated() {
        let d = minimal_dex_header();
        let dex = DexFile::parse(&d).unwrap();
        // Truncated code: just one byte (incomplete instruction)
        let insns = &[0x1au8];
        let decoded = decode_insns(insns, &dex);
        // Should produce at most one entry with a truncation note.
        assert!(decoded.len() <= 1);
    }

    #[test]
    fn xref_scan_finds_string_in_crafted_dex() {
        // Build a minimal dex that has:
        //   1 string ("v.whatsapp.net"),
        //   1 type (descriptor = string 0),
        //   1 method in a class that loads string 0 via const-string.
        // This is the minimal plumbing needed to exercise xref_one end-to-end.

        // Layout (all offsets relative to start of blob):
        //   0x000..0x070: dex header
        //   0x070..0x074: string_id_item[0]  → 0x0a0
        //   0x074..0x078: type_id_item[0] (str_idx=0)
        //   0x078..0x07a: proto_id_item[0].shorty_idx=0, return_type=0, params=0
        //   0x07e..0x086: method_id_item[0] (class=0, proto=0, name=0)
        //   0x086..0x0a0: class_def_item[0] (32 bytes)
        //       class_idx=0, access=1, super=0xffffffff, interfaces=0
        //       source=0xffffffff, annotations=0, class_data_off=0x0c0, static_values=0
        //   0x0a0..0x0b0: string_data_item[0]: len=14, "v.whatsapp.net\0"
        //   0x0b0: proto_id_item actual (shorty=0, return=0, params_off=0)
        //   0x0c0: class_data_item
        //       sf=0, inf=0, dm=1, vm=0
        //       encoded_method[0]: method_idx_diff=0, access=0x1, code_off=0x0e0
        //   0x0e0: code_item
        //       registers=2, ins=0, outs=1, tries=0
        //       debug_info=0, insns_size=3
        //       insns: [const-string v0 "v.whatsapp.net", return-void]
        //          0x1a 0x00 0x00 0x00    const-string v0, #0
        //          0x0e 0x00              return-void
        //          0x00 0x00              (padding to align to 4 insns_size)

        let mut d = vec![0u8; 0x100];
        // magic
        d[0..4].copy_from_slice(b"dex\n");
        d[4..8].copy_from_slice(b"035\0");
        // header_size
        d[8..12].copy_from_slice(&0x70u32.to_le_bytes());
        // endian_tag
        d[12..16].copy_from_slice(&0x12345678u32.to_le_bytes());

        let str_ids_off = 0x070usize;
        let type_ids_off = 0x074usize;
        let proto_ids_off = 0x078usize;
        let method_ids_off = 0x084usize; // 0x078 + 12 = 0x084
        let class_defs_off = 0x08cusize; // 0x084 + 8 = 0x08c
        let str_data_off = 0x0b0usize;
        let class_data_off = 0x0c0usize;
        let code_item_off = 0x0e0usize;

        // string_ids: count=1, off=str_ids_off
        d[0x38..0x3c].copy_from_slice(&1u32.to_le_bytes());
        d[0x3c..0x40].copy_from_slice(&(str_ids_off as u32).to_le_bytes());
        // type_ids: count=1, off=type_ids_off
        d[0x40..0x44].copy_from_slice(&1u32.to_le_bytes());
        d[0x44..0x48].copy_from_slice(&(type_ids_off as u32).to_le_bytes());
        // proto_ids: count=1, off=proto_ids_off
        d[0x48..0x4c].copy_from_slice(&1u32.to_le_bytes());
        d[0x4c..0x50].copy_from_slice(&(proto_ids_off as u32).to_le_bytes());
        // method_ids: count=1, off=method_ids_off
        d[0x58..0x5c].copy_from_slice(&1u32.to_le_bytes());
        d[0x5c..0x60].copy_from_slice(&(method_ids_off as u32).to_le_bytes());
        // class_defs: count=1, off=class_defs_off
        d[0x60..0x64].copy_from_slice(&1u32.to_le_bytes());
        d[0x64..0x68].copy_from_slice(&(class_defs_off as u32).to_le_bytes());

        // string_id_item[0] → str_data_off
        let sdoff = str_data_off as u32;
        d[str_ids_off..str_ids_off + 4].copy_from_slice(&sdoff.to_le_bytes());

        // type_id_item[0] → string index 0
        d[type_ids_off..type_ids_off + 4].copy_from_slice(&0u32.to_le_bytes());

        // proto_id_item[0]: shorty_idx=0, return_type=0, parameters_off=0 (12 bytes)
        // all zeros at proto_ids_off

        // method_id_item[0]: class_idx=0, proto_idx=0, name_idx=0
        d[method_ids_off..method_ids_off + 2].copy_from_slice(&0u16.to_le_bytes()); // class
        d[method_ids_off + 2..method_ids_off + 4].copy_from_slice(&0u16.to_le_bytes()); // proto
        d[method_ids_off + 4..method_ids_off + 8].copy_from_slice(&0u32.to_le_bytes()); // name

        // class_def_item[0] (32 bytes at class_defs_off)
        d[class_defs_off..class_defs_off + 4].copy_from_slice(&0u32.to_le_bytes()); // class_idx=0
        d[class_defs_off + 4..class_defs_off + 8].copy_from_slice(&1u32.to_le_bytes()); // access=public
        d[class_defs_off + 8..class_defs_off + 12].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // no super
        // interfaces=0, source=0xffffffff, annotations=0
        d[class_defs_off + 20..class_defs_off + 24].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        // class_data_off
        d[class_defs_off + 24..class_defs_off + 28]
            .copy_from_slice(&(class_data_off as u32).to_le_bytes());

        // string_data_item[0]: "v.whatsapp.net"
        let s = b"v.whatsapp.net";
        d[str_data_off] = s.len() as u8; // uleb128 length = 14
        d[str_data_off + 1..str_data_off + 1 + s.len()].copy_from_slice(s);
        d[str_data_off + 1 + s.len()] = 0; // NUL terminator

        // class_data_item: sf=0, inf=0, dm=1, vm=0; method: diff=0, access=1, code_off (uleb128)
        d[class_data_off] = 0; // sf=0
        d[class_data_off + 1] = 0; // inf=0
        d[class_data_off + 2] = 1; // dm=1
        d[class_data_off + 3] = 0; // vm=0
        // encoded_method[0]: method_idx_diff=0, access_flags=1
        d[class_data_off + 4] = 0; // method_idx_diff
        d[class_data_off + 5] = 1; // access_flags
        // code_off as ULEB128: 0x0e0 = 224 → two-byte ULEB: 0xe0 | 0x80, 0x01
        d[class_data_off + 6] = 0xe0 | 0x80; // low 7 bits of 224 = 96, with continuation
        // Actually 224 = 0b11100000; in ULEB128: first byte = (224 & 0x7f) | 0x80 = 0xe0|0x80...
        // Wait, 224 = 0b11100000. In ULEB128:
        // 224 in 7-bit groups: 1110000 and 0000001 (since 224 = 128 + 96 = 0b10000000 + 0b1100000)
        // Actually 224 = 0xE0. In binary: 11100000.
        // ULEB128 encoding: first byte = (0xE0 & 0x7F) | 0x80 = 0x60 | 0x80 = 0xE0
        // second byte = 0xE0 >> 7 = 0x01
        // So: 0xE0, 0x01
        // But 0xE0 = 11100000 in binary. Low 7 bits = 1100000 = 0x60. With MSB set = 0xE0.
        // Then second byte = 1 (no MSB), giving 0x60 | (0x01 << 7) = 0x60 | 0x80 = 0xE0.
        // So 0xE0 0x01 encodes 0x60 | (1 << 7) = 0x60 | 0x80 = 0xE0 = 224. Correct!
        d[class_data_off + 6] = 0xE0; // 0x60 | continuation bit
        d[class_data_off + 7] = 0x01; // high bits

        // code_item at 0x0e0
        d[code_item_off..code_item_off + 2].copy_from_slice(&2u16.to_le_bytes()); // registers=2
        // ins=0, outs=1, tries=0
        d[code_item_off + 4..code_item_off + 6].copy_from_slice(&1u16.to_le_bytes()); // outs=1
        // debug_info=0
        // insns_size=3 (3 code units)
        d[code_item_off + 12..code_item_off + 16].copy_from_slice(&3u32.to_le_bytes());
        // insns: const-string v0, #0 (4 bytes) + return-void (2 bytes)
        let insns_off = code_item_off + 16;
        d[insns_off] = 0x1a; // const-string
        d[insns_off + 1] = 0x00; // v0
        d[insns_off + 2] = 0x00; // string@0 low
        d[insns_off + 3] = 0x00; // string@0 high
        d[insns_off + 4] = 0x0e; // return-void
        d[insns_off + 5] = 0x00;

        // Run xref scan.
        let anchors = vec!["v.whatsapp.net".to_owned()];
        let mut out: BTreeMap<String, Vec<XRef>> = BTreeMap::new();
        xref_one("test.dex", &d, &anchors, &mut out).unwrap();

        assert!(
            out.contains_key("v.whatsapp.net"),
            "should find the anchor string"
        );
        let refs = &out["v.whatsapp.net"];
        assert_eq!(refs.len(), 1, "exactly one reference");
        assert_eq!(refs[0].class_name, "v.whatsapp.net"); // type descriptor = string 0
        assert_eq!(refs[0].method_name, "v.whatsapp.net"); // name = string 0 (same in this minimal test)
    }
}
