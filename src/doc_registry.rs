//! stash extra text for commands and guides keyed by their full path.
//!
//! keys use dot separators for subcommands (e.g. `"git.commit.amend"`). an empty key means "the
//! program itself".

use std::collections::HashMap;

/// extra help metadata layered on top of clap output.
#[derive(Debug, Clone)]
pub struct CommandDoc {
    /// short blurb shown under the header.
    pub description: Option<String>,

    /// sample invocations printed as a numbered list.
    pub examples: Vec<String>,

    /// quick tips / caveats shown as bullets.
    pub notes: Vec<String>,
}

impl CommandDoc {
    /// convenience constructor; empty descriptions are treated as `None`.
    pub fn new<D, E, N>(description: D, examples: E, notes: N) -> Self
    where
        D: Into<String>,
        E: IntoIterator,
        E::Item: Into<String>,
        N: IntoIterator,
        N::Item: Into<String>,
    {
        Self {
            description: Some(description.into()).filter(|s| !s.is_empty()),
            examples: examples.into_iter().map(Into::into).collect(),
            notes: notes.into_iter().map(Into::into).collect(),
        }
    }
}

/// holds every command doc and guide for the current program session.
#[derive(Default)]
pub struct DocRegistry {
    commands: HashMap<String, CommandDoc>,
    guides: HashMap<String, String>,
}

impl DocRegistry {
    /// start fresh.
    pub fn new() -> Self {
        Self::default()
    }

    /// attach metadata to a command path (dot-separated, `""` for the main program).
    pub fn register_command<K: Into<String>>(&mut self, key: K, doc: CommandDoc) {
        self.commands.insert(key.into(), doc);
    }

    /// add a free-form markdown guide that can be opened via `guide <path>`.
    pub fn register_guide<K, C>(&mut self, key: K, content: C)
    where
        K: Into<String>,
        C: Into<String>,
    {
        self.guides.insert(key.into(), content.into());
    }

    /// fetch docs for a command, if any.
    pub fn command(&self, key: &str) -> Option<&CommandDoc> {
        self.commands.get(key)
    }

    /// fetch markdown for a guide, if any.
    pub fn guide(&self, key: &str) -> Option<&str> {
        self.guides.get(key).map(|s| s.as_str())
    }
}
