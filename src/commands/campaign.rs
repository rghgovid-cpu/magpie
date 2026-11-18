use crate::config::{
    active_campaign, campaign_crm_path, campaign_exists, campaign_leads_path, campaign_path,
    campaign_manifest_path, list_campaigns, set_active_campaign,
};
use crate::error::{MagpieError, Result};
use clap::Args;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Manage marketing campaigns: create, research, crawl, analyze, export to CRM.
#[derive(Subcommand, Debug)]
pub enum CampaignCommand {
    /// Create a new campaign with industry and location.
    Create(CreateArgs),
    /// List all campaigns.
    List,
    /// Select/set the active campaign.
    Select(SelectArgs),
    /// Show campaign status and progress.
    Status(StatusArgs),
    /// Research businesses via Google Maps (Serper) for this campaign's industry/location.
    Research(ResearchArgs),
    /// Crawl all websites from discovered leads.
    Crawl(CrawlArgs),
    /// Run brand vision analysis and team extraction on crawled sites.
    Analyze(AnalyzeArgs),
    /// Find LinkedIn company pages via Google search.
    Linkedin(LinkedinArgs),
    /// Export all campaign data to a CRM-ready JSON file.
    Export(ExportArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Campaign name (used as folder name)
    #[arg(short = 'n', long = "name")]
    pub name: String,
    /// Industry to research (e.g. "dentist", "plumber", "marketing agency")
    #[arg(long)]
    pub industry: String,
    /// Location to search (e.g. "Chicago", "London", "New York")
    #[arg(long)]
    pub location: String,
    /// Set this campaign as active after creating
    #[arg(short = 's', long = "select", default_value = "true")]
    pub select: bool,
}

#[derive(Args, Debug)]
pub struct SelectArgs {
    /// Campaign name to activate
    pub name: String,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Campaign name (defaults to active campaign)
    #[arg(short = 'c', long = "campaign")]
    pub campaign: Option<String>,
}

#[derive(Args, Debug)]
pub struct ResearchArgs {
    /// Campaign name (defaults to active campaign)
    #[arg(short = 'c', long = "campaign")]
    pub campaign: Option<String>,
    /// Override industry for this research run
    #[arg(long)]
    pub industry: Option<String>,
    /// Override location for this research run
    #[arg(long)]
    pub location: Option<String>,
    /// Maximum number of leads to find
    #[arg(long, default_value = "20")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct CrawlArgs {
    /// Campaign name (defaults to active campaign)
    #[arg(short = 'c', long = "campaign")]
    pub campaign: Option<String>,
    /// Maximum number of websites to crawl
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Output format: csv or json
    #[arg(long, default_value = "json")]
    pub format: String,
}

#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Campaign name (defaults to active campaign)
    #[arg(short = 'c', long = "campaign")]
    pub campaign: Option<String>,
    /// Run vision brand analysis on crawled websites
    #[arg(long, default_value = "true")]
    pub brand: bool,
    /// Extract team/employee info from websites
    #[arg(long, default_value = "true")]
    pub team: bool,
}

#[derive(Args, Debug)]
pub struct LinkedinArgs {
    /// Campaign name (defaults to active campaign)
    #[arg(short = 'c', long = "campaign")]
    pub campaign: Option<String>,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Campaign name (defaults to active campaign)
    #[arg(short = 'c', long = "campaign")]
    pub campaign: Option<String>,
    /// Output file path (default: campaign/crm-export.json)
    #[arg(short = 'o', long = "out")]
    pub out: Option<PathBuf>,
}

/// A discovered business lead from Google Maps research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub rating: Option<f64>,
    pub reviews: Option<u32>,
    pub category: Option<String>,
    pub source: String, // "serper-maps"
    pub discovered_at: String,
}

/// Campaign manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignManifest {
    pub name: String,
    pub industry: String,
    pub location: String,
    pub created_at: String,
    pub status: String, // "created", "researching", "crawling", "analyzing", "exported"
    pub lead_count: u32,
    pub crawled_count: u32,
    pub analyzed_count: u32,
}

