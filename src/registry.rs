use crate::error::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The skill.sh API base URL.
const SKILL_SH_API: &str = "https://skills.sh/api";

/// A skill as returned by the skill.sh registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    pub id: String,
    pub full_id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub owner: Option<String>,
    pub tags: Vec<String>,
    pub installs: Option<u64>,
    pub install_url: Option<String>,
}

/// A skill from Magpie's built-in catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Unified skill view shown to the user (same list style as models).
#[derive(Debug, Clone)]
pub enum SkillEntry {
    Builtin(BuiltinSkill),
    Remote(RemoteSkill),
}

impl SkillEntry {
    pub fn id(&self) -> &str {
        match self {
            SkillEntry::Builtin(b) => &b.id,
            SkillEntry::Remote(r) => &r.id,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            SkillEntry::Builtin(b) => &b.name,
            SkillEntry::Remote(r) => &r.name,
        }
    }
    pub fn description(&self) -> &str {
        match self {
            SkillEntry::Builtin(b) => &b.description,
            SkillEntry::Remote(r) => &r.description,
        }
    }
    pub fn source(&self) -> &str {
        match self {
            SkillEntry::Builtin(_) => "magpie",
            SkillEntry::Remote(_) => "skill.sh",
        }
    }
    pub fn tags(&self) -> &[String] {
        match self {
            SkillEntry::Builtin(b) => &b.tags,
            SkillEntry::Remote(r) => &r.tags,
        }
    }
}

/// Fetches skills from the skill.sh API.
pub struct SkillRegistry {
    client: Client,
}

impl SkillRegistry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()?,
        })
    }

    /// Search skill.sh for skills matching a query.
    pub async fn search(&self, query: &str) -> Result<Vec<RemoteSkill>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let url = format!("{}/search?q={}&limit=50", SKILL_SH_API, urlencoding::encode(query));
        let resp: SkillShSearchResponse = self.client.get(&url).send().await?.error_for_status()?.json().await?;
        let mut out = Vec::new();
        for item in resp.skills {
            let parts: Vec<&str> = item.id.split('/').collect();
            let skill_id = item.skill_id.clone().unwrap_or_else(|| {
                parts.last().copied().unwrap_or(&item.id).to_string()
            });
            let skill_id_for_name = skill_id.clone();
            out.push(RemoteSkill {
                id: skill_id,
                full_id: item.id,
                name: item.name.unwrap_or(skill_id_for_name),
                description: item.description.unwrap_or_default(),
                source: item.source.clone().unwrap_or_default(),
                owner: item.owner.clone(),
                tags: item.tags.unwrap_or_default(),
                installs: item.installs,
                install_url: item.install_url.clone(),
            });
        }
        Ok(out)
    }

    /// Look up a specific skill by id via skill.sh search.
    pub async fn find(&self, id: &str) -> Result<Option<RemoteSkill>> {
        let results = self.search(id).await?;
        Ok(results.into_iter().find(|s| s.id == id))
    }
}

#[derive(Debug, Deserialize)]
struct SkillShSearchResponse {
    skills: Vec<SkillShSkillItem>,
}

#[derive(Debug, Deserialize)]
struct SkillShSkillItem {
    id: String,
    source: Option<String>,
    skill_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    owner: Option<String>,
    tags: Option<Vec<String>>,
    installs: Option<u64>,
    install_url: Option<String>,
}

