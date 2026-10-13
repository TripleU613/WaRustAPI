//! Protobuf-lite field numbers from committed research disassembly
//! (`docs/protocol/apk/**.txt`), for sessions without the APK.
//!
//! `apk-proto-fields` pairs a method's field-name literals with its encoded
//! descriptor in literal order. That is wrong whenever a shared helper writes a
//! name into the objects array: APK 2.26.37.71 `BZB.dynamicMethod` (document)
//! calls `LX/Ayp;->A1K`, which stores its argument at index 3 and the literal
//! `fileSha256_` at index 4 (retained in
//! `2.26.37.71-own-device-media/image-name-helper.txt`), so literal order loses
//! one name and shifts the rest. This reader instead executes the straight-line
//! path that builds the objects array: registers, `new-array`, `aput-object`,
//! and helper calls whose bodies are retained from the *same* APK (by SHA-256).
//! Anything it cannot follow exactly — an unretained helper receiving the array,
//! a branch, an unknown slot value — refuses the class rather than guessing.

use std::collections::BTreeMap;
use std::path::Path;

use crate::xref::{
    decode_proto_field_numbers, is_presence_bookkeeping, pair_proto_fields, quoted_literal,
    unescape,
};

/// Helper-call nesting followed before refusing.
const MAX_HELPER_DEPTH: usize = 4;
/// Instructions executed per method before refusing (no loops are followed,
/// so this only bounds malformed input).
const MAX_STEPS: usize = 4096;
/// Largest objects array accepted; `Message` (`BZc`) needs 114 slots.
const MAX_ARRAY: i64 = 4096;

/// One retained `.method` section.
#[derive(Clone, Debug)]
pub(crate) struct Method {
    pub(crate) class: String,
    pub(crate) sha256: String,
    pub(crate) name: String,
    /// Parameter registers (static methods only; wide types take two).
    pub(crate) param_regs: Option<usize>,
    pub(crate) registers: usize,
    pub(crate) insns: Vec<Insn>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Insn {
    pub(crate) offset: u32,
    /// Byte offset just past this instruction; the next one must start here.
    pub(crate) end: u32,
    pub(crate) mnemonic: String,
    pub(crate) operands: String,
}

/// Parse a retained disassembly file into its methods. Comment lines (`#`) are
/// skipped: they hold signatures, exception tables and notes, never operands.
pub(crate) fn parse_retained(text: &str) -> Vec<Method> {
    let header = |prefix: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(prefix))
            .map(|value| value.trim().to_owned())
            .unwrap_or_default()
    };
    let class = header("# APK-derived structural disassembly: ");
    let sha256 = header("# APK SHA-256: ");
    let mut methods: Vec<Method> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(".method ") {
            let name = rest.split('(').next().unwrap_or_default().to_owned();
            let shorty = rest
                .split_once('(')
                .and_then(|(_, tail)| tail.split_once(')'))
                .map(|(shorty, _)| shorty);
            let flags = attribute(rest, "flags=")
                .and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok());
            let is_static = flags.is_some_and(|f| f & 0x8 != 0);
            let param_regs = shorty.filter(|_| is_static).map(|s| {
                s.chars()
                    .skip(1)
                    .map(|c| if matches!(c, 'J' | 'D') { 2 } else { 1 })
                    .sum()
            });
            let registers = attribute(rest, "registers=")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            methods.push(Method {
                class: class.clone(),
                sha256: sha256.clone(),
                name,
                param_regs,
                registers,
                insns: Vec::new(),
            });
        } else if let Some(method) = methods.last_mut()
            && let Some(insn) = parse_insn(line)
        {
            method.insns.push(insn);
        }
    }
    methods
}

fn attribute<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let start = text.find(key)? + key.len();
    text[start..].split_whitespace().next()
}

/// `  002e..0032 op=13 const/16 v0, #0x72`
fn parse_insn(line: &str) -> Option<Insn> {
    let line = line.trim_start();
    let (range, rest) = line.split_once(' ')?;
    let (start, end) = range.split_once("..")?;
    let offset = u32::from_str_radix(start, 16).ok()?;
    let end = u32::from_str_radix(end, 16).ok()?;
    let rest = rest.strip_prefix("op=")?;
    let (_, rest) = rest.split_once(' ')?;
    let (mnemonic, operands) = rest.split_once(' ').unwrap_or((rest, ""));
    Some(Insn {
        offset,
        end,
        mnemonic: mnemonic.to_owned(),
        operands: operands.to_owned(),
    })
}

