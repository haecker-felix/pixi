use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use rattler_conda_types::NamedChannelOrUrl;
use serde::{Deserialize, Serialize};

/// The manifest format to create
#[derive(ValueEnum, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManifestFormat {
    Pixi,
    Pyproject,
    Mojoproject,
}

/// Source Control Management attributes for the workspace
#[derive(ValueEnum, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitAttributes {
    Github,
    Gitlab,
    Codeberg,
}

impl GitAttributes {
    pub fn template(&self) -> &'static str {
        match self {
            GitAttributes::Github | GitAttributes::Codeberg => {
                r#"# SCM syntax highlighting & preventing 3-way merges
pixi.lock merge=binary linguist-language=YAML linguist-generated=true
"#
            }
            GitAttributes::Gitlab => {
                r#"# GitLab syntax highlighting & preventing 3-way merges
pixi.lock merge=binary gitlab-language=yaml gitlab-generated=true
"#
            }
        }
    }
}

/// Creates a new workspace
///
/// This command is used to create a new workspace.
/// It prepares a manifest and some helpers for the user to start working.
///
/// As pixi can both work with `pixi.toml` and `pyproject.toml` files, the user can choose which one to use with `--format`.
///
/// You can import an existing conda environment file with the `--import` flag.
#[derive(Parser, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitOptions {
    /// Where to place the workspace (defaults to current path)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Channel to use in the workspace.
    #[arg(
        short,
        long = "channel",
        value_name = "CHANNEL",
        conflicts_with = "ENVIRONMENT_FILE"
    )]
    pub channels: Option<Vec<NamedChannelOrUrl>>,

    /// Platforms that the workspace supports.
    #[arg(short, long = "platform", id = "PLATFORM")]
    pub platforms: Vec<String>,

    /// Environment.yml file to bootstrap the workspace.
    #[arg(short = 'i', long = "import", id = "ENVIRONMENT_FILE")]
    pub env_file: Option<PathBuf>,

    /// The manifest format to create.
    #[arg(long, conflicts_with_all = ["ENVIRONMENT_FILE", "pyproject_toml"], ignore_case = true)]
    pub format: Option<ManifestFormat>,

    /// Create a pyproject.toml manifest instead of a pixi.toml manifest
    // BREAK (0.27.0): Remove this option from the cli in favor of the `format` option.
    #[arg(long, conflicts_with_all = ["ENVIRONMENT_FILE", "format"], alias = "pyproject", hide = true)]
    pub pyproject_toml: bool,

    /// Source Control Management used for this workspace
    #[arg(short = 's', long = "scm", ignore_case = true)]
    pub scm: Option<GitAttributes>,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            channels: None,
            platforms: Vec::new(),
            env_file: None,
            format: None,
            pyproject_toml: false,
            scm: None,
        }
    }
}
