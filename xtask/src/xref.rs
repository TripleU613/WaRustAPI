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
        | 0x24       // filled-new-array (35c)
        | 0x25       // filled-new-array/range (3rc)
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
        0x47 => "aget-boolean",
        0x48 => "aget-byte",
        0x49 => "aget-char",
        0x4a => "aget-short",
        0x4b => "aput",
        0x4c => "aput-wide",
        0x4d => "aput-object",
        0x4e => "aput-boolean",
        0x4f => "aput-byte",
        0x50 => "aput-char",
        0x51 => "aput-short",
        // Android DEX instruction table; needed by 37.71 identity metadata
        // and B7E serialization traces (wide/boolean/byte field reads).
        0x52 => "iget",
        0x53 => "iget-wide",
        0x55 => "iget-boolean",
        0x56 => "iget-byte",
        0x54 => "iget-object",
        0x59 => "iput",
        // Android DEX 22c writes; APK 37.71 Cg3.A00 stores verified/actionSeq
        // at 025c/02a4. See docs/protocol/peer-identity.md.
        0x5a => "iput-wide",
        0x5b => "iput-object",
        0x5c => "iput-boolean",
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
        0x7b => "neg-int",
        0x7c => "not-int",
        0x7d => "neg-long",
        0x7e => "not-long",
        0x7f => "neg-float",
        0x80 => "neg-double",
        0x81 => "int-to-long",
        0x82 => "int-to-float",
        0x83 => "int-to-double",
        0x84 => "long-to-int",
        0x85 => "long-to-float",
        0x86 => "long-to-double",
        0x87 => "float-to-int",
        0x88 => "float-to-long",
        0x89 => "float-to-double",
        0x8a => "double-to-int",
        0x8b => "double-to-long",
        0x8c => "double-to-float",
        0x8d => "int-to-byte",
        0x8e => "int-to-char",
        0x8f => "int-to-short",
        0x90 => "add-int",
        0x91 => "sub-int",
        0x92 => "mul-int",
        0x93 => "div-int",
        0x94 => "rem-int",
        0x95 => "and-int",
        0x96 => "or-int",
        0x97 => "xor-int",
        0xa0 => "and-long",
        0xb0 => "add-int/2addr",
        0xb1 => "sub-int/2addr",
        0xb4 => "rem-int/2addr",
        0xb5 => "and-int/2addr",
        0xb6 => "or-int/2addr",
        0xb7 => "xor-int/2addr",
        0xbb => "add-long/2addr",
        0xbc => "sub-long/2addr",
        // CjU.A01: five unsigned bytes form a 40-bit value modulo 100000.
        // https://source.android.com/docs/core/runtime/dalvik-bytecode
        0xbf => "rem-long/2addr",
        0xc0 => "and-long/2addr",
        0xc1 => "or-long/2addr",
        0xc3 => "shl-long/2addr",
        0xd0 => "add-int/lit16",
        0xd1 => "rsub-int",
        0xd2 => "mul-int/lit16",
        0xd3 => "div-int/lit16",
        0xd4 => "rem-int/lit16",
        0xd5 => "and-int/lit16",
        0xd6 => "or-int/lit16",
        0xd7 => "xor-int/lit16",
        0xd8 => "add-int/lit8",
        0xd9 => "rsub-int/lit8",
        0xda => "mul-int/lit8",
        0xdb => "div-int/lit8",
        0xdc => "rem-int/lit8",
        0xdd => "and-int/lit8",
        0xde => "or-int/lit8",
        0xdf => "xor-int/lit8",
        0xe0 => "shl-int/lit8",
        0xe1 => "shr-int/lit8",
        0xe2 => "ushr-int/lit8",
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
    /// Human-readable opcode or payload name.
    pub mnemonic: &'static str,
    /// Decoded annotation (empty for instructions without interesting operands).
    pub annotation: String,
    /// Register-flow detail for interactive inspection; compact snapshots stay stable.
    pub flow_annotation: String,
}

/// Decode all instructions in `insns` (a slice of raw code-unit bytes,
/// length = insns_size * 2). References are resolved via `dex`.
fn decode_insns(insns: &[u8], dex: &DexFile<'_>) -> Vec<Insn> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos + 1 < insns.len() {
        let op = insns[pos];
        let reg = insns[pos + 1]; // high byte of first code unit (register / other)
        let payload = payload_at(insns, pos);
        let (width, mnemonic, payload_annotation) = match payload {
            Some(payload) => payload,
            None => (insn_width(op), opcode_name(op), String::new()),
        };
        let byte_width = width * 2;

        let annotation = if pos + byte_width > insns.len() {
            // truncated — stop
            out.push(Insn {
                byte_off: pos,
                op,
                width,
                mnemonic,
                annotation: "# truncated".into(),
                flow_annotation: String::new(),
            });
            break;
        } else if !payload_annotation.is_empty() {
            payload_annotation
        } else if op == 0x26 {
            decode_fill_array_reference(insns, pos, reg)
        } else {
            decode_annotation(op, reg, &insns[pos..pos + byte_width], dex)
        };

        out.push(Insn {
            byte_off: pos,
            op,
            width,
            mnemonic,
            annotation,
            flow_annotation: if matches!(op, 0x2b | 0x2c) {
                decode_switch_reference(insns, pos)
            } else {
                decode_register_flow(&insns[pos..pos + byte_width])
            },
        });
        pos += byte_width;
    }
    out
}

/// Return the dynamic width and description of a Dalvik payload pseudo-
/// instruction. Payloads use opcode byte zero and an identifying high byte.
fn payload_at(insns: &[u8], pos: usize) -> Option<(usize, &'static str, String)> {
    let ident = u16::from_le_bytes([*insns.get(pos)?, *insns.get(pos + 1)?]);
    let size = u16::from_le_bytes([*insns.get(pos + 2)?, *insns.get(pos + 3)?]) as usize;
    match ident {
        0x0100 => Some((
            4 + size.saturating_mul(2),
            "packed-switch-payload",
            format!("size={size}"),
        )),
        0x0200 => Some((
            2 + size.saturating_mul(4),
            "sparse-switch-payload",
            format!("size={size}"),
        )),
        0x0300 => {
            let element_width = size;
            let count = u32::from_le_bytes([
                *insns.get(pos + 4)?,
                *insns.get(pos + 5)?,
                *insns.get(pos + 6)?,
                *insns.get(pos + 7)?,
            ]) as usize;
            let byte_count = element_width.checked_mul(count)?;
            let width = 4usize.checked_add(byte_count.div_ceil(2))?;
            let data_start = pos + 8;
            let data_end = data_start.checked_add(byte_count)?;
            let data = insns.get(data_start..data_end)?;
            Some((
                width,
                "fill-array-data-payload",
                format!(
                    "element_width={element_width}, size={count}, data={}",
                    format_bytes(data)
                ),
            ))
        }
        _ => None,
    }
}

