//! typed snapshot of clap data that the renderer can turn into markdown.

use clap::builder::OsStr;

use crate::doc_registry::CommandDoc;

/// everything we need to print help for a command path.
#[derive(Debug, Clone)]
pub struct HelpPage {
    /// Name of the binary / application (e.g. "git")
    pub app_name: String,

    /// Application version (from clap metadata).
    pub version: Option<String>,

    /// Full command path (e.g. "commit main")
    pub path: String,

    /// One-line summary (from clap)
    pub summary: Option<String>,

    /// Optional longer description
    pub description: Option<String>,

    /// Usage string (from clap)
    pub usage: String,

    /// Positional arguments
    pub positionals: Vec<HelpArg>,

    /// Flags and options
    pub options: Vec<HelpOption>,

    /// Subcommands (for category-level help)
    pub subcommands: Vec<HelpSubcommand>,

    /// Examples
    pub examples: Vec<String>,

    /// Notes / tips / caveats
    pub notes: Vec<String>,
}

/// a flag/option with optional value and default info.
#[derive(Debug, Clone)]
pub struct HelpOption {
    pub short: Option<char>,
    pub long: Option<String>,
    pub value: Option<String>,
    pub description: String,
    pub default: String,
}

/// positional argument.
#[derive(Debug, Clone)]
pub struct HelpArg {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub multiple: bool,
}

/// child command for category-level help.
#[derive(Debug, Clone)]
pub struct HelpSubcommand {
    pub name: String,
    pub summary: Option<String>,
}

impl HelpPage {
    /// build a page straight from a `clap::Command`.
    pub fn from_clap(
        app_name: &str,
        version: Option<&str>,
        path: &str,
        cmd: &clap::Command,
    ) -> Self {
        let positionals = cmd
            .get_positionals()
            .map(|arg| HelpArg {
                name: arg.get_id().to_string(),
                description: arg.get_help().map(|s| s.to_string()),
                required: arg.is_required_set() || arg.get_num_args().unwrap().min_values() == 0,
                multiple: arg
                    .get_num_args()
                    .map(|n| n.min_values() != n.max_values() || 1 < n.min_values())
                    .unwrap_or_default(),
            })
            .collect();

        let options = cmd
            .get_arguments()
            .filter(|a| !a.is_positional())
            .map(|arg| HelpOption {
                short: arg.get_short(),
                long: arg.get_long().map(str::to_string),
                value: if arg.get_action().takes_values() {
                    arg.get_value_names()
                        .and_then(|v| v.first())
                        .map(|v| v.to_string())
                } else {
                    None
                },
                description: arg.get_help().unwrap_or_default().to_string(),
                default: arg
                    .get_default_values()
                    .join(&OsStr::from(", "))
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
            })
            .collect();

        let subcommands = cmd
            .get_subcommands()
            .map(|sc| HelpSubcommand {
                name: sc.get_name().to_string(),
                summary: sc.get_about().map(|s| s.to_string()),
            })
            .collect();

        Self {
            app_name: app_name.to_string(),
            version: version.map(|s| s.to_string()),
            path: path.to_string(),
            summary: cmd.get_about().map(|s| s.to_string()),
            description: cmd.get_long_about().map(|s| s.to_string()),
            usage: cmd.clone().render_usage().to_string(),
            positionals,
            options,
            subcommands,
            examples: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// merge data from a [`CommandDoc`], falling back to clap metadata when missing.
    pub fn with_docs(mut self, doc: Option<&CommandDoc>) -> Self {
        if let Some(doc) = doc {
            self.description = doc.description.clone().or(self.description);
            self.examples = doc.examples.clone();
            self.notes = doc.notes.clone();
        }
        self
    }
}