pub fn handle(cmd: CampaignCommand) -> Result<()> {
    match cmd {
        CampaignCommand::Create(args) => handle_create(args),
        CampaignCommand::List => handle_list(),
        CampaignCommand::Select(args) => handle_select(&args.name),
        CampaignCommand::Status(args) => handle_status(args.campaign.as_deref()),
        CampaignCommand::Research(args) => handle_research(args),
        CampaignCommand::Crawl(args) => handle_crawl(args),
        CampaignCommand::Analyze(args) => handle_analyze(args),
        CampaignCommand::Linkedin(args) => handle_linkedin(args),
        CampaignCommand::Export(args) => handle_export(args),
    }
}

/// Resolve the campaign name: use explicit arg, then active campaign, then error.
fn resolve_campaign(explicit: Option<&str>) -> Result<String> {
    if let Some(name) = explicit {
        return Ok(name.to_string());
    }
    active_campaign()?
        .ok_or_else(|| MagpieError::Custom("No campaign specified and no active campaign set. Use `magpie campaign create` or `magpie campaign select`.".to_string()))
}

/// Ensure campaign directory structure exists.
fn ensure_campaign_dirs(name: &str) -> Result<PathBuf> {
    let dir = campaign_path(name)?;
    for sub in &["leads", "crawled", "brand", "team", "linkedin", "exports"] {
        fs::create_dir_all(dir.join(sub))?;
    }
    Ok(dir)
}

fn handle_create(args: CreateArgs) -> Result<()> {
    if campaign_exists(&args.name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" already exists.",
            args.name
        )));
    }

    let dir = ensure_campaign_dirs(&args.name)?;
    let now = now_iso();

    let manifest = CampaignManifest {
        name: args.name.clone(),
        industry: args.industry.clone(),
        location: args.location.clone(),
        created_at: now.clone(),
        status: "created".to_string(),
        lead_count: 0,
        crawled_count: 0,
        analyzed_count: 0,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(campaign_manifest_path(&args.name)?, manifest_json)?;

    println!("✅ Campaign \"{}\" created.", args.name);
    println!("   Industry: {}", args.industry);
    println!("   Location: {}", args.location);
    println!("   Directory: {:?}", dir);

    if args.select {
        set_active_campaign(&args.name)?;
        println!("   ✅ Set as active campaign.");
    }

    Ok(())
}

fn handle_list() -> Result<()> {
    let campaigns = list_campaigns()?;
    let active = active_campaign()?;

    if campaigns.is_empty() {
        println!("No campaigns found.");
        println!("Create one: magpie campaign create --name <name> --industry <industry> --location <location>");
        return Ok(());
    }

    println!("\n📋 Magpie Campaigns ({})\n", campaigns.len());
    for name in &campaigns {
        let marker = if active.as_deref() == Some(name) { " ▶" } else { "  " };
        let manifest_path = campaign_manifest_path(name)?;
        let status_str = if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<CampaignManifest>(&content) {
                    format!("{} | {} leads | {}", manifest.status, manifest.lead_count, manifest.location)
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };
        println!("{}{} — {}", marker, name, status_str);
    }

    if active.is_some() {
        println!("\nActive campaign: {}", active.unwrap());
    } else {
        println!("\nNo active campaign. Set one: magpie campaign select <name>");
    }

    Ok(())
}

fn handle_select(name: &str) -> Result<()> {
    if !campaign_exists(name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" not found.",
            name
        )));
    }
    set_active_campaign(name)?;
    println!("✅ Campaign \"{}\" is now active.", name);
    Ok(())
}