fn format_bytes(data: &[u8]) -> String {
    const DISPLAY_LIMIT: usize = 64;
    let shown = &data[..data.len().min(DISPLAY_LIMIT)];
    let mut text = shown
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    if data.len() > DISPLAY_LIMIT {
        text.push_str(" …");
    }
    format!("[{text}]")
}

fn decode_fill_array_reference(insns: &[u8], pos: usize, reg: u8) -> String {
    let Some(raw) = insns.get(pos + 2..pos + 6) else {
        return format!("v{reg}, # truncated");
    };
    let offset = i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    let target = (pos as i64).checked_add(i64::from(offset) * 2);
    let target_text = target.map_or_else(|| "overflow".into(), |v| format!("0x{v:x}"));
    let detail = target
        .and_then(|v| usize::try_from(v).ok())
        .and_then(|target_pos| payload_at(insns, target_pos))
        .map(|(_, _, annotation)| format!(", {annotation}"))
        .unwrap_or_default();
    format!("v{reg}, payload@{target_text}{detail}")
}

fn decode_switch_reference(bytes: &[u8], pos: usize) -> String {
    let read_i32 = |offset: usize| -> Option<i32> {
        Some(i32::from_le_bytes(
            bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
        ))
    };
    let Some(offset) = read_i32(pos.saturating_add(2)) else {
        return "# truncated switch".into();
    };
    let target = pos as i128 + i128::from(offset) * 2;
    let Ok(payload) = usize::try_from(target) else {
        return "# invalid switch payload".into();
    };
    let Some(header) = bytes.get(payload..payload.saturating_add(4)) else {
        return "# invalid switch payload".into();
    };
    let size = usize::from(u16::from_le_bytes([header[2], header[3]]));
    let packed = matches!(bytes.get(pos), Some(0x2b));
    if header[..2] != [0, if packed { 1 } else { 2 }] {
        return "# wrong switch payload kind".into();
    }
    let first = if packed {
        read_i32(payload.saturating_add(4))
    } else {
        Some(0)
    };
    let Some(first) = first else {
        return "# truncated switch payload".into();
    };
    let mut cases = Vec::new();
    for index in 0..size.min(64) {
        let key = if packed {
            first.checked_add(index as i32)
        } else {
            read_i32(payload.saturating_add(4 + index * 4))
        };
        let start = if packed { 8 } else { 4 + size * 4 };
        let jump = read_i32(payload.saturating_add(start + index * 4));
        let (Some(key), Some(jump)) = (key, jump) else {
            return "# truncated/invalid switch cases".into();
        };
        let destination = pos as i128 + i128::from(jump) * 2;
        let destination = if destination < 0 {
            format!("-{:#x}", -destination)
        } else {
            format!("{destination:#x}")
        };
        cases.push(format!("{key}:@{destination}"));
    }
    if size > 64 {
        cases.push("...".into());
    }
    format!(
        "v{}, payload@{payload:#x}, cases=[{}], default@{:#x}",
        bytes[pos + 1],
        cases.join(", "),
        pos + 6
    )
}