/// Magpie's built-in marketing skills catalog.
pub fn builtin_skills() -> Vec<BuiltinSkill> {
    vec![
        BuiltinSkill {
            id: "launch-email".to_string(),
            name: "Launch Email".to_string(),
            description: "Write compelling product launch emails with subject lines, preheaders, and CTAs optimised for open and click-through rates.".to_string(),
            tags: vec!["email".into(), "launch".into(), "copywriting".into()],
        },
        BuiltinSkill {
            id: "landing-page-copy".to_string(),
            name: "Landing Page Copy".to_string(),
            description: "High-converting landing page structure: hero, problem/agitation, solution, social proof, pricing, and FAQ sections.".to_string(),
            tags: vec!["landing-page".into(), "conversion".into(), "copywriting".into()],
        },
        BuiltinSkill {
            id: "ad-copy".to_string(),
            name: "Ad Copy".to_string(),
            description: "Short-form paid-social and search ad headlines + descriptions tuned to platform character limits and ROAS goals.".to_string(),
            tags: vec!["ads".into(), "social".into(), "ppc".into(), "copywriting".into()],
        },
        BuiltinSkill {
            id: "seo-blog-post".to_string(),
            name: "SEO Blog Post".to_string(),
            description: "Search-optimised blog posts with keyword placement, H1/H2 structure, meta title/description, and internal-linking plan.".to_string(),
            tags: vec!["seo".into(), "blog".into(), "content".into()],
        },
        BuiltinSkill {
            id: "brand-voice".to_string(),
            name: "Brand Voice".to_string(),
            description: "Define and enforce brand voice, tone, do/don't wording, and forbidden phrases across all marketing assets.".to_string(),
            tags: vec!["brand".into(), "voice".into(), "guidelines".into()],
        },
        BuiltinSkill {
            id: "print".to_string(),
            name: "Print".to_string(),
            description: "Print documents to PDF or Word in A4 and US Letter sizes. No external programs required.".to_string(),
            tags: vec!["print".into(), "pdf".into(), "word".into(), "docx".into(), "a4".into(), "letter".into()],
        },
        BuiltinSkill {
            id: "crawl".to_string(),
            name: "Web Crawler".to_string(),
            description: "Crawl websites and export structured data (CSV/JSON) ready for CRM import. Extracts title, description, contacts, links, and metadata.".to_string(),
            tags: vec!["crawl".into(), "web".into(), "crm".into(), "csv".into(), "json".into(), "extract".into(), "leads".into()],
        },
        BuiltinSkill {
            id: "web-search".to_string(),
            name: "Web Search".to_string(),
            description: "Search the web and return structured results (title, URL, snippet) that can be fed into the crawler skill for CRM extraction.".to_string(),
            tags: vec!["search".into(), "web".into(), "google".into(), "duckduckgo".into(), "query".into(), "results".into()],
        },
        BuiltinSkill {
            id: "brand-analysis".to_string(),
            name: "Brand Analysis".to_string(),
            description: "Analyze company websites with a vision model to extract logos, fonts, colors, and brand identity for marketing use.".to_string(),
            tags: vec!["brand".into(), "vision".into(), "website".into(), "logo".into(), "fonts".into(), "colors".into()],
        },
        BuiltinSkill {
            id: "rag".to_string(),
            name: "RAG".to_string(),
            description: "Retrieval-Augmented Generation for company-specific knowledge. Ingest company documents and use them to refine marketing outputs with accurate, on-brand context.".to_string(),
            tags: vec!["rag".into(), "retrieval".into(), "knowledge".into(), "company".into(), "context".into()],
        },
    ]
}

/// List all skills: built-in first, then skill.sh results (deduplicated).
pub async fn list_skills(
    registry: &SkillRegistry,
    source: Option<&str>,
    query: Option<&str>,
) -> Result<Vec<SkillEntry>> {
    let mut entries: Vec<SkillEntry> = Vec::new();
    let mut seen_ids: HashMap<String, bool> = HashMap::new();

    // Built-in catalog (always shown unless filtered out).
    if source.is_none() || source == Some("magpie") {
        for skill in builtin_skills() {
            if let Some(q) = query {
                let hay = format!(
                    "{} {} {} {}",
                    skill.id, skill.name, skill.description, skill.tags.join(" ")
                )
                .to_lowercase();
                if !hay.contains(&q.to_lowercase()) {
                    continue;
                }
            }
            entries.push(SkillEntry::Builtin(skill.clone()));
            seen_ids.insert(skill.id, true);
        }
    }

    // skill.sh remote source.
    if source.is_none() || source == Some("skill.sh") {
        let q = query.unwrap_or("");
        let remotes = registry.search(q).await?;
        for remote in remotes {
            if seen_ids.contains_key(&remote.id) {
                continue;
            }
            entries.push(SkillEntry::Remote(remote.clone()));
            seen_ids.insert(remote.id, true);
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_skills_count() {
        assert_eq!(builtin_skills().len(), 10);
    }
}