fn handle_status(campaign_name: Option<&str>) -> Result<()> {
    let name = resolve_campaign(campaign_name)?;

    if !campaign_exists(&name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" not found.",
            name
        )));
    }

    let manifest_path = campaign_manifest_path(&name)?;
    let manifest: CampaignManifest = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)?;
        serde_json::from_str(&content)?
    } else {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" has no manifest.",
            name
        )));
    };

    let leads_path = campaign_leads_path(&name)?;
    let lead_count = if leads_path.exists() {
        if let Ok(content) = fs::read_to_string(&leads_path) {
            if let Ok(leads) = serde_json::from_str::<Vec<Lead>>(&content) {
                leads.len()
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    let crawled_dir = campaign_path(&name)?.join("crawled");
    let crawled_count = if crawled_dir.exists() {
        fs::read_dir(&crawled_dir)?.filter_map(|e| e.ok()).count()
    } else {
        0
    };

    let brand_dir = campaign_path(&name)?.join("brand");
    let analyzed_count = if brand_dir.exists() {
        fs::read_dir(&brand_dir)?.filter_map(|e| e.ok()).count()
    } else {
        0
    };

    println!("\n📊 Campaign: {}", name);
    println!("   Industry: {}", manifest.industry);
    println!("   Location: {}", manifest.location);
    println!("   Status: {}", manifest.status);
    println!("   Created: {}", manifest.created_at);
    println!("   Leads discovered: {}", lead_count);
    println!("   Websites crawled: {}", crawled_count);
    println!("   Brand analyses: {}", analyzed_count);
    println!("   Directory: {:?}", campaign_path(&name)?);

    Ok(())
}

fn handle_research(args: ResearchArgs) -> Result<()> {
    let name = resolve_campaign(args.campaign.as_deref())?;

    if !campaign_exists(&name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" not found.",
            name
        )));
    }

    let manifest_path = campaign_manifest_path(&name)?;
    let mut manifest: CampaignManifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;

    let industry = args.industry.unwrap_or_else(|| manifest.industry.clone());
    let location = args.location.unwrap_or_else(|| manifest.location.clone());

    // Check for Serper API key
    let api_key = crate::config::serper_api_key()?;
    if api_key.is_none() {
        return Err(MagpieError::Custom(
            "Serper API key not configured. Run 'magpie config serper-setup' to set it."
                .to_string(),
        ));
    }

    manifest.status = "researching".to_string();
    manifest.lead_count = 0;
    save_manifest(&name, &manifest)?;

    // Run the research Python script
    let script_dir = std::env::current_dir()?.join("skills/campaign");
    let script = script_dir.join("campaign-research.py");

    if !script.exists() {
        return Err(MagpieError::Custom(
            "Campaign research script not found. Make sure skills/campaign/campaign-research.py exists."
                .to_string(),
        ));
    }

    let leads_out = campaign_leads_path(&name)?;
    let status = std::process::Command::new("python3")
        .arg(&script)
        .arg("--industry")
        .arg(&industry)
        .arg("--location")
        .arg(&location)
        .arg("--limit")
        .arg(args.limit.to_string())
        .arg("--out")
        .arg(leads_out.to_string_lossy().as_ref())
        .status()?;

    if !status.success() {
        manifest.status = "created".to_string();
        save_manifest(&name, &manifest)?;
        return Err(MagpieError::Custom("Campaign research failed.".to_string()));
    }

    // Count leads and update manifest
    let lead_count = if leads_out.exists() {
        if let Ok(content) = fs::read_to_string(&leads_out) {
            if let Ok(leads) = serde_json::from_str::<Vec<Lead>>(&content) {
                leads.len() as u32
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    manifest.status = "researched".to_string();
    manifest.lead_count = lead_count;
    save_manifest(&name, &manifest)?;

    println!("✅ Research complete: {} leads found for {} in {}.", lead_count, industry, location);
    println!("   Leads saved to: {:?}", leads_out);

    Ok(())
}

fn handle_crawl(args: CrawlArgs) -> Result<()> {
    let name = resolve_campaign(args.campaign.as_deref())?;

    if !campaign_exists(&name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" not found.",
            name
        )));
    }

    let leads_path = campaign_leads_path(&name)?;
    if !leads_path.exists() {
        return Err(MagpieError::Custom(
            "No leads found. Run `magpie campaign research` first.".to_string(),
        ));
    }

    let leads: Vec<Lead> = serde_json::from_str(&fs::read_to_string(&leads_path)?)?;
    let crawl_dir = campaign_path(&name)?.join("crawled");
    fs::create_dir_all(&crawl_dir)?;

    let limit = args.limit as usize;
    let mut crawled = 0;

    for (i, lead) in leads.iter().enumerate() {
        if crawled >= limit {
            break;
        }
        let website = match &lead.website {
            Some(w) if !w.is_empty() => w,
            _ => continue,
        };

        let out_file = crawl_dir.join(format!("{}-{}.json", i + 1, slugify(&lead.name)));
        let status = std::process::Command::new("python3")
            .arg(format!("{}/skills/crawl/magpie-crawl.py", std::env::current_dir()?.display()))
            .arg(website)
            .arg("--format")
            .arg("json")
            .arg("--out")
            .arg(out_file.to_string_lossy().as_ref())
            .status()?;

        if status.success() {
            crawled += 1;
            println!("[{}/{}] Crawled: {}", crawled, limit, lead.name);
        } else {
            println!("[{}/{}] Failed: {}", crawled, limit, lead.name);
        }
    }

    // Update manifest
    let manifest_path = campaign_manifest_path(&name)?;
    let mut manifest: CampaignManifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    manifest.status = "crawled".to_string();
    manifest.crawled_count = crawled as u32;
    save_manifest(&name, &manifest)?;

    println!("✅ Crawl complete: {} websites crawled.", crawled);

    Ok(())
}