/// Retained static helper bodies, keyed by APK SHA-256, class and name.
#[derive(Default)]
pub(crate) struct Helpers {
    by_key: BTreeMap<(String, String, String), Vec<Method>>,
}

impl Helpers {
    pub(crate) fn add(&mut self, methods: Vec<Method>) {
        for method in methods {
            // Without its build's SHA-256 a body could be matched to any build.
            if method.param_regs.is_some() && !method.sha256.is_empty() {
                let key = (
                    method.sha256.clone(),
                    method.class.clone(),
                    method.name.clone(),
                );
                self.by_key.entry(key).or_default().push(method);
            }
        }
    }

    /// Every retained `.txt` under `root`.
    pub(crate) fn load(root: &Path) -> Result<Self, String> {
        let mut helpers = Self::default();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let entries = std::fs::read_dir(&dir)
                .map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
            for entry in entries {
                let path = entry.map_err(|e| e.to_string())?.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "txt") {
                    let text = std::fs::read_to_string(&path)
                        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                    helpers.add(parse_retained(&text));
                }
            }
        }
        Ok(helpers)
    }

    /// The one body for this call. Several retained copies must agree, or the
    /// call is treated as unknown.
    fn find(&self, sha256: &str, class: &str, name: &str, args: usize) -> Option<&Method> {
        if sha256.is_empty() {
            return None;
        }
        let bodies = self
            .by_key
            .get(&(sha256.to_owned(), class.to_owned(), name.to_owned()))?;
        let matching: Vec<_> = bodies
            .iter()
            .filter(|m| m.param_regs == Some(args))
            .collect();
        let first = *matching.first()?;
        matching
            .iter()
            .all(|m| m.insns == first.insns && m.registers == first.registers)
            .then_some(first)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Val {
    Int(i64),
    Str(String),
    Arr(usize),
    /// A known non-string object: a class literal or a static non-String field.
    Object,
    /// The result of a call that was not followed (body not retained).
    Opaque(String),
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
enum Slot {
    Name(String),
    Object,
}

#[derive(Clone)]
struct Arrays {
    slots: Vec<Vec<Option<Val>>>,
    /// Why an array may hold anything: it was handed to code that was not
    /// followed, or written in a way that was not modelled.
    poisoned: Vec<Option<String>>,
}

impl Arrays {
    fn poison(&mut self, id: usize, why: &str) {
        self.poisoned[id].get_or_insert_with(|| why.to_owned());
    }

    fn poison_all(&mut self, why: &str) {
        for id in 0..self.poisoned.len() {
            self.poison(id, why);
        }
    }
}

/// What following one method produced.
enum Outcome {
    Returned(Val),
    Descriptor {
        info: Val,
        objects: Val,
    },
    /// `throw`, or a `return` on the entry path before `newMessageInfo`.
    Ended,
}

struct Ctx<'a> {
    helpers: &'a Helpers,
    used: Vec<String>,
}

/// `(field number, name)` pairs and the helpers that supplied names.
type Decoded = (Vec<(u32, String)>, Vec<String>);

/// The decoded `(field number, name)` pairs of a retained `dynamicMethod`, and
/// the helpers it went through.
pub(crate) fn descriptor_fields(method: &Method, helpers: &Helpers) -> Result<Decoded, String> {
    let mut ctx = Ctx {
        helpers,
        used: Vec::new(),
    };
    let regs = vec![Val::Unknown; method.registers.max(1)];
    let mut arrays = Arrays {
        slots: Vec::new(),
        poisoned: Vec::new(),
    };
    let (info, objects) = entry_descriptor(&mut ctx, method, regs, &mut arrays)?;
    let Val::Str(info) = info else {
        return Err("descriptor argument is not a string constant".into());
    };
    let id = match objects {
        Val::Arr(id) => id,
        Val::Opaque(call) => {
            return Err(format!(
                "objects array comes from {call}, whose body is not retained"
            ));
        }
        _ => return Err("objects argument is not a followed array".into()),
    };
    if let Some(why) = &arrays.poisoned[id] {
        return Err(format!("objects array {why}"));
    }
    let mut slots = Vec::new();
    for (index, value) in arrays.slots[id].iter().enumerate() {
        slots.push(match value {
            Some(Val::Str(s)) => Slot::Name(s.clone()),
            Some(Val::Object) => Slot::Object,
            Some(Val::Opaque(call)) => {
                return Err(format!(
                    "objects[{index}] comes from {call}, whose body is not retained"
                ));
            }
            Some(_) => return Err(format!("objects[{index}] holds an unknown value")),
            None => return Err(format!("objects[{index}] is never written")),
        });
    }
    let numbers = decode_proto_field_numbers(&unescape(&info)).map_err(str::to_owned)?;
    let mut names = Vec::new();
    for slot in slots {
        let Slot::Name(name) = slot else { continue };
        let field = name
            .strip_suffix('_')
            .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        let Some(field) = field else {
            return Err(format!(
                "objects holds a string that is not a field name: {name:?}"
            ));
        };
        if !is_presence_bookkeeping(&name) {
            names.push(field.to_owned());
        }
    }
    if numbers.len() != names.len() {
        return Err(format!(
            "decoded {} fields against {} names",
            numbers.len(),
            names.len()
        ));
    }
    Ok((numbers.into_iter().zip(names).collect(), ctx.used))
}

/// Follow the entry method. A leading `packed-switch` on the
/// `MethodToInvoke` ordinal fans out; exactly one case must reach
/// `newMessageInfo`, so no enum ordinal is assumed.
fn entry_descriptor(
    ctx: &mut Ctx<'_>,
    method: &Method,
    regs: Vec<Val>,
    arrays: &mut Arrays,
) -> Result<(Val, Val), String> {
    let mut targets = Vec::new();
    if let Outcome::Descriptor { info, objects } = follow(
        ctx,
        method,
        first_offset(method)?,
        regs,
        arrays,
        0,
        Some(&mut targets),
    )? {
        return Ok((info, objects));
    }
    let Some((switch_regs, cases)) = targets.pop() else {
        return Err("no newMessageInfo call on the followed path".into());
    };
    // Other cases (the parser's double-checked lock, for one) branch and are
    // not followed. That is safe only while the call site is unique: a case
    // that fails then cannot reach a second, differently built descriptor.
    let calls: Vec<u32> = method
        .insns
        .iter()
        .filter(|i| i.mnemonic.starts_with("invoke-") && is_new_message_info(&i.operands))
        .map(|i| i.offset)
        .collect();
    let [call] = calls[..] else {
        return Err(format!("{} newMessageInfo call sites", calls.len()));
    };
    // The case whose block holds the call, for a precise refusal reason.
    let home = cases.iter().copied().filter(|t| *t <= call).max();
    let mut found = Vec::new();
    let mut home_error = None;
    for target in cases {
        let mut branch = arrays.clone();
        let before = ctx.used.len();
        let outcome = follow(
            ctx,
            method,
            target,
            switch_regs.clone(),
            &mut branch,
            0,
            None,
        );
        let used = ctx.used.split_off(before);
        match outcome {
            Ok(Outcome::Descriptor { info, objects }) => found.push((info, objects, branch, used)),
            Err(error) if Some(target) == home => home_error = Some(error),
            _ => {}
        }
    }
    match found.pop() {
        Some((info, objects, branch, used)) if found.is_empty() => {
            *arrays = branch;
            ctx.used.extend(used);
            Ok((info, objects))
        }
        Some(_) => Err("more than one switch case reaches newMessageInfo".into()),
        None => Err(home_error.unwrap_or_else(|| "no switch case reaches newMessageInfo".into())),
    }
}

type SwitchTargets = Vec<(Vec<Val>, Vec<u32>)>;

/// Follow straight-line code from `start`. `switch` is `Some` only on the
/// entry method's first pass: a `packed-switch` there records the register
/// state and case targets and stops.
fn follow(
    ctx: &mut Ctx<'_>,
    method: &Method,
    start: u32,
    mut regs: Vec<Val>,
    arrays: &mut Arrays,
    depth: usize,
    mut switch: Option<&mut SwitchTargets>,
) -> Result<Outcome, String> {
    let first = method
        .insns
        .iter()
        .position(|i| i.offset == start)
        .ok_or_else(|| format!("no instruction at {start:#x}"))?;
    let mut result = Val::Unknown;
    for index in (first..).take(MAX_STEPS) {
        let Some(insn) = method.insns.get(index) else {
            return Err("ran past the end of the retained method".into());
        };
        if index > first && method.insns[index - 1].end != insn.offset {
            return Err(format!(
                "retained code is not contiguous at {:#x}",
                insn.offset
            ));
        }
        let ops = &insn.operands;
        let m = insn.mnemonic.as_str();
        let named = operand_regs(ops);
        let count = regs.len();
        let reg = |n: usize| -> Result<usize, String> {
            named
                .get(n)
                .copied()
                .filter(|r| *r < count)
                .ok_or_else(|| format!("bad register operand in {m} {ops}"))
        };
        match m {
            "nop" => {}
            "const/4" | "const/16" | "const" | "const/high16" => {
                let dst = reg(0)?;
                regs[dst] = literal_int(ops).map_or(Val::Unknown, Val::Int);
            }
            "const-string" | "const-string/jumbo" => {
                let dst = reg(0)?;
                regs[dst] = quoted_literal(ops).map_or(Val::Unknown, Val::Str);
            }
            "const-class" => regs[reg(0)?] = Val::Object,
            "sget-object" => {
                let dst = reg(0)?;
                regs[dst] = if ops.ends_with(":Ljava/lang/String;") {
                    Val::Unknown
                } else {
                    Val::Object
                };
            }
            "move-object" | "move-object/from16" | "move-object/16" => {
                let (dst, src) = (reg(0)?, reg(1)?);
                regs[dst] = regs[src].clone();
            }
            "move-result-object" | "move-result" => {
                let dst = reg(0)?;
                regs[dst] = std::mem::replace(&mut result, Val::Unknown);
            }
            "new-array" => {
                let (dst, size) = (reg(0)?, reg(1)?);
                regs[dst] = match regs[size] {
                    Val::Int(n) if (0..=MAX_ARRAY).contains(&n) => {
                        arrays.slots.push(vec![None; n as usize]);
                        arrays.poisoned.push(None);
                        Val::Arr(arrays.slots.len() - 1)
                    }
                    _ => Val::Unknown,
                };
            }
            "aput-object" => {
                let (value, array, idx) = (reg(0)?, reg(1)?, reg(2)?);
                match (&regs[array], &regs[idx]) {
                    (Val::Arr(id), Val::Int(i))
                        if (0..arrays.slots[*id].len() as i64).contains(i) =>
                    {
                        let (id, i) = (*id, *i as usize);
                        if arrays.slots[id][i].is_some() {
                            arrays.poison(id, &format!("has slot {i} written twice"));
                        }
                        if let Val::Arr(inner) = regs[value] {
                            arrays.poison(inner, "escapes into another array");
                        }
                        arrays.slots[id][i] = Some(regs[value].clone());
                    }
                    (Val::Arr(id), _) => {
                        let id = *id;
                        arrays.poison(id, "is written at an unknown index");
                    }
                    _ => arrays.poison_all("may be written through an unknown reference"),
                }
            }
            "packed-switch" => {
                let Some(targets) = switch.take() else {
                    return Err("switch inside the followed path".into());
                };
                targets.push((regs.clone(), case_targets(ops)?));
                return Ok(Outcome::Ended);
            }
            "return-object" => return Ok(Outcome::Returned(regs[reg(0)?].clone())),
            "return-void" => return Ok(Outcome::Returned(Val::Unknown)),
            "throw" => return Ok(Outcome::Ended),
            _ if m.starts_with("invoke-") => {
                let args: Vec<Val> = operand_regs(invoke_braces(ops))
                    .into_iter()
                    .map(|r| regs.get(r).cloned().ok_or("bad invoke register"))
                    .collect::<Result<_, _>>()?;
                let (class, name) = invoke_target(ops).ok_or("unparsed invoke target")?;
                if is_new_message_info(ops) {
                    let [_, info, objects] = <[Val; 3]>::try_from(args)
                        .map_err(|_| "newMessageInfo with an unexpected argument count")?;
                    return Ok(Outcome::Descriptor { info, objects });
                }
                let helpers: &Helpers = ctx.helpers;
                let helper = (m == "invoke-static" && depth < MAX_HELPER_DEPTH)
                    .then(|| helpers.find(&method.sha256, class, name, args.len()))
                    .flatten();
                match helper {
                    Some(helper) => {
                        // Static arguments occupy the callee's last registers.
                        let mut callee = vec![Val::Unknown; helper.registers.max(args.len())];
                        let base = callee.len() - args.len();
                        callee[base..].clone_from_slice(&args);
                        let used = format!("{class}->{name}");
                        if !ctx.used.contains(&used) {
                            ctx.used.push(used);
                        }
                        let start = first_offset(helper)?;
                        match follow(ctx, helper, start, callee, arrays, depth + 1, None)? {
                            Outcome::Returned(value) => result = value,
                            _ => return Err(format!("helper {class}->{name} did not return")),
                        }
                    }
                    None => {
                        // Code that is not followed may write any array it receives.
                        let call = format!("{class}->{name}");
                        let why = if m == "invoke-static" && depth < MAX_HELPER_DEPTH {
                            format!("was passed to {call}, whose body is not retained")
                        } else {
                            format!("was passed to {call}, which is not followed ({m})")
                        };
                        for arg in &args {
                            if let Val::Arr(id) = arg {
                                arrays.poison(*id, &why);
                            }
                        }
                        result = Val::Opaque(call);
                    }
                }
            }
            _ if m.starts_with("if-")
                || m.starts_with("goto")
                || m.starts_with("sparse-switch")
                || m.starts_with("fill-array")
                || m.starts_with("filled-new-array") =>
            {
                return Err(format!("{m} on the followed path is not modelled"));
            }
            _ if m.starts_with("aput") => {
                if let Ok(array) = reg(1)
                    && let Val::Arr(id) = regs[array]
                {
                    arrays.poison(id, &format!("is written by {m}"));
                }
            }
            _ if m.starts_with("iput") || m.starts_with("sput") => {
                // An array stored in a field can be written by code not followed.
                if let Ok(value) = reg(0)
                    && let Val::Arr(id) = regs[value]
                {
                    arrays.poison(id, &format!("escapes through {m}"));
                }
            }
            _ if m.starts_with("monitor-") || m == "check-cast" => {}
            // Every other instruction writes its first register.
            _ => {
                if let Ok(dst) = reg(0) {
                    regs[dst] = Val::Unknown;
                }
            }
        }
    }
    Err("instruction budget exhausted".into())
}

fn is_new_message_info(ops: &str) -> bool {
    invoke_target(ops)
        == Some((
            "Lcom/google/protobuf/GeneratedMessageLite;",
            "newMessageInfo",
        ))
}

/// A whole retained body starts at offset 0; an excerpt is not followed.
fn first_offset(method: &Method) -> Result<u32, String> {
    match method.insns.first() {
        Some(insn) if insn.offset == 0 => Ok(0),
        Some(_) => Err(format!(
            "{}->{} is retained only in part",
            method.class, method.name
        )),
        None => Err(format!(
            "{}->{} has no retained body",
            method.class, method.name
        )),
    }
}

/// Registers named in an operand list, in order: `v0, v2, v1`, `{v2}`, or a
/// range `{v2..v11}`.
fn operand_regs(ops: &str) -> Vec<usize> {
    let head = ops.split(['"', '#']).next().unwrap_or_default();
    let head = head.split(", L").next().unwrap_or(head);
    let mut out = Vec::new();
    for token in head.split([',', ' ', '{', '}']) {
        if let Some((a, b)) = token.split_once("..") {
            if let (Some(a), Some(b)) = (register(a), register(b))
                && a <= b
                && b - a < 256
            {
                out.extend(a..=b);
            }
        } else if let Some(r) = register(token) {
            out.push(r);
        }
    }
    out
}

fn register(token: &str) -> Option<usize> {
    token.strip_prefix('v')?.parse().ok()
}

fn invoke_braces(ops: &str) -> &str {
    ops.split_once('}').map_or("", |(braces, _)| braces)
}

/// `{v2, v0}, LX/Ayo;->A1P(VLL)` → (`LX/Ayo;`, `A1P`).
fn invoke_target(ops: &str) -> Option<(&str, &str)> {
    let (_, target) = ops.split_once("}, ")?;
    let (class, rest) = target.split_once("->")?;
    let name = rest.split('(').next()?;
    Some((class, name))
}

/// `v0, #0x72`; a signed annotation such as `#0xff (-1)` uses its decimal.
fn literal_int(ops: &str) -> Option<i64> {
    let (_, lit) = ops.split_once('#')?;
    if let Some((_, signed)) = lit.split_once('(') {
        return signed.trim_end_matches(')').parse().ok();
    }
    i64::from_str_radix(lit.trim().strip_prefix("0x")?, 16).ok()
}

/// `cases=[0:@0x1c, 1:@0x26, ...]` as byte offsets.
fn case_targets(ops: &str) -> Result<Vec<u32>, String> {
    let list = ops
        .split_once("cases=[")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
        .ok_or("unparsed packed-switch")?;
    list.split(", ")
        .filter(|case| !case.is_empty())
        .map(|case| {
            case.split_once(":@0x")
                .and_then(|(_, hex)| u32::from_str_radix(hex, 16).ok())
                .ok_or_else(|| format!("unparsed case {case}"))
        })
        .collect()
}

/// `xtask retained-proto-fields`: print every `dynamicMethod` in `path` whose
/// objects array and descriptor pair exactly; helpers are looked up under
/// `helpers` (default: the bundle directory's parent, `docs/protocol/apk`).
/// With `literal_order`, a class the exact path refuses is paired the way
/// `apk-proto-fields` pairs it (field-name literals in method order), under a
/// header that says so: that pairing is wrong whenever an unfollowed helper
/// also writes a name, which a matching count does not rule out.
pub fn run_fields(path: &Path, helpers: Option<&Path>, literal_order: bool) -> Result<(), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let root = match helpers {
        Some(root) => root.to_path_buf(),
        None => path
            .parent()
            .and_then(Path::parent)
            .ok_or("no helper directory; pass --helpers")?
            .to_path_buf(),
    };
    let helpers = Helpers::load(&root)?;
    let mut printed = 0usize;
    for method in parse_retained(&text) {
        if method.name != "dynamicMethod" {
            continue;
        }
        let reason = match descriptor_fields(&method, &helpers) {
            Ok((pairs, used)) => {
                println!("# {}  ({} fields)", method.class, pairs.len());
                for helper in used {
                    println!("#   names via retained helper {helper}");
                }
                print_pairs(&pairs);
                printed += 1;
                continue;
            }
            Err(reason) => reason,
        };
        let literals: Vec<String> = method
            .insns
            .iter()
            .filter_map(|insn| quoted_literal(&insn.operands))
            .collect();
        match (pair_proto_fields(&literals), literal_order) {
            (Ok(pairs), true) => {
                println!(
                    "# {}  ({} fields, LITERAL ORDER, unverified: {reason})",
                    method.class,
                    pairs.len()
                );
                print_pairs(&pairs);
                printed += 1;
            }
            (Ok(pairs), false) => eprintln!(
                "{}: {reason}; not guessing (--literal-order pairs {} names in method order, unverified)",
                method.class,
                pairs.len()
            ),
            (Err(literal), _) => {
                eprintln!(
                    "{}: {reason}; literal order: {literal}; not guessing",
                    method.class
                )
            }
        }
    }
    if printed == 0 {
        return Err(format!(
            "no protobuf descriptor decoded in {}",
            path.display()
        ));
    }
    Ok(())
}