fn decode_register_flow(bytes: &[u8]) -> String {
    if bytes.len() < 2 {
        return String::new();
    }
    let reg = bytes[1];
    match bytes[0] {
        0x01 | 0x04 | 0x07 | 0x21 => format!("v{}, v{}", reg & 15, reg >> 4),
        0x02 | 0x05 | 0x08 if bytes.len() >= 4 => {
            format!("v{reg}, v{}", u16::from_le_bytes([bytes[2], bytes[3]]))
        }
        0x03 | 0x06 | 0x09 if bytes.len() >= 6 => format!(
            "v{}, v{}",
            u16::from_le_bytes([bytes[2], bytes[3]]),
            u16::from_le_bytes([bytes[4], bytes[5]])
        ),
        0x0a..=0x0d => format!("v{reg}"),
        _ => String::new(),
    }
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
            let registers = [c, d, e, f, g_reg];
            let count = usize::from(arg_count).min(registers.len());
            let args: Vec<String> = registers[..count].iter().map(|r| format!("v{r}")).collect();
            let meth = dex
                .method_info(meth_idx)
                .map(|(cls, name, shorty)| format!("{cls}->{name}({shorty})"))
                .unwrap_or_else(|_| format!("?{meth_idx}"));
            let invalid = if arg_count > 5 {
                format!(", # invalid register count {arg_count}")
            } else {
                String::new()
            };
            format!("{{{}}}, {meth}{invalid}", args.join(", "))
        }
        // filled-new-array {vC,..vG}, type@BBBB (format 35c)
        0x24 if insn_bytes.len() >= 6 => {
            let type_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let arg_count = (reg >> 4) & 0xf;
            let g_reg = reg & 0xf;
            let d4 = insn_bytes[4];
            let d5 = insn_bytes[5];
            let registers = [d4 & 0xf, d4 >> 4, d5 & 0xf, d5 >> 4, g_reg];
            let count = usize::from(arg_count).min(registers.len());
            let args = registers[..count]
                .iter()
                .map(|r| format!("v{r}"))
                .collect::<Vec<_>>()
                .join(", ");
            let ty = dex
                .type_name(type_idx)
                .unwrap_or_else(|_| format!("?{type_idx}"));
            let invalid = if arg_count > 5 {
                format!(", # invalid register count {arg_count}")
            } else {
                String::new()
            };
            format!("{{{args}}}, {ty}{invalid}")
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
            let last = first.saturating_add(u16::from(reg).saturating_sub(1));
            format!("{{v{first}..v{last}}}, {meth}")
        }
        // filled-new-array/range {vCCCC..vNNNN}, type@BBBB (format 3rc)
        0x25 if insn_bytes.len() >= 6 => {
            let type_idx = u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32;
            let first = u16::from_le_bytes([insn_bytes[4], insn_bytes[5]]);
            let last = first.saturating_add(u16::from(reg).saturating_sub(1));
            let ty = dex
                .type_name(type_idx)
                .unwrap_or_else(|_| format!("?{type_idx}"));
            format!("{{v{first}..v{last}}}, {ty}")
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
        // const/4 vA, #+B — 11n: low nibble is register, high nibble is a
        // signed four-bit literal.
        0x12 => {
            let v = reg & 0xf;
            let lit = (reg as i8) >> 4;
            format!("v{v}, #{lit:#x}")
        }
        // const/16 vAA, #+BBBB — 21s
        0x13 | 0x16 if insn_bytes.len() >= 4 => {
            let lit = i16::from_le_bytes([insn_bytes[2], insn_bytes[3]]);
            format!("v{reg}, #{lit:#x}")
        }
        // const vAA, #+BBBBBBBB — 31i
        0x14 | 0x17 if insn_bytes.len() >= 6 => {
            let lit =
                i32::from_le_bytes([insn_bytes[2], insn_bytes[3], insn_bytes[4], insn_bytes[5]]);
            format!("v{reg}, #{lit:#x}")
        }
        // const-wide vAA, #+BBBBBBBBBBBBBBBB — 51l (64-bit literal).
        0x18 if insn_bytes.len() >= 10 => {
            let mut raw = [0u8; 8];
            raw.copy_from_slice(&insn_bytes[2..10]);
            let lit = i64::from_le_bytes(raw);
            format!("v{reg}, #{lit:#x} ({lit})")
        }
        // const-wide/high16 vAA, #+BBBB000000000000 — 21h (64-bit value).
        0x19 if insn_bytes.len() >= 4 => {
            let lit = (u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u64) << 48;
            format!("v{reg}, #{lit:#x}")
        }
        // const/high16 vAA, #+BBBB0000 — 21h (32-bit value).
        0x15 if insn_bytes.len() >= 4 => {
            let lit = (u16::from_le_bytes([insn_bytes[2], insn_bytes[3]]) as u32) << 16;
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
        // cmp and aget/aput vAA, vBB, vCC — format 23x.
        0x2d..=0x31 | 0x44..=0x51 | 0xa0 if insn_bytes.len() >= 4 => {
            format!("v{reg}, v{}, v{}", insn_bytes[2], insn_bytes[3])
        }
        // Unary/conversion and binary /2addr operations vA, vB — format 12x.
        0x7b..=0x8f | 0xb0..=0xcf => {
            let va = reg & 0xf;
            let vb = (reg >> 4) & 0xf;
            format!("v{va}, v{vb}")
        }
        // Binary operation with signed 16-bit literal — format 22s.
        0xd0..=0xd7 if insn_bytes.len() >= 4 => {
            let va = reg & 0xf;
            let vb = reg >> 4;
            let lit = i16::from_le_bytes([insn_bytes[2], insn_bytes[3]]);
            format!("v{va}, v{vb}, #{lit:#x}")
        }
        // Binary operation with signed 8-bit literal — format 22b.
        0xd8..=0xe2 if insn_bytes.len() >= 4 => {
            let lit = insn_bytes[3] as i8;
            format!("v{reg}, v{}, #{lit:#x}", insn_bytes[2])
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

/// Decode both encoded-method lists in a class. DEX delta-encodes direct and
/// virtual method indices independently, so each list starts at index zero.
fn encoded_methods(
    d: &[u8],
    pos: &mut usize,
    direct_count: u32,
    virtual_count: u32,
) -> Result<Vec<(u32, u32, u32)>, String> {
    let capacity = direct_count
        .checked_add(virtual_count)
        .ok_or("dex: method count overflow")? as usize;
    let mut methods = Vec::with_capacity(capacity);
    for count in [direct_count, virtual_count] {
        let mut method_idx = 0u32;
        for _ in 0..count {
            let (method_idx_diff, next) = uleb128(d, *pos)?;
            *pos = next;
            let (access_flags, next) = uleb128(d, *pos)?;
            *pos = next;
            let (code_off, next) = uleb128(d, *pos)?;
            *pos = next;
            method_idx = method_idx
                .checked_add(method_idx_diff)
                .ok_or("dex: method index overflow")?;
            methods.push((method_idx, access_flags, code_off));
        }
    }
    Ok(methods)
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

        // Skip encoded_field entries: each is two ULEBs.
        for _ in 0..field_count {
            let (_, p) = uleb128(d, pos).unwrap_or((0, pos + 1));
            let (_, p2) = uleb128(d, p).unwrap_or((0, p + 1));
            pos = p2;
        }

        // Walk encoded_method entries with independent direct/virtual deltas.
        for (method_idx, _, code_off) in encoded_methods(d, &mut pos, dm, vm)? {
            if code_off == 0 {
                continue;
            }

            let (method_class, method_name, method_proto) =
                dex.method_info(method_idx).unwrap_or_default();
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
                let width =
                    payload_at(insns, i).map_or_else(|| insn_width(op), |(width, _, _)| width);
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

fn invoked_method_index(instruction: &[u8]) -> Option<u32> {
    let opcode = *instruction.first()?;
    if !matches!(opcode, 0x6e..=0x72 | 0x74..=0x78 | 0xfa..=0xfb) {
        return None;
    }
    Some(u32::from(u16::from_le_bytes([
        *instruction.get(2)?,
        *instruction.get(3)?,
    ])))
}

// Standard DEX field opcodes use 22c (instance) or 21c (static), with
// a little-endian field_idx in the second code unit. Do not reinterpret
// quickened odex offsets as field IDs. Literal APK 37.71 contact writes
// and a synthetic payload/truncation fixture cover this search.
fn accessed_field_index(instruction: &[u8], writes_only: bool) -> Option<u32> {
    let opcode = *instruction.first()?;
    let write = matches!(opcode, 0x59..=0x5f | 0x67..=0x6d);
    let read = matches!(opcode, 0x52..=0x58 | 0x60..=0x66);
    if !write && (!read || writes_only) {
        return None;
    }
    Some(u32::from(u16::from_le_bytes([
        *instruction.get(2)?,
        *instruction.get(3)?,
    ])))
}

fn references_one(
    dex_name: &str,
    d: &[u8],
    targets: &std::collections::HashSet<u32>,
    reference_index: impl Fn(&[u8]) -> Option<u32>,
    mut record: impl FnMut(XRef, u8),
) -> Result<(), String> {
    let dex = DexFile::parse(d)?;
    if targets.is_empty() {
        return Ok(());
    }
    for class_index in 0..dex.class_count {
        let definition_offset = dex.class_off as usize + 32 * class_index as usize;
        let class_type = u32le(d, definition_offset)?;
        let class_name = dex.type_name(class_type).unwrap_or_default();
        let class_data_offset = u32le(d, definition_offset + 24)?;
        if class_data_offset == 0 {
            continue;
        }
        let mut position = class_data_offset as usize;
        let (static_fields, next) = uleb128(d, position)?;
        position = next;
        let (instance_fields, next) = uleb128(d, position)?;
        position = next;
        let (direct_methods, next) = uleb128(d, position)?;
        position = next;
        let (virtual_methods, next) = uleb128(d, position)?;
        position = next;
        for _ in 0..static_fields.saturating_add(instance_fields) {
            let (_, next) = uleb128(d, position)?;
            let (_, next) = uleb128(d, next)?;
            position = next;
        }

        for (caller_index, _, code_offset) in
            encoded_methods(d, &mut position, direct_methods, virtual_methods)?
        {
            if code_offset == 0 {
                continue;
            }
            let (_, caller_name, caller_proto) = dex.method_info(caller_index).unwrap_or_default();
            let code_item_offset = code_offset as usize;
            let instruction_count = u32le(d, code_item_offset + 12)? as usize;
            let instruction_start = code_item_offset + 16;
            let instruction_end = instruction_start.saturating_add(instruction_count * 2);
            let instructions = d
                .get(instruction_start..instruction_end)
                .ok_or_else(|| "truncated method instruction stream".to_owned())?;
            let mut offset = 0usize;
            while offset + 1 < instructions.len() {
                let opcode = instructions[offset];
                let width = payload_at(instructions, offset)
                    .map_or_else(|| insn_width(opcode), |(width, _, _)| width);
                if width * 2 > instructions.len() - offset {
                    return Err("truncated instruction or payload".to_owned());
                }
                if let Some(callee) = reference_index(&instructions[offset..])
                    && targets.contains(&callee)
                {
                    record(
                        XRef {
                            dex_file: dex_name.to_owned(),
                            class_name: class_name.clone(),
                            method_name: caller_name.clone(),
                            method_proto: caller_proto.clone(),
                            dex_byte_off: instruction_start + offset,
                        },
                        opcode,
                    );
                }
                offset = offset.saturating_add(width.max(1) * 2);
            }
        }
    }
    Ok(())
}

fn callers_one(
    dex_name: &str,
    d: &[u8],
    target_class: &str,
    target_method: &str,
    out: &mut Vec<XRef>,
) -> Result<(), String> {
    let dex = DexFile::parse(d)?;
    let targets = (0..dex.method_count)
        .filter(|&index| {
            dex.method_info(index)
                .is_ok_and(|(class, method, _)| class == target_class && method == target_method)
        })
        .collect();
    references_one(dex_name, d, &targets, invoked_method_index, |site, _| {
        out.push(site)
    })
}

fn field_refs_one(
    dex_name: &str,
    d: &[u8],
    target_class: &str,
    target_field: &str,
    target_type: &str,
    writes_only: bool,
    out: &mut Vec<(XRef, u8)>,
) -> Result<(), String> {
    let dex = DexFile::parse(d)?;
    let mut targets = std::collections::HashSet::new();
    for index in 0..dex.field_count {
        let (class, field, ty) = dex.field_info(index)?;
        if class == target_class && field == target_field && ty == target_type {
            targets.insert(index);
        }
    }
    references_one(
        dex_name,
        d,
        &targets,
        |instruction| accessed_field_index(instruction, writes_only),
        |site, opcode| out.push((site, opcode)),
    )
}

/// Exact DEX field search, no reflective/native/quickened access claim.
/// Assemble all results before stdout so a malformed DEX cannot appear complete.
pub fn run_field_refs(
    apk: &Path,
    class: &str,
    field: &str,
    ty: &str,
    writes_only: bool,
) -> Result<(), String> {
    let file = std::fs::File::open(apk).map_err(|error| format!("{}: {error}", apk.display()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| format!("zip: {error}"))?;
    let mut names: Vec<String> = archive
        .file_names()
        .filter(|name| name.starts_with("classes") && name.ends_with(".dex"))
        .map(str::to_owned)
        .collect();
    if names.is_empty() {
        return Err("APK contains no classes*.dex files".to_owned());
    }
    names.sort_unstable();
    let mut sites = Vec::new();
    for name in names {
        let mut bytes = Vec::new();
        archive
            .by_name(&name)
            .map_err(|error| error.to_string())?
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        field_refs_one(&name, &bytes, class, field, ty, writes_only, &mut sites)
            .map_err(|error| format!("{name}: {error}"))?;
    }
    for (site, opcode) in &sites {
        println!(
            "{}  {}->{}({})",
            site.dex_file, site.class_name, site.method_name, site.method_proto
        );
        let access = if matches!(opcode, 0x59..=0x5f | 0x67..=0x6d) {
            "write"
        } else {
            "read"
        };
        println!(
            "  {access} opcode 0x{opcode:02x}, dex byte offset: 0x{:x}",
            site.dex_byte_off
        );
    }
    eprintln!(
        "{} field reference(s) to {class}->{field}:{ty}",
        sites.len()
    );
    Ok(())
}

/// Find every invoke instruction targeting an exact class and method name.
pub fn apk_callers(
    apk: &Path,
    target_class: &str,
    target_method: &str,
) -> Result<Vec<XRef>, String> {
    let file = std::fs::File::open(apk).map_err(|error| format!("{}: {error}", apk.display()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| format!("zip: {error}"))?;
    let names: Vec<String> = archive
        .file_names()
        .filter(|name| name.starts_with("classes") && name.ends_with(".dex"))
        .map(str::to_owned)
        .collect();
    let mut callers = Vec::new();
    for name in &names {
        let mut bytes = Vec::new();
        archive
            .by_name(name)
            .map_err(|error| error.to_string())?
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        if let Err(error) = callers_one(name, &bytes, target_class, target_method, &mut callers) {
            eprintln!("warning: {name}: {error}");
        }
    }
    Ok(callers)
}

pub fn run_callers(apk: &Path, target_class: &str, target_method: &str) -> Result<(), String> {
    let callers = apk_callers(apk, target_class, target_method)?;
    for caller in &callers {
        println!(
            "{}  {}->{}({})",
            caller.dex_file, caller.class_name, caller.method_name, caller.method_proto
        );
        println!("  invoke dex byte offset: 0x{:x}", caller.dex_byte_off);
    }
    eprintln!(
        "{} caller(s) of {}->{}",
        callers.len(),
        target_class,
        target_method
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

        for (method_idx, access_flags, code_off) in encoded_methods(d, &mut pos, dm, vm)? {
            let (_, method_name, method_proto) = dex.method_info(method_idx).unwrap_or_default();

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
            let name = if insn.mnemonic == "??" {
                format!("??(0x{:02x})", insn.op)
            } else {
                insn.mnemonic.to_owned()
            };
            let end = insn.byte_off + insn.width * 2;
            let annotation = if insn.annotation.is_empty() {
                &insn.flow_annotation
            } else {
                &insn.annotation
            };
            if annotation.is_empty() {
                println!("    {:04x}..{:04x}  {name}", insn.byte_off, end);
            } else {
                println!(
                    "    {:04x}..{:04x}  {name}  {}",
                    insn.byte_off, end, annotation
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

/// Decode a protobuf-lite message descriptor's field numbers from the
/// `newMessageInfo` string a `dynamicMethod` builds.
///
/// Derive field numbers from the descriptor rather than assuming static values
/// were recovered: `GeneratedMessageLite.newMessageInfo` receives field names and an encoded
/// string pairing each with its number and type. Reading a number any other way
/// — from position in the name array, from the public schema, or from memory —
/// has produced wrong answers in this project before, and a wrong field number
/// is self-consistent locally and unreadable by every real client.
///
/// The descriptor has a ten-integer header and variable-width field entries:
/// presence-bearing scalar fields have an additional index; lists do not.
/// Validate the entire descriptor and exact field-name count before pairing.
pub fn run_proto_fields(apk: &Path, class_filter: &str) -> Result<(), String> {
    let methods = apk_smali(apk, class_filter, Some("dynamicMethod"))?;
    let mut printed = 0usize;
    for method in &methods {
        let class = method.class_name.clone();
        let literals: Vec<String> = method
            .insns
            .iter()
            .filter_map(|insn| {
                let annotation = if insn.annotation.is_empty() {
                    &insn.flow_annotation
                } else {
                    &insn.annotation
                };
                quoted_literal(annotation)
            })
            .collect();
        let names: Vec<String> = literals
            .iter()
            .filter(|literal| {
                literal.ends_with('_')
                    && literal
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
            .cloned()
            .collect();
        let candidates: Vec<_> = literals
            .iter()
            .filter(|literal| literal.starts_with("\\u{1}") || literal.starts_with("\\0"))
            .map(|literal| decode_proto_field_numbers(&unescape(literal)))
            .collect();
        let decoded: Vec<_> = candidates.iter().filter_map(|r| r.as_ref().ok()).collect();
        if decoded.len() != 1 {
            let reason = candidates
                .iter()
                .find_map(|r| r.as_ref().err())
                .copied()
                .unwrap_or("missing or ambiguous descriptor");
            eprintln!("{class}: {reason}; not guessing");
            continue;
        }
        let numbers = decoded[0];
        // The bitField bookkeeping names may be supplied by helper calls, or
        // occur as literals. Neither is a protobuf field name. Oneof objects
        // need distinct object-array tracing and are explicitly refused below.
        let names: Vec<_> = names
            .iter()
            .filter(|name| !is_presence_bookkeeping(name))
            .collect();
        if numbers.len() != names.len() {
            eprintln!(
                "{class}: decoded {} fields against {} names; not guessing",
                numbers.len(),
                names.len()
            );
            continue;
        }
        println!("# {class}  ({} fields)", numbers.len());
        for (name, number) in names.iter().zip(numbers) {
            println!("{number:>4}  {}", name.trim_end_matches('_'));
        }
        printed += 1;
    }
    if printed == 0 {
        eprintln!("no protobuf descriptor decoded for {class_filter}");
    }
    Ok(())
}

fn is_presence_bookkeeping(name: &str) -> bool {
    name.strip_prefix("bitField")
        .and_then(|s| s.strip_suffix('_'))
        .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}

/// Field-entry layout follows protobuf MessageSchema.newSchemaForRawMessageInfo
/// (official Java runtime): low byte is type, HAS_HAS_BIT=0x1000 adds a presence
/// index only for scalar types <=17. The former fixed triples silently stopped
/// at repeated fields such as APK 2.26.37.71 BX5.dynamicMethod's type 26.
fn decode_proto_field_numbers(info: &str) -> Result<Vec<u32>, &'static str> {
    fn integer(input: &mut std::str::Chars<'_>) -> Result<u32, &'static str> {
        let mut value = 0u32;
        let mut shift = 0;
        loop {
            let c = input.next().ok_or("truncated descriptor integer")? as u32;
            if c > u16::MAX as u32 {
                return Err("non-UTF16 descriptor integer");
            }
            let part = if c >= 0xd800 { c & 0x1fff } else { c };
            if shift >= 32 || part > (u32::MAX >> shift) {
                return Err("descriptor integer overflow");
            }
            value |= part << shift;
            if c < 0xd800 {
                return Ok(value);
            }
            shift += 13;
        }
    }
    let mut input = info.chars();
    let flags = integer(&mut input)?;
    if flags > 1 {
        return Err("unsupported descriptor flags");
    }
    let count = integer(&mut input)?;
    if count == 0 {
        return if input.next().is_none() {
            Ok(Vec::new())
        } else {
            Err("trailing empty descriptor data")
        };
    }
    if count > 4096 {
        return Err("descriptor field count exceeds bound");
    }
    let oneofs = integer(&mut input)?;
    let presence_words = integer(&mut input)?;
    let min = integer(&mut input)?;
    let max = integer(&mut input)?;
    let entries = integer(&mut input)?;
    let maps = integer(&mut input)?;
    let lists = integer(&mut input)?;
    let initialized = integer(&mut input)?;
    if oneofs != 0 {
        return Err("oneof field-name pairing is unsupported");
    }
    if entries != count || presence_words > count || min == 0 || max > 0x1fff_ffff {
        return Err("invalid descriptor header");
    }
    let mut numbers = Vec::with_capacity(count as usize);
    let mut seen_maps = 0;
    let mut seen_lists = 0;
    let mut seen_initialized = 0;
    for _ in 0..count {
        let number = integer(&mut input)?;
        let kind = integer(&mut input)?;
        let base = kind & 0xff;
        if number == 0
            || number > 0x1fff_ffff
            || numbers.last().is_some_and(|previous| number <= *previous)
            || base > 50
            || kind & !0x1fff != 0
        {
            return Err("invalid descriptor field entry");
        }
        if kind & 0x1000 != 0 {
            if base > 17 {
                return Err("presence on a non-scalar field");
            }
            let presence = integer(&mut input)?;
            if presence >= presence_words * 32 {
                return Err("presence index outside descriptor header");
            }
        }
        seen_maps += u32::from(base == 50);
        seen_lists += u32::from((18..=49).contains(&base));
        seen_initialized += u32::from(kind & 0x400 != 0);
        numbers.push(number);
    }
    if input.next().is_some()
        || numbers.first() != Some(&min)
        || numbers.last() != Some(&max)
        || maps != seen_maps
        || lists != seen_lists
        || initialized != seen_initialized
    {
        return Err("descriptor header or length mismatch");
    }
    Ok(numbers)
}

/// The quoted literal inside an instruction annotation, honouring `\"`.
///
/// Splitting on `"` truncates any literal containing an escaped quote — and a
/// protobuf descriptor does, because the field-number encoding uses the raw
/// character for field 34, which is `"`. That truncation is silent: the walk
/// simply stops at field 34 and a 110-field message decodes as a 34-field one.
fn quoted_literal(annotation: &str) -> Option<String> {
    let chars: Vec<char> = annotation.chars().collect();
    let open = chars.iter().position(|c| *c == '"')?;
    let mut out = String::new();
    let mut i = open + 1;
    while i < chars.len() {
        match chars[i] {
            '"' => return Some(out),
            '\\' if i + 1 < chars.len() => {
                out.push('\\');
                out.push(chars[i + 1]);
                i += 2;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    None
}

/// Decode the disassembler's escaping, entirely in `char` positions. An earlier
/// version mixed a byte offset from `str::find` with a char index and panicked
/// on the first multi-byte escape it met.
fn unescape(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '\\' || i + 1 >= chars.len() {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        match chars[i + 1] {
            'u' if i + 2 < chars.len() && chars[i + 2] == '{' => {
                let decoded = chars[i + 3..]
                    .iter()
                    .position(|c| *c == '}')
                    .and_then(|len| {
                        let hex: String = chars[i + 3..i + 3 + len].iter().collect();
                        u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .map(|c| (c, len))
                    });
                match decoded {
                    Some((c, len)) => {
                        out.push(c);
                        i += 4 + len;
                    }
                    None => {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
            }
            '0' => {
                out.push('\0');
                i += 2;
            }
            't' => {
                out.push('\t');
                i += 2;
            }
            'n' => {
                out.push('\n');
                i += 2;
            }
            'r' => {
                out.push('\r');
                i += 2;
            }
            other => {
                out.push(other);
                i += 2;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn switch_references_resolve_signed_targets_and_reject_truncation() {
        let packed = [
            0x2b, 3, 4, 0, 0, 0, 0, 0, 0, 1, 2, 0, 29, 0, 0, 0, 3, 0, 0, 0, 0xff, 0xff, 0xff, 0xff,
        ];
        assert_eq!(
            decode_switch_reference(&packed, 0),
            "v3, payload@0x8, cases=[29:@0x6, 30:@-0x2], default@0x6"
        );
        let sparse = [
            0x2c, 1, 4, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0xff, 0xff, 0xff, 0xff, 3, 0, 0, 0,
        ];
        assert_eq!(
            decode_switch_reference(&sparse, 0),
            "v1, payload@0x8, cases=[-1:@0x6], default@0x6"
        );
        for end in 0..packed.len() {
            assert!(decode_switch_reference(&packed[..end], 0).starts_with('#'));
        }
    }

    #[test]
    fn register_flow_formats_and_truncation() {
        for op in [0x01, 0x04, 0x07, 0x21] {
            assert_eq!(decode_register_flow(&[op, 0xa3]), "v3, v10");
        }
        for op in [0x02, 0x05, 0x08] {
            assert_eq!(decode_register_flow(&[op, 3, 0x01, 0x02]), "v3, v513");
            assert!(decode_register_flow(&[op, 3]).is_empty());
        }
        for op in [0x03, 0x06, 0x09] {
            assert_eq!(decode_register_flow(&[op, 0, 1, 2, 3, 4]), "v513, v1027");
            assert!(decode_register_flow(&[op, 0, 1, 2]).is_empty());
        }
        for op in 0x0a..=0x0d {
            assert_eq!(decode_register_flow(&[op, 255]), "v255");
        }
        assert!(decode_register_flow(&[]).is_empty());
        assert!(decode_register_flow(&[0]).is_empty());
        assert!(decode_register_flow(&[0x0e, 0]).is_empty());
    }

    use super::*;

    #[test]
    fn mixed_history_metadata_descriptor_uses_variable_width_entries() {
        // Exact 2.26.37.71 BX5.dynamicMethod descriptor at 0066..006a.
        // Repeated string fields 1/4 have no presence index; int64 2/3/5 do.
        let info = "\u{1}\u{5}\0\u{1}\u{1}\u{5}\u{5}\0\u{2}\0\u{1}\u{1a}\u{2}\u{1002}\0\u{3}\u{1002}\u{1}\u{4}\u{1a}\u{5}\u{1002}\u{2}";
        assert_eq!(
            decode_proto_field_numbers(info).unwrap(),
            vec![1, 2, 3, 4, 5]
        );
        for (end, _) in info.char_indices() {
            assert!(decode_proto_field_numbers(&info[..end]).is_err());
        }
        assert!(decode_proto_field_numbers(&format!("{info}\0")).is_err());
        assert!(is_presence_bookkeeping("bitField0_"));
        assert!(is_presence_bookkeeping("bitField12_"));
        assert!(!is_presence_bookkeeping("bitFieldName_"));
        assert!(!is_presence_bookkeeping("messageCount_"));
    }

    #[test]
    fn descriptor_checks_header_presence_types_and_complete_input() {
        // Singular proto2 bool with presence bit 0; no assumption that its
        // presence index equals its field number or array position.
        let valid = "\u{1}\u{1}\0\u{1}\u{5}\u{5}\u{1}\0\0\0\u{5}\u{1007}\0";
        assert_eq!(decode_proto_field_numbers(valid).unwrap(), vec![5]);
        let mut values: Vec<char> = valid.chars().collect();
        for (index, replacement) in [
            (0, 2),
            (1, 2),
            (2, 1),
            (3, 0),
            (4, 1),
            (5, 6),
            (6, 2),
            (7, 1),
            (8, 1),
            (9, 1),
            (10, 0),
            (11, 0x1033),
            (12, 32),
        ] {
            let old = values[index];
            values[index] = char::from_u32(replacement).unwrap();
            assert!(decode_proto_field_numbers(&values.iter().collect::<String>()).is_err());
            values[index] = old;
        }
        assert_eq!(
            decode_proto_field_numbers("\0\0").unwrap(),
            Vec::<u32>::new()
        );
        // A proto3 bool without presence consumes exactly number/type.
        assert_eq!(
            decode_proto_field_numbers("\0\u{1}\0\0\u{1}\u{1}\u{1}\0\0\0\u{1}\u{7}").unwrap(),
            vec![1]
        );
    }

    #[test]
    fn descriptor_integers_support_utf16_continuations_and_refuse_overflow() {
        // 65536 encodes as continuation 0xe000 followed by terminal 8.
        let info = "\u{1}\u{1}\0\0\u{e000}\u{8}\u{e000}\u{8}\u{1}\0\0\0\u{e000}\u{8}\u{7}";
        assert_eq!(decode_proto_field_numbers(info).unwrap(), vec![65536]);
        for info in [
            "\u{e000}",
            "\u{e000}\u{e000}\u{e000}\u{ffff}\u{1}",
            "\u{10000}",
            "\u{1}\0\0",
        ] {
            assert!(decode_proto_field_numbers(info).is_err());
        }
    }

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
                (1..=5).contains(&w),
                "opcode 0x{op:02x} width {w} out of expected range"
            );
        }
    }

    #[test]
    fn insn_width_key_opcodes() {
        assert_eq!(insn_width(0x1a), 2, "const-string");
        assert_eq!(insn_width(0x1b), 3, "const-string/jumbo");
        assert_eq!(insn_width(0x24), 3, "filled-new-array");
        assert_eq!(insn_width(0x25), 3, "filled-new-array/range");
        assert_eq!(insn_width(0x6e), 3, "invoke-virtual");
        assert_eq!(insn_width(0x71), 3, "invoke-static");
        assert_eq!(insn_width(0x22), 2, "new-instance");
        assert_eq!(insn_width(0x18), 5, "const-wide");
        assert_eq!(insn_width(0xfa), 4, "invoke-polymorphic");
        assert_eq!(insn_width(0x0e), 1, "return-void");
    }

    #[test]
    fn invoke_method_index_covers_normal_range_and_polymorphic_forms() {
        assert_eq!(
            invoked_method_index(&[0x6e, 0x10, 0x34, 0x12]),
            Some(0x1234)
        );
        assert_eq!(
            invoked_method_index(&[0x74, 0x01, 0xcd, 0xab]),
            Some(0xabcd)
        );
        assert_eq!(
            invoked_method_index(&[0xfa, 0x10, 0x78, 0x56]),
            Some(0x5678)
        );
        assert_eq!(invoked_method_index(&[0x0e, 0x00, 0x34, 0x12]), None);
        assert_eq!(invoked_method_index(&[0x71, 0x00, 0x34]), None);
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

        // A corrupt 35c register count greater than the format's five slots
        // is annotated rather than indexing past the decoded register array.
        let malformed = &[0x71, 0xf0, 0, 0, 0, 0];
        let decoded = decode_insns(malformed, &dex);
        assert!(decoded[0].annotation.contains("invalid register count 15"));
    }

    #[test]
    fn decodes_const_nibbles_and_fill_array_payload() {
        let d = minimal_dex_header();
        let dex = DexFile::parse(&d).unwrap();
        let insns = [
            0x12, 0x41, // const/4 v1, #4
            0x26, 0x00, 0x03, 0x00, 0x00, 0x00, // payload is 3 code units ahead
            0x00, 0x03, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, // payload header
            0x57, 0x41, 0x06, 0x03, // payload bytes
        ];
        let decoded = decode_insns(&insns, &dex);
        assert_eq!(decoded[0].annotation, "v1, #0x4");
        assert!(decoded[1].annotation.contains("data=[57 41 06 03]"));
        assert_eq!(decoded[2].mnemonic, "fill-array-data-payload");
        assert_eq!(decoded[2].width, 6);
    }

    #[test]
    fn decodes_high16_frame_bounds_and_signed_bit_patterns() {
        let d = minimal_dex_header();
        let dex = DexFile::parse(&d).unwrap();
        let insns = [
            0x15, 0x00, 0x00, 0x01, // v0 = 1 << 24, APK frame rejection bound
            0x15, 0xff, 0x00, 0x80, // v255 = i32::MIN bit pattern
            0x15, 0x01, 0xff, 0xff, // v1 = -65536 bit pattern
            0x15, 0x02, 0x00, 0x00, // v2 = 0
        ];
        let decoded = decode_insns(&insns, &dex);
        assert_eq!(decoded[0].annotation, "v0, #0x1000000");
        assert_eq!(decoded[1].annotation, "v255, #0x80000000");
        assert_eq!(decoded[2].annotation, "v1, #0xffff0000");
        assert_eq!(decoded[3].annotation, "v2, #0x0");
        assert_eq!(decoded[0].width, 2);
        for end in 0..4 {
            let _ = decode_insns(&insns[..end], &dex);
        }
    }

    #[test]
    fn decodes_xor_literal_and_conversion_operands() {
        let d = minimal_dex_header();
        let dex = DexFile::parse(&d).unwrap();
        let insns = [
            0xdf, 0x03, 0x04, 0x12, // xor-int/lit8 v3, v4, #0x12
            0x8e, 0x53, // int-to-char v3, v5
        ];
        let decoded = decode_insns(&insns, &dex);
        assert_eq!(decoded[0].mnemonic, "xor-int/lit8");
        assert_eq!(decoded[0].annotation, "v3, v4, #0x12");
        assert_eq!(decoded[1].mnemonic, "int-to-char");
        assert_eq!(decoded[1].annotation, "v3, v5");
    }

    #[test]
    fn decodes_wide_timestamp_constant_and_comparison_order() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        let bytes = [0x16, 0x00, 0xff, 0xff, 0x31, 0x00, 0x04, 0x01];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(decoded[0].mnemonic, "const-wide/16");
        assert_eq!(decoded[0].annotation, "v0, #0xffff");
        assert_eq!(decoded[1].mnemonic, "cmp-long");
        assert_eq!(decoded[1].annotation, "v0, v4, v1");
        for end in 0..bytes.len() {
            let _ = decode_insns(&bytes[..end], &dex);
        }
    }

    #[test]
    fn decodes_apk_safety_number_unsigned_group_arithmetic() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        // 2.26.37.71 CjU.A01 offsets 000a,0010,0024,0062, preserving registers.
        let bytes = [0xc0, 0x64, 0xc3, 0x04, 0xc1, 0x14, 0xbf, 0x02];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(
            decoded
                .iter()
                .map(|v| (v.mnemonic, v.annotation.as_str()))
                .collect::<Vec<_>>(),
            [
                ("and-long/2addr", "v4, v6"),
                ("shl-long/2addr", "v4, v0"),
                ("or-long/2addr", "v4, v1"),
                ("rem-long/2addr", "v2, v0")
            ]
        );
        assert_eq!(
            decoded.last().unwrap().byte_off + decoded.last().unwrap().width * 2,
            bytes.len()
        );
    }
    #[test]
    fn decodes_apk_verification_boolean_and_sequence_writes() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        // Literal 37.71 classes7.dex bytes at 0x5fc138 and 0x5fc180,
        // Cg3.A00 method offsets 025c and 02a4. Minimal DEX intentionally
        // leaves real field indices unresolved instead of inventing fields.
        let bytes = [0x5c, 0x16, 0xa8, 0x29, 0x5a, 0x13, 0xa4, 0x29];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(
            decoded.iter().map(|v| v.mnemonic).collect::<Vec<_>>(),
            ["iput-boolean", "iput-wide"]
        );
        assert!(decoded[0].annotation.starts_with("v6, v1,"));
        assert!(decoded[1].annotation.starts_with("v3, v1,"));
        assert_eq!(decoded[0].byte_off, 0);
        assert_eq!(decoded[1].byte_off, 4);
        assert!(decoded.iter().all(|v| v.width == 2));
        assert_eq!(decoded.last().unwrap().byte_off + 4, bytes.len());
    }

    #[test]
    fn decodes_identity_metadata_and_serialized_type_field_reads() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        // 22c operand layouts seen in 0jc metadata and B7E.A00; field index zero
        // is deliberately unresolved in this minimal fixture, not fabricated.
        let bytes = [0x53, 0x50, 0, 0, 0x55, 0x50, 0, 0, 0x56, 0x50, 0, 0];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(
            decoded.iter().map(|v| v.mnemonic).collect::<Vec<_>>(),
            ["iget-wide", "iget-boolean", "iget-byte"]
        );
        assert!(decoded.iter().all(|v| v.annotation.starts_with("v0, v5,")));
        assert_eq!(
            decoded.iter().map(|v| v.width).collect::<Vec<_>>(),
            [2, 2, 2]
        );
    }

    #[test]
    fn decodes_wide_age_subtraction_and_bootstrap_timeout() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        let bytes = [
            0x17, 0x04, 0xe0, 0x93, 0x04, 0x00, // const-wide/32 v4, 300000
            0xbc, 0x03, // sub-long/2addr v3, v0 (age = now - stored)
            0xbb, 0x28, // add-long/2addr v8, v2
        ];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(decoded[0].mnemonic, "const-wide/32");
        assert_eq!(decoded[0].annotation, "v4, #0x493e0");
        assert_eq!(decoded[1].mnemonic, "sub-long/2addr");
        assert_eq!(decoded[1].annotation, "v3, v0");
        assert_eq!(decoded[2].mnemonic, "add-long/2addr");
        assert_eq!(decoded[2].annotation, "v8, v2");
        for end in 0..bytes.len() {
            let _ = decode_insns(&bytes[..end], &dex);
        }
    }

    #[test]
    fn decodes_a_full_width_wide_literal() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        // const-wide v8, 25 — the 51l form, literal little-endian over 8 bytes.
        let bytes = [0x18, 0x08, 0x19, 0, 0, 0, 0, 0, 0, 0];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(decoded[0].mnemonic, "const-wide");
        assert_eq!(decoded[0].annotation, "v8, #0x19 (25)");
        for end in 0..bytes.len() {
            let _ = decode_insns(&bytes[..end], &dex);
        }
    }

    #[test]
    fn decodes_unsigned_byte_mask_and_signed_lit16() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        let bytes = [0xd5, 0x01, 0xff, 0x00, 0xd0, 0x43, 0xff, 0xff];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(decoded[0].mnemonic, "and-int/lit16");
        assert_eq!(decoded[0].annotation, "v1, v0, #0xff");
        assert_eq!(decoded[1].annotation, "v3, v4, #0xffff");
        for end in 0..bytes.len() {
            let _ = decode_insns(&bytes[..end], &dex);
        }
    }

    fn field_fixture(code: &[u8]) -> Vec<u8> {
        let mut data = minimal_dex_header();
        data.resize(0x200, 0);
        let strings = ["LX/Contact;", "state", "I", "update", "V"];
        let mut body = 0x150;
        for (index, value) in strings.iter().enumerate() {
            data[0x70 + index * 4..0x74 + index * 4].copy_from_slice(&(body as u32).to_le_bytes());
            data[body] = value.len() as u8;
            data[body + 1..body + 1 + value.len()].copy_from_slice(value.as_bytes());
            body += value.len() + 2;
        }
        for (header, count, offset) in [
            (0x38, 5u32, 0x70u32),
            (0x40, 3, 0x88),
            (0x48, 1, 0x94),
            (0x50, 1, 0xa0),
            (0x58, 1, 0xa8),
            (0x60, 1, 0xb0),
        ] {
            data[header..header + 4].copy_from_slice(&count.to_le_bytes());
            data[header + 4..header + 8].copy_from_slice(&offset.to_le_bytes());
        }
        // Types: Contact/I/V; one method update():V; field Contact.state:I.
        data[0x8c..0x90].copy_from_slice(&2u32.to_le_bytes());
        data[0x90..0x94].copy_from_slice(&4u32.to_le_bytes());
        data[0x94..0x98].copy_from_slice(&4u32.to_le_bytes());
        data[0x98..0x9c].copy_from_slice(&2u32.to_le_bytes());
        data[0xa2..0xa4].copy_from_slice(&1u16.to_le_bytes());
        data[0xa4..0xa8].copy_from_slice(&1u32.to_le_bytes());
        data[0xac..0xb0].copy_from_slice(&3u32.to_le_bytes());
        data[0xc8..0xcc].copy_from_slice(&0xe0u32.to_le_bytes());
        // class_data: zero fields, one direct method at code offset 0x100.
        data[0xe0..0xe8].copy_from_slice(&[0, 0, 1, 0, 0, 1, 0x80, 2]);
        data[0x10c..0x110].copy_from_slice(&((code.len() / 2) as u32).to_le_bytes());
        data[0x110..0x110 + code.len()].copy_from_slice(code);
        data
    }

    #[test]
    fn exact_field_search_skips_payload_and_selects_writes() {
        // Standard DEX 22c/21c forms. Embedded iput bytes in a real
        // fill-array-data payload must not become a spurious field write.
        let code = [
            0x52, 0x10, 0, 0, 0x59, 0x10, 0, 0, 0x60, 0, 0, 0, 0x67, 0, 0, 0, 0, 3, 1, 0, 4, 0, 0,
            0, 0x59, 0x10, 0, 0, 0x0e, 0,
        ];
        let data = field_fixture(&code);
        let mut all = Vec::new();
        field_refs_one(
            "classes.dex",
            &data,
            "LX/Contact;",
            "state",
            "I",
            false,
            &mut all,
        )
        .unwrap();
        assert_eq!(
            all.iter()
                .map(|(site, op)| (site.dex_byte_off, *op))
                .collect::<Vec<_>>(),
            [(0x110, 0x52), (0x114, 0x59), (0x118, 0x60), (0x11c, 0x67)]
        );
        assert!(all.iter().all(|(site, _)| site.class_name == "LX/Contact;"
            && site.method_name == "update"
            && site.method_proto == "V"));
        let mut writes = Vec::new();
        field_refs_one(
            "classes.dex",
            &data,
            "LX/Contact;",
            "state",
            "I",
            true,
            &mut writes,
        )
        .unwrap();
        assert_eq!(
            writes.iter().map(|(_, op)| *op).collect::<Vec<_>>(),
            [0x59, 0x67]
        );
        for (class, field, ty) in [
            ("LX/Other;", "state", "I"),
            ("LX/Contact;", "other", "I"),
            ("LX/Contact;", "state", "J"),
        ] {
            let mut missing = Vec::new();
            field_refs_one("classes.dex", &data, class, field, ty, false, &mut missing).unwrap();
            assert!(missing.is_empty());
        }
    }

    #[test]
    fn field_search_refuses_truncated_instructions_and_quickened_offsets() {
        let data = field_fixture(&[0x59, 0x10]);
        let mut out = Vec::new();
        assert!(
            field_refs_one(
                "classes.dex",
                &data,
                "LX/Contact;",
                "state",
                "I",
                true,
                &mut out
            )
            .unwrap_err()
            .contains("truncated")
        );
        assert!(out.is_empty());
        // An odex quickened opcode's operand is an offset, not a DEX field ID.
        assert_eq!(accessed_field_index(&[0xe3, 0x10, 0, 0], false), None);
        assert_eq!(accessed_field_index(&[0x59, 0x10, 0], true), None);
    }

    #[test]
    fn decodes_pending_invitation_chat_flag_mask() {
        let data = minimal_dex_header();
        let dex = DexFile::parse(&data).unwrap();
        // APK 37.71 classes3.dex 1xz.A01 relative 0094, absolute 1e0094:
        // this masks the wide chat-state register pair, not isGuest in v1.
        let bytes = [0xa0, 0x07, 0x07, 0x13];
        let decoded = decode_insns(&bytes, &dex);
        assert_eq!(decoded[0].mnemonic, "and-long");
        assert_eq!(decoded[0].annotation, "v7, v7, v19");
        assert_eq!(decoded[0].width, 2);
        for end in 0..bytes.len() {
            let _ = decode_insns(&bytes[..end], &dex);
        }
    }

    #[test]
    fn direct_and_virtual_method_deltas_reset() {
        // direct: index 5, virtual: index 2. Each entry is
        // method_idx_diff, access_flags, code_off.
        let data = [5, 1, 9, 2, 1, 7];
        let mut pos = 0;
        let methods = encoded_methods(&data, &mut pos, 1, 1).unwrap();
        assert_eq!(methods, vec![(5, 1, 9), (2, 1, 7)]);
        assert_eq!(pos, data.len());
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
