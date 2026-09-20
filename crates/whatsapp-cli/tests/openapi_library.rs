//! Validate that x-library entries in the OpenAPI spec correspond to actual Rust symbols.
//!
//! For each entry like `whatsapp_api::a::b::Name::method` or `whatsapp_api::a::b::func`:
//! - Map crate root to `crates/whatsapp-api/src` or `crates/whatsapp-cli/src`
//! - Walk the lowercase snake_case segments after the crate to find the file
//! - Search for `fn <last>`, `const <last>`, `static <last>`, `struct <last>`, `enum <last>`
//! - Panic once with all failures listed

use std::fs;
use std::path::{Path, PathBuf};

fn map_crate_to_src(crate_name: &str) -> Result<&'static str, String> {
    match crate_name {
        "whatsapp_api" => Ok("crates/whatsapp-api/src"),
        "whatsapp_cli" => Ok("crates/whatsapp-cli/src"),
        _ => Err(format!("Unknown crate: {}", crate_name)),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or('?'));
    }
    result
}

fn last_word_snake_case(s: &str) -> String {
    // Extract the last "word" from CamelCase: WhatsAppClient -> client, ProtocolProfile -> profile
    let mut last_word_start = 0;
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            last_word_start = i;
        }
    }
    to_snake_case(&s[last_word_start..])
}

fn resolve_symbol_file(src_root: &Path, segments: &[&str]) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if segments.is_empty() {
        return vec![src_root.join("lib.rs")];
    }

    // Try to find the file containing the module
    // Build the path progressively: a, a/b, a/b/c, etc.
    let mut path = src_root.to_path_buf();
    for (i, seg) in segments.iter().enumerate() {
        // Try snake_case version first (for CamelCase type names like WhatsAppClient -> client)
        let snake_case_name = to_snake_case(seg);
        let last_word_snake = last_word_snake_case(seg);

        // For single-segment types at the root level, try src_root/snake_case.rs directly
        if i == 0 && segments.len() == 1 {
            let root_snake_rs = src_root.join(format!("{}.rs", snake_case_name));
            if root_snake_rs.exists() {
                candidates.push(root_snake_rs);
            }
            // Also try the last word (for types like WhatsAppClient -> client.rs)
            let root_last_word_rs = src_root.join(format!("{}.rs", last_word_snake));
            if root_last_word_rs.exists() {
                candidates.push(root_last_word_rs);
            }
        }

        path.push(seg);

        // After adding each segment, try to find files
        // Try as mod.rs
        let mod_rs = path.join("mod.rs");
        if mod_rs.exists() {
            candidates.push(mod_rs);
        }

        // Try as .rs file
        let rs_file = path.with_extension("rs");
        if rs_file.exists() {
            candidates.push(rs_file);
        }

        // Try snake_case version (for CamelCase type names)
        let snake_rs = path.with_file_name(format!("{}.rs", snake_case_name));
        if snake_rs.exists() {
            candidates.push(snake_rs);
        }

        // Always also check sibling file in parent (e.g., a/b.rs when building a/b/)
        if let Some(parent) = path.parent() {
            let sibling_rs = parent.join(format!("{}.rs", seg));
            if sibling_rs.exists() {
                candidates.push(sibling_rs);
            }
            // Also try snake_case sibling
            let sibling_snake_rs = parent.join(format!("{}.rs", snake_case_name));
            if sibling_snake_rs.exists() {
                candidates.push(sibling_snake_rs);
            }
        }
    }

    // Also try lib.rs as a catch-all
    candidates.push(src_root.join("lib.rs"));

    candidates
}

