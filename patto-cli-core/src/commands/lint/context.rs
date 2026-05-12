use std::fs;
use std::path::Path;

use crate::diagnostic::Diagnostic;
use crate::lang::{self, Lang};
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::{CommandIndex, CommandKind};
use crate::project::{ProjectScan, find_text_location, find_value_location};

pub struct RuleContext<'a> {
    pub project: &'a ProjectScan,
    pub locale: Lang,
}

impl<'a> RuleContext<'a> {
    pub fn root(&self) -> &Path {
        &self.project.root
    }

    pub fn read_file(&self, relative_file: &str) -> Option<String> {
        fs::read_to_string(self.root().join(relative_file)).ok()
    }

    pub fn location_for_text(&self, relative_file: &str, value: &str) -> Option<(u32, u32)> {
        find_text_location(self.root(), relative_file, value)
    }

    pub fn location_for_value(
        &self,
        relative_file: &str,
        property: &str,
        value: &str,
    ) -> Option<(u32, u32)> {
        find_value_location(self.root(), relative_file, property, value)
    }

    pub fn diagnostic(
        &self,
        code: &'static str,
        severity: LintRuleSeverity,
        message_key: &str,
        args: &[(&str, &str)],
    ) -> Diagnostic {
        Diagnostic::new(
            severity.into(),
            code,
            lang::message(self.locale, message_key, args),
        )
        .with_hint(lang::text(self.locale, &format!("{code}.hint")))
    }

    pub fn command_marker(command: &CommandIndex) -> Option<(&'static str, &str)> {
        match command.kind {
            CommandKind::Command | CommandKind::Subcommand => {
                command.name.as_deref().map(|name| ("name", name))
            }
            CommandKind::SubcommandGroup => command
                .subcommand
                .as_deref()
                .map(|subcommand| ("subcommand", subcommand)),
            CommandKind::Unknown => None,
        }
    }

    pub fn attach_command_location(
        &self,
        mut diagnostic: Diagnostic,
        command: &CommandIndex,
    ) -> Diagnostic {
        if let Some((property, value)) = Self::command_marker(command) {
            if let Some((line, column)) =
                self.location_for_value(&command.metadata_file, property, value)
            {
                diagnostic = diagnostic.with_location(&command.metadata_file, line, column);
            } else {
                diagnostic = diagnostic.with_location(&command.metadata_file, 1, 1);
            }
        } else {
            diagnostic = diagnostic.with_location(&command.file, 1, 1);
        }

        diagnostic
    }
}
