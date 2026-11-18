#!/usr/bin/env python3
"""
magpie-company-setup.py — Setup Magpie with company knowledge from website or PDFs.

This is the starting point for company-focused Magpie usage.
Pass a website URL or PDF files, and Magpie will:
1. Crawl/analyze the company information
2. Save it to knowledge/
3. Ingest it into RAG
4. Make all subsequent magpie run commands company-focused

Usage:
    python3 magpie-company-setup.py --website https://example.com
    python3 magpie-company-setup.py --pdf document1.pdf document2.pdf
    python3 magpie-company-setup.py --website https://example.com --pdf brochure.pdf
    python3 magpie-company-setup.py --interactive
"""

import sys
import os
import shutil
import argparse
from pathlib import Path
from datetime import datetime, timezone

KNOWLEDGE_DIR = Path(__file__).resolve().parent.parent.parent / "knowledge"


def ensure_knowledge_dir():
    KNOWLEDGE_DIR.mkdir(parents=True, exist_ok=True)


def clear_knowledge_dir():
    """Clear all files in knowledge/ - website URL is the primary key."""
    if KNOWLEDGE_DIR.exists():
        for entry in KNOWLEDGE_DIR.iterdir():
            if entry.is_file():
                entry.unlink()
            elif entry.is_dir():
                import shutil
                shutil.rmtree(entry)
    print(f"🗑️  Cleared existing knowledge directory: {KNOWLEDGE_DIR}")


def copy_pdf_to_knowledge(pdf_path):
    """Copy a PDF file into the knowledge directory."""
    src = Path(pdf_path)
    if not src.exists():
        print(f"❌ PDF not found: {src}", file=sys.stderr)
        return False
    if src.suffix.lower() != ".pdf":
        print(f"❌ Not a PDF file: {src}", file=sys.stderr)
        return False
    dest = KNOWLEDGE_DIR / src.name
    shutil.copy2(src, dest)
    print(f"✅ Copied PDF to knowledge/: {dest.name}")
    return True


def crawl_website(url):
    """Crawl a website using the crawl skill and save to knowledge/."""
    import subprocess
    result = subprocess.run(
        ["python3", str(Path(__file__).resolve().parent.parent / "crawl/magpie-crawl.py"),
         url, "--format", "json", "--out", str(KNOWLEDGE_DIR / "crawled-pages.json")],
        capture_output=True, text=True, timeout=120
    )
    if result.returncode != 0:
        print(f"⚠️  Crawl warning for {url}: {result.stderr.strip()}", file=sys.stderr)
        return False
    print(f"✅ Crawled: {url}")
    return True


def analyze_brand(url, model, api_key, base_url):
    """Run brand analysis on a website and save to knowledge/."""
    import subprocess
    env = os.environ.copy()
    env["MAGPIE_API_KEY"] = api_key
    env["MAGPIE_BASE_URL"] = base_url
    env["MAGPIE_MODEL"] = model

    result = subprocess.run(
        ["python3", str(Path(__file__).resolve().parent.parent / "brand-analysis/magpie-brand-analysis.py"),
         url, "--format", "md", "--out", str(KNOWLEDGE_DIR / "brand-analysis.md")],
        env=env, capture_output=True, text=True, timeout=120
    )
    if result.returncode != 0:
        print(f"⚠️  Brand analysis warning for {url}: {result.stderr.strip()}", file=sys.stderr)
        return False
    print(f"✅ Brand analysis saved: knowledge/brand-analysis.md")
    return True


def ingest_rag():
    """Run RAG ingest on the knowledge directory."""
    import subprocess
    result = subprocess.run(
        ["python3", str(Path(__file__).resolve().parent / "magpie-rag.py"), "ingest"],
        capture_output=True, text=True, timeout=120
    )
    if result.returncode != 0:
        print(f"⚠️  RAG ingest warning: {result.stderr.strip()}", file=sys.stderr)
        return False
    print(f"✅ RAG index built from knowledge/")
    return True


def write_company_summary(website_urls, pdf_files):
    """Write a company summary file linking all sources."""
    summary = KNOWLEDGE_DIR / "company-summary.md"
    lines = [
        "# Company Knowledge Base",
        "",
        f"Generated: {datetime.now(timezone.utc).isoformat()}",
        "",
        "## Sources",
        "",
    ]
    if website_urls:
        lines.append("### Websites")
        for url in website_urls:
            lines.append(f"- {url}")
        lines.append("")
    if pdf_files:
        lines.append("### Documents")
        for pdf in pdf_files:
            lines.append(f"- {pdf}")
        lines.append("")
    lines.extend([
        "## How to Use",
        "",
        "All company knowledge is stored in this directory and automatically",
        "used by Magpie when you run: `magpie run \"your marketing request\"`",
        "",
        "To update: re-run `magpie company-setup` with new URLs or PDFs.",
    ])
    summary.write_text("\n".join(lines), encoding="utf-8")
    print(f"✅ Company summary: knowledge/company-summary.md")


def main():
    parser = argparse.ArgumentParser(
        description="Setup Magpie with company knowledge from website or PDFs"
    )
    parser.add_argument("--website", nargs="+", help="Company website URL(s)")
    parser.add_argument("--pdf", nargs="+", help="Company PDF file(s)")
    parser.add_argument("--model", default=None, help="Vision model for brand analysis")
    args = parser.parse_args()

    if not args.website and not args.pdf:
        parser.print_help()
        print("\n❌ Error: Provide at least --website or --pdf", file=sys.stderr)
        sys.exit(1)

    ensure_knowledge_dir()
    clear_knowledge_dir()

    api_key = os.environ.get("MAGPIE_API_KEY") or os.environ.get("OPENAI_API_KEY")
    base_url = os.environ.get("MAGPIE_BASE_URL", "https://api.openai.com/v1")
    model = args.model or os.environ.get("MAGPIE_MODEL", "nemotron-3.5-super-vl-free")

    website_urls = args.website or []
    pdf_files = args.pdf or []

    print("🏢 Magpie Company Setup")
    print("=" * 50)
    print()

    # Step 1: Copy PDFs
    if pdf_files:
        print("📄 Processing PDFs...")
        for pdf in pdf_files:
            copy_pdf_to_knowledge(pdf)
        print()

    # Step 2: Crawl websites
    if website_urls:
        print("🌐 Crawling websites...")
        for url in website_urls:
            crawl_website(url)
        print()

    # Step 3: Brand analysis (needs API key)
    if website_urls and api_key:
        print("🎨 Running brand analysis...")
        for url in website_urls:
            analyze_brand(url, model, api_key, base_url)
        print()
    elif website_urls and not api_key:
        print("⚠️  Skipping brand analysis (no MAGPIE_API_KEY set)")
        print("   Set it and re-run to get brand colors, fonts, logo analysis")
        print()

    # Step 4: Write company summary
    write_company_summary(website_urls, pdf_files)

    # Step 5: Ingest into RAG
    print("🧠 Building RAG index...")
    ingest_rag()
    print()

    print("=" * 50)
    print("✅ Company setup complete!")
    print()
    print("🚀 Now run Magpie with your company context:")
    print('   magpie run "write a product launch email"')
    print('   magpie run "create ad copy for our new campaign"')
    print('   magpie web  (then open http://127.0.0.1:7070)')
    print()


if __name__ == "__main__":
    main()
