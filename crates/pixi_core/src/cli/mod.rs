use clap::Parser;
use miette::Diagnostic;
use pixi_consts::consts;
use thiserror::Error;

pub mod cli_config;
pub mod has_specs;

/// Configuration for lock file usage, used by LockFileUpdateConfig
#[derive(Parser, Debug, Default, Clone)]
pub struct LockFileUsageConfig {
    /// Install the environment as defined in the lockfile, doesn't update
    /// lockfile if it isn't up-to-date with the manifest file.
    #[clap(long, env = "PIXI_FROZEN", help_heading = consts::CLAP_UPDATE_OPTIONS)]
    pub frozen: bool,
    /// Check if lockfile is up-to-date before installing the environment,
    /// aborts when lockfile isn't up-to-date with the manifest file.
    #[clap(long, env = "PIXI_LOCKED", help_heading = consts::CLAP_UPDATE_OPTIONS)]
    pub locked: bool,
}

impl LockFileUsageConfig {
    /// Validate that the configuration is valid
    pub fn validate(&self) -> Result<(), LockFileUsageError> {
        if self.frozen && self.locked {
            return Err(LockFileUsageError::FrozenAndLocked);
        }
        Ok(())
    }
}

impl TryFrom<LockFileUsageConfig> for crate::environment::LockFileUsage {
    type Error = LockFileUsageError;

    fn try_from(value: LockFileUsageConfig) -> Result<Self, LockFileUsageError> {
        value.validate()?;
        if value.frozen {
            Ok(Self::Frozen)
        } else if value.locked {
            Ok(Self::Locked)
        } else {
            Ok(Self::Update)
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum LockFileUsageError {
    #[error("the argument '--locked' cannot be used together with '--frozen'")]
    FrozenAndLocked,
}
