//! helpers for wiring clap commands into nice terminal help output.

use clap::{Args, Command};
use termimad::crossterm::style::Stylize;

use crate::{DocRegistry, HelpPage, HelpTheme};

/// args for your help command.
#[derive(Args, Debug, Clone)]
pub struct HelpArgs {
    /// Command to print information about
    pub topic: Vec<String>,
}

/// resolved help target (a command path, a guide, or the program root).
pub enum HelpTarget<'a> {
    Command { path: String, cmd: &'a Command },
    Guide { path: String },
    Program { cmd: &'a Command },
}

/// walk the clap tree to find the thing the user asked for.
///
/// `guide` is special-cased so `app help foo bar guide` opens the `foo.bar` guide instead of looking
/// for a `guide` subcommand.
pub fn resolve_help<'a>(root: &'a Command, topic: &[String]) -> anyhow::Result<HelpTarget<'a>> {
    let mut cmd = root;
    let mut path = Vec::new();

    if topic.is_empty() {
        return Ok(HelpTarget::Program { cmd });
    }

    for segment in topic {
        if segment == "guide" {
            return Ok(HelpTarget::Guide {
                path: path.join("."),
            });
        }

        cmd = cmd
            .get_subcommands()
            .find(|c| c.get_name() == segment)
            .ok_or_else(|| anyhow::anyhow!("Unknown help topic"))?;

        path.push(segment.clone());
    }

    Ok(HelpTarget::Command {
        path: path.join("."),
        cmd,
    })
}

/// handle a `help <topic>` command without custom docs.
pub fn help_command(
    app_name: &str,
    app_version: Option<&str>,
    root: &Command,
    theme: &HelpTheme,
    args: &HelpArgs,
) -> anyhow::Result<()> {
    run_help_topic(app_name, app_version, root, &DocRegistry::new(), theme, &args.topic)
}

/// like [`help_command`] but with an attached [`DocRegistry`].
pub fn help_command_docs(
    app_name: &str,
    app_version: Option<&str>,
    root: &Command,
    docs: &DocRegistry,
    theme: &HelpTheme,
    args: &HelpArgs,
) -> anyhow::Result<()> {
    run_help_topic(app_name, app_version, root, docs, theme, &args.topic)
}

/// show program help when no command was provided.
pub fn help_command_program(
    app_name: &str,
    app_version: Option<&str>,
    root: &Command,
    theme: &HelpTheme,
) -> anyhow::Result<()> {
    run_help_topic(app_name, app_version, root, &DocRegistry::new(), theme, &Vec::new())
}

/// program help with attached docs.
pub fn help_command_program_docs(
    app_name: &str,
    app_version: Option<&str>,
    root: &Command,
    docs: &DocRegistry,
    theme: &HelpTheme,
) -> anyhow::Result<()> {
    run_help_topic(app_name, app_version, root, docs, theme, &Vec::new())
}

/// shared execution path for all help entrypoints.
pub fn run_help_topic(
    app_name: &str,
    app_version: Option<&str>,
    root: &Command,
    docs: &DocRegistry,
    theme: &HelpTheme,
    topic: &[String],
) -> anyhow::Result<()> {
    let target = resolve_help(root, topic)?;

    match target {
        HelpTarget::Command { path, cmd } => {
            let page = HelpPage::from_clap(
                std::env::current_exe()
                    .expect("Failed to get executable path")
                    .file_name()
                    .expect("Failed to get executable name")
                    .to_str()
                    .unwrap(),
                app_version,
                &path,
                cmd,
            )
            .with_docs(docs.command(&path));

            crate::render_command_help(theme, &page);
        }
        HelpTarget::Guide { path } => {
            let path = if path.is_empty() {
                app_name.to_string()
            } else {
                path
            };
            if let Some(guide) = docs.guide(&path) {
                let (_, rows) = termimad::crossterm::terminal::size().unwrap();
                if guide.lines().count() > rows.into() {
                    crate::run_scrollable_help(theme, app_name, guide.to_string())?;
                } else {
                    println!("{}", theme.skin.term_text(&guide));
                }
            } else {
                println!(
                    "Guide for {} was {}.",
                    &path.with(theme.accent).bold(),
                    "not found".red().bold()
                )
            }
        }
        HelpTarget::Program { cmd } => {
            let page = HelpPage::from_clap(app_name, app_version, "", cmd)
                .with_docs(docs.command(""));

            crate::render_command_help(theme, &page);
        }
    }

    Ok(())
}