fn print_pairs(pairs: &[(u32, String)]) {
    for (number, name) in pairs {
        println!("{number:>4}  {name}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENT: &str =
        include_str!("../../docs/protocol/apk/2.26.37.71-own-device-media/document-descriptor.txt");
    const BIT_FIELD: &str = include_str!(
        "../../docs/protocol/apk/2.26.37.71-adv-details-serialization/descriptor-bit-field.txt"
    );
    const FILE_SHA_HELPER: &str =
        include_str!("../../docs/protocol/apk/2.26.37.71-own-device-media/image-name-helper.txt");
    const CONVERSATION: &str =
        include_str!("../../docs/protocol/apk/2.26.37.71-token-import/conversation-descriptor.txt");
    const PRESENCE_ARRAY: &str =
        include_str!("../../docs/protocol/apk/2.26.37.71-token-import/presence-array-helper.txt");
    const ID_ARRAY: &str =
        include_str!("../../docs/protocol/apk/2.26.37.71-token-import/id-array-helper.txt");

    fn helpers(files: &[&str]) -> Helpers {
        let mut helpers = Helpers::default();
        for text in files {
            helpers.add(parse_retained(text));
        }
        helpers
    }

    fn entry(text: &str) -> Method {
        parse_retained(text)
            .into_iter()
            .find(|m| m.name == "dynamicMethod")
            .expect("retained dynamicMethod")
    }

    fn number(pairs: &[(u32, String)], name: &str) -> u32 {
        pairs
            .iter()
            .find(|(_, n)| n == name)
            .map(|(n, _)| *n)
            .unwrap()
    }

    #[test]
    fn document_names_written_by_a_retained_helper_are_placed_exactly() {
        let (pairs, used) =
            descriptor_fields(&entry(DOCUMENT), &helpers(&[BIT_FIELD, FILE_SHA_HELPER])).unwrap();
        assert_eq!(pairs.len(), 21);
        assert_eq!(used, ["LX/6al;->A1W", "LX/Ayp;->A1K"]);
        // Independent of this reader: the document tags the sender already
        // uses (transport/media.rs `Document`, traced against the APK), and
        // fileSha256, which only the helper writes (Ayp.A1K stores it at
        // objects[4]). Literal order loses it and shifts every later name.
        for (name, tag) in [
            ("url", 1),
            ("mimetype", 2),
            ("title", 3),
            ("fileSha256", 4),
            ("fileLength", 5),
            ("mediaKey", 7),
            ("fileName", 8),
            ("fileEncSha256", 9),
            ("caption", 20),
        ] {
            assert_eq!(number(&pairs, name), tag, "{name}");
        }
        let literals: Vec<String> = entry(DOCUMENT)
            .insns
            .iter()
            .filter_map(|i| quoted_literal(&i.operands))
            .collect();
        assert_eq!(
            pair_proto_fields(&literals).unwrap_err(),
            "decoded 21 fields against 20 names"
        );
    }

    #[test]
    fn conversation_reproduces_the_manual_token_import_pairing() {
        // docs/protocol/token-import.md paired these by hand from the same
        // descriptor and the two helper dumps.
        let (pairs, _) =
            descriptor_fields(&entry(CONVERSATION), &helpers(&[PRESENCE_ARRAY, ID_ARRAY])).unwrap();
        assert_eq!(pairs.len(), 59);
        for (name, tag) in [
            ("id", 1),
            ("messages", 2),
            ("tcToken", 21),
            ("tcTokenTimestamp", 22),
            ("contactPrimaryIdentityKey", 23),
            ("tcTokenSenderTimestamp", 28),
            ("identityVerification", 63),
        ] {
            assert_eq!(number(&pairs, name), tag, "{name}");
        }
    }

    #[test]
    fn a_missing_or_foreign_helper_is_named_and_refused() {
        let error = descriptor_fields(&entry(DOCUMENT), &helpers(&[BIT_FIELD])).unwrap_err();
        assert_eq!(
            error,
            "objects array was passed to LX/Ayp;->A1K, whose body is not retained"
        );
        // The same helper body from another APK build is not used.
        let foreign = FILE_SHA_HELPER.replace(
            "1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c",
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert!(descriptor_fields(&entry(DOCUMENT), &helpers(&[BIT_FIELD, &foreign])).is_err());
        // Two retained copies that disagree are treated as unknown.
        let altered = FILE_SHA_HELPER.replace("fileSha256_", "otherName_");
        assert!(
            descriptor_fields(
                &entry(DOCUMENT),
                &helpers(&[BIT_FIELD, FILE_SHA_HELPER, &altered])
            )
            .is_err()
        );
        let error = descriptor_fields(&entry(CONVERSATION), &helpers(&[ID_ARRAY])).unwrap_err();
        assert_eq!(
            error,
            "objects array comes from LX/Ayl;->A1b, whose body is not retained"
        );
    }

    #[test]
    fn unwritten_slots_overwrites_and_extra_call_sites_are_refused() {
        let full = helpers(&[BIT_FIELD, FILE_SHA_HELPER]);
        // Replace the write of objects[5] (fileLength_) with a nop.
        let gap = DOCUMENT.replace(
            "  005e..0062 op=4d aput-object v1, v2, v0\n",
            "  005e..0062 op=00 nop\n",
        );
        assert_ne!(gap, DOCUMENT);
        assert_eq!(
            descriptor_fields(&entry(&gap), &full).unwrap_err(),
            "objects[5] is never written"
        );
        // Write objects[6] twice.
        let twice = DOCUMENT.replace("  0068..006c op=4d aput-object v1, v2, v0\n",
            "  0068..006a op=4d aput-object v1, v2, v0\n  006a..006c op=4d aput-object v1, v2, v0\n");
        assert_ne!(twice, DOCUMENT);
        assert_eq!(
            descriptor_fields(&entry(&twice), &full).unwrap_err(),
            "objects array has slot 6 written twice"
        );
        // A second newMessageInfo call site.
        let call = "invoke-static {v0, v1, v2}, Lcom/google/protobuf/GeneratedMessageLite;->newMessageInfo(";
        let line = DOCUMENT.lines().find(|l| l.contains(call)).unwrap();
        let doubled = DOCUMENT.replacen(line, &format!("{line}\n{line}"), 1);
        assert_eq!(
            descriptor_fields(&entry(&doubled), &full).unwrap_err(),
            "2 newMessageInfo call sites"
        );
    }

    #[test]
    fn excerpts_gaps_and_unidentified_builds_are_not_followed() {
        let full = helpers(&[BIT_FIELD, FILE_SHA_HELPER]);
        // An elided instruction inside the followed path.
        let gap = DOCUMENT.replace("  0058..005c op=1a const-string v1, \"fileLength_\"\n", "");
        assert_ne!(gap, DOCUMENT);
        assert_eq!(
            descriptor_fields(&entry(&gap), &full).unwrap_err(),
            "retained code is not contiguous at 0x5c"
        );
        // A helper retained from offset 2 onwards is an excerpt.
        let excerpt = FILE_SHA_HELPER.replace("  0000..0002 op=12 const/4 v0, #0x3\n", "");
        assert_ne!(excerpt, FILE_SHA_HELPER);
        assert!(
            descriptor_fields(&entry(DOCUMENT), &helpers(&[BIT_FIELD, &excerpt]))
                .unwrap_err()
                .contains("retained only in part")
        );
        // Neither side may match on a missing build header.
        let anonymous = |text: &str| {
            text.lines()
                .filter(|l| !l.starts_with("# APK SHA-256: "))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(
            descriptor_fields(
                &entry(&anonymous(DOCUMENT)),
                &helpers(&[&anonymous(BIT_FIELD), &anonymous(FILE_SHA_HELPER)])
            )
            .is_err()
        );
    }

    #[test]
    fn operands_and_switches_parse() {
        assert_eq!(operand_regs("v0, v2, v1"), [0, 2, 1]);
        assert_eq!(operand_regs("{v2..v4}, LX/A;->b(LLL)V"), [2, 3, 4]);
        assert_eq!(operand_regs("v1, \"v9_\""), [1]);
        assert_eq!(operand_regs("v0, LX/BZc;->DEFAULT_INSTANCE:LX/BZc;"), [0]);
        assert_eq!(literal_int("v0, #0x72"), Some(0x72));
        assert_eq!(literal_int("v0, #0xff (-1)"), Some(-1));
        assert_eq!(
            case_targets("v2, payload@0x5dc, cases=[0:@0x1c, 2:@0x2e], default@0x12").unwrap(),
            [0x1c, 0x2e]
        );
        assert_eq!(
            invoke_target("{v2, v0}, LX/Ayo;->A1P(VLL)"),
            Some(("LX/Ayo;", "A1P"))
        );
    }
}
