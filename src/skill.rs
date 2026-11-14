use crate::config::{copy_builtin_skill, register_opencode_skill_source};
use crate::error::{MagpieError, Result};
use crate::registry::{BuiltinSkill, RemoteSkill, SkillEntry};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Confirm;
use std::path::PathBuf;

/// Install a skill. For built-in skills, copy into OpenCode's skills dir.
/// For remote (skill.sh) skills, delegate to `npx skills add` then register the source.
pub async fn install_skill(entry: &SkillEntry, yes: bool) -> Result<()> {
    match entry {
        SkillEntry::Builtin(skill) => install_builtin(skill, yes).await,
        SkillEntry::Remote(skill) => install_remote(skill, yes).await,
    }
}

async fn install_builtin(skill: &BuiltinSkill, yes: bool) -> Result<()> {
    show_skill_preview(skill)?;

    if !yes {
        let confirmed = Confirm::new(&format!("Do you want to install the skill \"{}\"?", skill.name))
            .with_default(true)
            .prompt()?;
        if !confirmed {
            println!("Installation cancelled.");
            return Ok(());
        }
    }

    let pb = ProgressBar::new_spinner();
    let style = ProgressStyle::default_spinner()
        .template("{spinner:.green} Installing {msg}...");
    pb.set_style(style);
    pb.set_message(skill.name.as_str());
    pb.enable_steady_tick(100);

    // Resolve the built-in skill directory from the compiled-in skills folder.
    let skills_dir = std::env::var("MAGPIE_BUILTIN_SKILLS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skills"));

    let source_dir = skills_dir.join(&skill.id);
    if !source_dir.exists() {
        return Err(MagpieError::SkillNotFound(format!(
            "Built-in skill {} not found at {:?}",
            skill.id, source_dir
        )));
    }
    let dest = copy_builtin_skill(&skill.id, &source_dir)?;
    pb.finish_with_message(format!("Installed {} into {:?}", skill.name, dest).as_str());

    println!(
        "✅ \"{}\" installed into OpenCode skills directory. Restart OpenCode/Magpie to use it.",
        skill.name
    );
    Ok(())
}

async fn install_remote(skill: &RemoteSkill, yes: bool) -> Result<()> {
    show_remote_skill_preview(skill)?;

    if !yes {
        let confirmed = Confirm::new(&format!("Do you want to install the skill \"{}\"?", skill.name))
            .with_default(true)
            .prompt()?;
        if !confirmed {
            println!("Installation cancelled.");
            return Ok(());
        }
    }

    let pb = ProgressBar::new_spinner();
    let style = ProgressStyle::default_spinner()
        .template("{spinner:.green} Installing {msg} via npx skills...");
    pb.set_style(style);
    pb.set_message(skill.name.as_str());
    pb.enable_steady_tick(100);

    // Delegate to the official `npx skills` CLI, targeting OpenCode.
    let output = std::process::Command::new("npx")
        .args([
            "skills", "add", &skill.full_id, "--skill", &skill.id, "-g", "-a", "opencode", "-y",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        pb.finish_with_message(format!("Failed to install {}", skill.name).as_str());
        return Err(MagpieError::Custom(format!(
            "npx skills add failed:\n{}",
            stderr
        )));
    }

    pb.finish_with_message(format!("Installed {}", skill.name).as_str());

    // Register the skill.sh source so OpenCode keeps discovering it.
    let source_url = "https://skills.sh";
    if let Err(e) = register_opencode_skill_source(source_url) {
        eprintln!("Warning: could not register skill source in OpenCode config: {e}");
    } else {
        println!("📡 Registered skill source {source_url} in OpenCode config.");
    }

    println!(
        "✅ \"{}\" installed and configured for OpenCode. Restart OpenCode/Magpie to use it.",
        skill.name
    );
    Ok(())
}

fn show_skill_preview(skill: &BuiltinSkill) -> Result<()> {
    println!("\n📦 Skill: {}", skill.name);
    println!("   ID: {}", skill.id);
    println!("   Source: magpie (built-in)");
    println!("   Description: {}", skill.description);
    if !skill.tags.is_empty() {
        println!("   Tags: {}", skill.tags.join(", "));
    }
    Ok(())
}

fn show_remote_skill_preview(skill: &RemoteSkill) -> Result<()> {
    println!("\n📦 Skill: {}", skill.name);
    println!("   ID: {}", skill.id);
    println!("   Source: skill.sh");
    if let Some(owner) = &skill.owner {
        println!("   Owner: {}", owner);
    }
    if let Some(installs) = skill.installs {
        println!("   Installs: {}", installs);
    }
    if !skill.description.is_empty() {
        println!("   Description: {}", skill.description);
    } else {
        println!("   Description: (not provided by skill.sh registry)");
    }
    if !skill.tags.is_empty() {
        println!("   Tags: {}", skill.tags.join(", "));
    }
    Ok(())
}

/// List installed skills in OpenCode's skills directory.
pub fn list_installed() -> Result<Vec<String>> {
    use crate::config::list_installed_skills;
    let paths = list_installed_skills()?;
    let mut names = Vec::new();
    for p in paths {
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            names.push(name.to_string());
        }
    }
    Ok(names)
}