fn handle_analyze(args: AnalyzeArgs) -> Result<()> {
    let name = resolve_campaign(args.campaign.as_deref())?;

    if !campaign_exists(&name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" not found.",
            name
        )));
    }

    let crawl_dir = campaign_path(&name)?.join("crawled");
    if !crawl_dir.exists() {
        return Err(MagpieError::Custom(
            "No crawled websites found. Run `magpie campaign crawl` first.".to_string(),
        ));
    }

    let brand_dir = campaign_path(&name)?.join("brand");
    fs::create_dir_all(&brand_dir)?;

    let mut analyzed = 0;
    for entry in fs::read_dir(&crawl_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let record: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
        let url = record.get("final_url").and_then(|v| v.as_str()).unwrap_or("");
        if url.is_empty() {
            continue;
        }

        // Vision brand analysis
        if args.brand {
            let out_file = brand_dir.join(format!("{}.md", slugify(url)));
            let status = std::process::Command::new("python3")
                .arg(format!("{}/skills/brand-analysis/magpie-brand-analysis.py", std::env::current_dir()?.display()))
                .arg(url)
                .arg("--format")
                .arg("md")
                .arg("--out")
                .arg(out_file.to_string_lossy().as_ref())
                .status();
            if let Ok(s) = status {
                if s.success() {
                    analyzed += 1;
                }
            }
        }
    }

    let manifest_path = campaign_manifest_path(&name)?;
    let mut manifest: CampaignManifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    manifest.status = "analyzed".to_string();
    manifest.analyzed_count = analyzed as u32;
    save_manifest(&name, &manifest)?;

    println!("✅ Analysis complete: {} websites analyzed.", analyzed);

    Ok(())
}

fn handle_linkedin(args: LinkedinArgs) -> Result<()> {
    let name = resolve_campaign(args.campaign.as_deref())?;

    if !campaign_exists(&name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" not found.",
            name
        )));
    }

    let api_key = crate::config::serper_api_key()?;
    if api_key.is_none() {
        return Err(MagpieError::Custom(
            "Serper API key not configured. Run 'magpie config serper-setup' to set it."
                .to_string(),
        ));
    }

    let leads_path = campaign_leads_path(&name)?;
    if !leads_path.exists() {
        return Err(MagpieError::Custom(
            "No leads found. Run `magpie campaign research` first.".to_string(),
        ));
    }

    let leads: Vec<Lead> = serde_json::from_str(&fs::read_to_string(&leads_path)?)?;
    let linkedin_dir = campaign_path(&name)?.join("linkedin");
    fs::create_dir_all(&linkedin_dir)?;

    let mut found = 0;
    for lead in &leads {
        let query = format!("{} LinkedIn", lead.name);
        // Use Serper search to find LinkedIn page
        let body = serde_json::json!({
            "q": query,
            "num": 5
        });

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", api_key.as_deref().unwrap_or(""))
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        match resp {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<serde_json::Value>() {
                    let linkedin_url = data.get("organic").and_then(|org| org.as_array())
                        .and_then(|arr| arr.iter().find(|item| {
                            item.get("link").and_then(|l| l.as_str())
                                .map(|l| l.contains("linkedin.com")).unwrap_or(false)
                        }))
                        .and_then(|item| item.get("link").and_then(|l| l.as_str()))
                        .map(|s| s.to_string());

                    if let Some(li_url) = linkedin_url {
                        let out_file = linkedin_dir.join(format!("{}.json", slugify(&lead.name)));
                        let linkedin_record = serde_json::json!({
                            "business_name": lead.name,
                            "linkedin_url": li_url,
                            "source": "serper-google"
                        });
                        fs::write(out_file, serde_json::to_string_pretty(&linkedin_record)?)?;
                        found += 1;
                        println!("✅ LinkedIn found: {} -> {}", lead.name, li_url);
                    }
                }
            }
            _ => {
                eprintln!("⚠️  LinkedIn search failed for: {}", lead.name);
            }
        }
    }

    println!("✅ LinkedIn discovery complete: {} profiles found.", found);

    Ok(())
}