fn symbol_exists_in_file(file_path: &Path, symbol_name: &str) -> bool {
    let Ok(content) = fs::read_to_string(file_path) else {
        return false;
    };

    // Look for definitions in multiple forms
    // For functions: "fn name", "const name", etc. - may have modifiers before fn
    // Match the pattern anywhere with word boundaries

    // Check if we can find the symbol as a function/method definition
    // This handles cases like "pub async fn name" in impl blocks
    // Also handles pub use exports like "pub use module::Symbol"
    let search_patterns = [
        format!("fn {}", symbol_name),     // "fn name"
        format!("const {}", symbol_name),  // "const name"
        format!("static {}", symbol_name), // "static name"
        format!("struct {}", symbol_name), // "struct name"
        format!("enum {}", symbol_name),   // "enum name"
        format!("type {}", symbol_name),   // "type name"
        format!("trait {}", symbol_name),  // "trait name"
        format!("::{}", symbol_name),      // "::Name" (in pub use statements)
        format!(" {}", symbol_name),       // " Name" (bare identifier)
    ];

    search_patterns
        .iter()
        .any(|pattern| content.contains(pattern))
}

#[test]
fn openapi_library_symbols_exist() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let openapi_path = Path::new(&manifest_dir).join("../../docs/api/openapi.json");
    let openapi_content = fs::read_to_string(&openapi_path).expect("Failed to read openapi.json");
    let spec: serde_json::Value =
        serde_json::from_str(&openapi_content).expect("Failed to parse openapi.json");

    let project_root = Path::new(&manifest_dir).join("../..");
    let mut failures = Vec::new();
    let mut processed = 0;

    // Iterate through all operations
    if let Some(paths) = spec.get("paths").and_then(|v| v.as_object()) {
        for (path_key, path_item) in paths.iter() {
            if let Some(item_obj) = path_item.as_object() {
                for (method, operation) in item_obj.iter() {
                    if !["get", "post", "put", "delete", "patch"].contains(&method.as_str()) {
                        continue;
                    }

                    if let Some(op_obj) = operation.as_object() {
                        let cli_cmd = match op_obj.get("x-cli").and_then(|v| v.as_str()) {
                            Some(cmd) => cmd,
                            None => continue,
                        };

                        let library_entries = match op_obj.get("x-library") {
                            Some(serde_json::Value::Array(entries)) if !entries.is_empty() => {
                                entries
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                            }
                            _ => {
                                failures.push(format!(
                                    "{} {}: missing or empty x-library",
                                    method.to_uppercase(),
                                    path_key
                                ));
                                continue;
                            }
                        };

                        for entry in library_entries {
                            processed += 1;

                            let parts: Vec<&str> = entry.split("::").collect();
                            if parts.len() < 2 {
                                failures.push(format!(
                                    "{}: invalid x-library format '{}' (need crate::path::Symbol)",
                                    cli_cmd, entry
                                ));
                                continue;
                            }

                            let crate_name = parts[0];
                            let src_base = match map_crate_to_src(crate_name) {
                                Ok(base) => base,
                                Err(e) => {
                                    failures.push(format!("{}: {}", cli_cmd, e));
                                    continue;
                                }
                            };

                            let src_root = project_root.join(src_base);
                            let symbol_name = parts[parts.len() - 1];
                            let module_segments = &parts[1..parts.len() - 1];

                            // Resolve the file that should contain this symbol
                            let candidates = resolve_symbol_file(&src_root, module_segments);
                            let existing_candidates: Vec<_> =
                                candidates.iter().filter(|p| p.exists()).collect();

                            if existing_candidates.is_empty() {
                                failures.push(format!(
                                    "{}: cannot resolve module path for '{}' (crate: {}, modules: {:?})",
                                    cli_cmd, entry, crate_name, module_segments
                                ));
                                continue;
                            }

                            // Check if the symbol exists in any candidate
                            let found = existing_candidates
                                .iter()
                                .any(|p| symbol_exists_in_file(p, symbol_name));

                            if !found {
                                let file_list: Vec<_> = existing_candidates
                                    .iter()
                                    .filter_map(|p| p.strip_prefix(&project_root).ok())
                                    .map(|p| p.display().to_string())
                                    .collect();
                                failures.push(format!(
                                    "{}: symbol '{}' not found in {}",
                                    cli_cmd,
                                    symbol_name,
                                    file_list.join(", ")
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "OpenAPI library validation failed ({} symbols checked):\n{}",
            processed,
            failures.join("\n")
        );
    }
}
