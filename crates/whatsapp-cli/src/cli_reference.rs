use clap::{Arg, Command};

#[allow(dead_code)]
pub fn render(cmd: &Command) -> String {
    let mut output = String::new();
    output.push_str("# whatsapp-cli reference\n\n");
    output.push_str(
        "Generated from the command definitions by the `cli_reference_is_current` test; \
         regenerate with `UPDATE_CLI_DOCS=1 cargo test -p whatsapp-cli --features experimental-transport cli_reference`. \
         Every command here also has an operation in docs/api/openapi.json (`x-cli`).\n\n",
    );

    render_global_options(&mut output, cmd);
    output.push('\n');

    let path = Vec::new();
    render_commands(&mut output, cmd, path);

    output
}

fn is_global_arg(arg: &Arg) -> bool {
    arg.is_global_set()
}

#[allow(dead_code)]
fn render_global_options(output: &mut String, cmd: &Command) {
    output.push_str("## Global options\n\n");
    output.push_str("| Flag | Value | Required | Default | Env | Values | Help |\n");
    output.push_str("|------|-------|----------|---------|-----|--------|------|\n");

    for arg in cmd.get_arguments() {
        if !is_global_arg(arg) {
            continue;
        }
        render_arg_row(output, arg);
    }
    output.push('\n');
}

#[allow(dead_code)]
fn render_arg_row(output: &mut String, arg: &Arg) {
    let flag = format_flag(arg);
    let value = format_value(arg);
    let required = if arg.is_required_set() { "yes" } else { "no" };
    let default = format_default(arg);
    let env = format_env(arg);
    let values = format_values(arg);
    let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();

    output.push_str(&format!(
        "| {} | {} | {} | {} | {} | {} | {} |\n",
        flag, value, required, default, env, values, help
    ));
}

#[allow(dead_code)]
fn format_flag(arg: &Arg) -> String {
    let mut result = String::new();
    if let Some(short) = arg.get_short() {
        result.push('-');
        result.push(short);
    }
    if let Some(long) = arg.get_long() {
        if !result.is_empty() {
            result.push_str(", ");
        }
        result.push_str("--");
        result.push_str(long);
    }
    result
}

#[allow(dead_code)]
fn format_value(arg: &Arg) -> String {
    if let Some(names) = arg.get_value_names() {
        names
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    } else if !arg.get_action().takes_values() {
        String::new()
    } else {
        "VALUE".to_string()
    }
}

#[allow(dead_code)]
fn format_default(arg: &Arg) -> String {
    let defaults = arg.get_default_values();
    if !defaults.is_empty() {
        defaults
            .iter()
            .map(|d| format!("`{}`", d.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    }
}

#[allow(dead_code)]
fn format_env(arg: &Arg) -> String {
    if let Some(env) = arg.get_env() {
        format!("`{}`", env.to_string_lossy())
    } else {
        String::new()
    }
}

#[allow(dead_code)]
fn format_values(arg: &Arg) -> String {
    let values = arg.get_possible_values();
    if !values.is_empty() {
        values
            .iter()
            .map(|v| format!("`{}`", v.get_name()))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    }
}

#[allow(dead_code)]
fn render_commands(output: &mut String, cmd: &Command, path: Vec<String>) {
    for subcmd in cmd.get_subcommands() {
        if subcmd.get_name() == "help" {
            continue;
        }

        let mut current_path = path.clone();
        current_path.push(subcmd.get_name().to_string());
        let path_str = current_path.join(" ");

        output.push_str(&format!("## whatsapp-cli {}\n\n", path_str));

        if let Some(about) = subcmd.get_about() {
            output.push_str(&about.to_string());
            output.push_str("\n\n");
        }

        if let Some(long_about) = subcmd.get_long_about() {
            output.push_str(&long_about.to_string());
            output.push_str("\n\n");
        }

        output.push_str(&format!(
            "```\nUsage: whatsapp-cli {} [OPTIONS]...\n```\n\n",
            path_str
        ));

        let local_args: Vec<_> = subcmd
            .get_arguments()
            .filter(|arg| {
                !arg.is_hide_set() && arg.get_id().as_str() != "help" && !is_global_arg(arg)
            })
            .collect();

        if !local_args.is_empty() {
            output.push_str("| Flag | Value | Required | Default | Env | Values | Help |\n");
            output.push_str("|------|-------|----------|---------|-----|--------|------|\n");
            for arg in local_args {
                render_arg_row(output, arg);
            }
            output.push('\n');
        }

        render_commands(output, subcmd, current_path);
    }
}

#[cfg(all(test, feature = "experimental-transport"))]
mod tests {
    use super::*;
    use crate::Cli;
    use clap::CommandFactory;

    #[test]
    fn cli_reference_is_current() {
        let root_cmd = Cli::command();
        let generated = render(&root_cmd);

        let docs_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/cli.md");

        if std::env::var("UPDATE_CLI_DOCS").is_ok() {
            std::fs::write(docs_path, &generated).expect("failed to write docs/cli.md");
            let leaf_count = count_leaf_commands(&root_cmd);
            let openapi_count = count_openapi_commands();
            eprintln!(
                "Leaf commands: {}, OpenAPI x-cli entries: {}",
                leaf_count, openapi_count
            );
            assert_eq!(
                leaf_count, openapi_count,
                "leaf command count must match OpenAPI x-cli count"
            );
            return;
        }

        let existing = std::fs::read_to_string(docs_path).unwrap_or_else(|_| String::new());

        if generated != existing {
            panic!(
                "docs/cli.md is stale; run UPDATE_CLI_DOCS=1 cargo test -p whatsapp-cli --features experimental-transport cli_reference"
            );
        }

        let leaf_count = count_leaf_commands(&root_cmd);
        let openapi_count = count_openapi_commands();
        assert_eq!(
            leaf_count, openapi_count,
            "leaf command count ({}) must match OpenAPI x-cli count ({})",
            leaf_count, openapi_count
        );
    }

    fn count_leaf_commands(cmd: &Command) -> usize {
        let mut count = 0;
        count_leaf_commands_recursive(cmd, &mut count);
        count
    }

    fn count_leaf_commands_recursive(cmd: &Command, count: &mut usize) {
        let subcommands: Vec<_> = cmd
            .get_subcommands()
            .filter(|s| s.get_name() != "help")
            .collect();

        if subcommands.is_empty() {
            *count += 1;
        } else {
            for subcmd in subcommands {
                count_leaf_commands_recursive(subcmd, count);
            }
        }
    }

    fn count_openapi_commands() -> usize {
        let openapi_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/api/openapi.json");
        let content = std::fs::read_to_string(openapi_path).expect("failed to read openapi.json");
        let doc: serde_json::Value =
            serde_json::from_str(&content).expect("failed to parse openapi.json");

        let mut cli_set = std::collections::HashSet::new();

        if let Some(paths) = doc.get("paths").and_then(|p| p.as_object()) {
            for path_obj in paths.values() {
                if let Some(path_map) = path_obj.as_object() {
                    for method_obj in path_map.values() {
                        if let Some(method_map) = method_obj.as_object()
                            && let Some(extensions) = method_map.get("x-cli")
                            && let Some(cli_str) = extensions.as_str()
                        {
                            cli_set.insert(cli_str.to_string());
                        }
                    }
                }
            }
        }

        cli_set.len()
    }
}