fn handle_export(args: ExportArgs) -> Result<()> {
    let name = resolve_campaign(args.campaign.as_deref())?;

    if !campaign_exists(&name)? {
        return Err(MagpieError::Custom(format!(
            "Campaign \"{}\" not found.",
            name
        )));
    }

    let manifest_path = campaign_manifest_path(&name)?;
    let manifest: CampaignManifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;

    let leads_path = campaign_leads_path(&name)?;
    let leads: Vec<Lead> = if leads_path.exists() {
        serde_json::from_str(&fs::read_to_string(&leads_path)?)?
    } else {
        Vec::new()
    };

    let out = args.out.unwrap_or_else(|| campaign_crm_path(&name).unwrap());

    // Build CRM export records
    let mut crm_records: Vec<serde_json::Value> = Vec::new();
    for lead in &leads {
        let mut record = serde_json::json!({
            "campaign": name,
            "industry": manifest.industry,
            "location": manifest.location,
            "business_name": lead.name,
            "address": lead.address,
            "phone": lead.phone,
            "website": lead.website,
            "rating": lead.rating,
            "reviews": lead.reviews,
            "category": lead.category,
            "source": lead.source,
            "discovered_at": lead.discovered_at
        });

        // Attach LinkedIn if available
        let li_file = campaign_path(&name)?.join("linkedin").join(format!("{}.json", slugify(&lead.name)));
        if li_file.exists() {
            if let Ok(li_content) = fs::read_to_string(&li_file) {
                if let Ok(li_data) = serde_json::from_str::<serde_json::Value>(&li_content) {
                    record["linkedin_url"] = li_data.get("linkedin_url").cloned().unwrap_or(serde_json::Value::Null);
                }
            }
        }

        // Attach crawled data if available
        // Find crawled file matching this business
        let crawl_dir = campaign_path(&name)?.join("crawled");
        if crawl_dir.exists() {
            for entry in fs::read_dir(&crawl_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(crawl_data) = serde_json::from_str::<serde_json::Value>(&content) {
                                let crawl_title = crawl_data.get("title").and_then(|v| v.as_str()).unwrap_or("");
                                if !crawl_title.is_empty() && lead.name.to_lowercase() == crawl_title.to_lowercase() {
                                    record["crawled_title"] = serde_json::Value::String(crawl_title.to_string());
                                    record["crawled_description"] = crawl_data.get("description").cloned().unwrap_or(serde_json::Value::Null);
                                    record["crawled_email"] = crawl_data.get("email").cloned().unwrap_or(serde_json::Value::Null);
                                    record["crawled_phone"] = crawl_data.get("phone").cloned().unwrap_or(serde_json::Value::Null);
                                    record["crawled_address"] = crawl_data.get("address").cloned().unwrap_or(serde_json::Value::Null);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        crm_records.push(record);
    }

    let export_json = serde_json::to_string_pretty(&crm_records)?;
    fs::write(&out, export_json)?;

    println!("✅ CRM export complete: {} records written to {:?}", crm_records.len(), out);
    println!("   Import this JSON into your local CRM (HubSpot, Salesforce, Pipedrive, Zoho, etc.)");

    Ok(())
}

fn save_manifest(name: &str, manifest: &CampaignManifest) -> Result<()> {
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(campaign_manifest_path(name)?, json)?;
    Ok(())
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Return current time as a simple ISO-8601-like string (no external dependency).
fn now_iso() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => {
            let secs = n.as_secs();
            let days = secs / 86400;
            let year = 1970 + (days / 365) as u32;
            let day_of_year = (days % 365) as u32;
            // Approximate ISO-8601 string good enough for manifest timestamps
            format!("{}-{:02}-{:02}T00:00:00Z", year, 1 + day_of_year / 31, 1 + day_of_year % 31)
        }
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Acme Dental Clinic"), "acme-dental-clinic");
        assert_eq!(slugify("Joe's Plumbing & Co"), "joe-s-plumbing-co");
    }
}
