use itertools::Itertools;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct ClippyConfiguration {
    pub name: String,
    pub default: String,
    pub lints: &'static [&'static str],
    pub doc: &'static str,
    pub deprecation_reason: Option<&'static str>,
}

impl fmt::Display for ClippyConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "- `{}`: {}", self.name, self.doc)?;
        if !self.default.is_empty() {
            write!(f, "\n\n   (default: `{}`)", self.default)?;
        }
        Ok(())
    }
}

impl ClippyConfiguration {
    pub fn to_markdown_paragraph(&self, lint_groups: &HashMap<String, &str>) -> String {
        format!(
            "## `{}`\n{}\n\n**Default Value:** `{}`\n\n---\n**Affected lints:**\n{}\n\n",
            self.name,
            self.doc.lines().map(|x| x.strip_prefix(' ').unwrap_or(x)).join("\n"),
            self.default,
            self.lints.iter().format_with("\n", |name, f| {
                let group = lint_groups.get(*name).copied().unwrap_or("unknown");
                f(&format_args!(
                    "* [`{name}`](https://rust-lang.github.io/rust-clippy/master/index.html#{name}) ({group})"
                ))
            }),
        )
    }

    pub fn to_toml_paragraph(&self, lint_groups: &HashMap<String, &str>) -> String {
        let mut result = String::new();

        // Write the full doc string as comments
        for line in self.doc.lines() {
            let line = line.strip_prefix(' ').unwrap_or(line);
            if line.is_empty() {
                result.push_str("#\n");
            } else {
                result.push_str(&format!("# {line}\n"));
            }
        }

        // Write affected lints with their groups
        if self.lints.len() > 10 {
            // For configs affecting many lints (like msrv), list unique groups only
            let unique_groups: Vec<&str> = self
                .lints
                .iter()
                .filter_map(|name| lint_groups.get(*name).copied())
                .sorted_unstable()
                .dedup()
                .collect();
            result.push_str(&format!("# Affects lints in groups: {}\n", unique_groups.join(", ")));
        } else {
            for name in self.lints {
                let group = lint_groups.get(*name).copied().unwrap_or("unknown");
                result.push_str(&format!("# Affects: {name} ({group})\n"));
            }
        }

        // Write the config key = value
        // If the default is not a valid TOML value (e.g. "current version",
        // "target_pointer_width * 2"), comment it out.
        let is_valid_toml = format!("x = {}", self.default).parse::<toml::Table>().is_ok();
        if is_valid_toml {
            result.push_str(&format!("{} = {}\n", self.name, self.default));
        } else {
            result.push_str(&format!("# {} = <{}>\n", self.name, self.default));
        }

        result
    }

    pub fn to_markdown_link(&self) -> String {
        const BOOK_CONFIGS_PATH: &str = "https://doc.rust-lang.org/clippy/lint_configuration.html";
        format!("[`{}`]: {BOOK_CONFIGS_PATH}#{}", self.name, self.name)
    }
}
