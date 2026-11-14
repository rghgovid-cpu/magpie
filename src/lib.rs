//! Magpie — a marketing-focused, open-source OpenCode alternative in Rust.
//!
//! Features:
//! - Agentic search & writing (marketing-focused)
//! - Skill marketplace (skill.sh) listed like models
//! - Skill install with confirmation + auto-configure
//! - PDF / Word printing (A4 & US Letter)

pub mod agent;
pub mod commands;
pub mod config;
pub mod error;
pub mod model;
pub mod print;
pub mod registry;
pub mod skill;

pub use error::{MagpieError, Result};
