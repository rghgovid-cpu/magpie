#!/usr/bin/env python3
"""
magpie-crawl.py — Web crawler for Magpie skill system.
Extracts CRM-ready data (title, company, emails, phones, address, etc.)
and outputs CSV or JSON.

Usage:
    python3 magpie-crawl.py <url> [--format csv|json] [--out <file>]
    python3 magpie-crawl.py <url1> <url2> ... [--format csv] [--out leads.csv]
"""

import sys
import re
import csv
import json
import argparse
from datetime import datetime, timezone
from urllib.parse import urljoin, urlparse
from html import unescape

try:
    import requests
    from bs4 import BeautifulSoup
    HAS_REQUESTS = True
except ImportError:
    HAS_REQUESTS = False


def fetch_html(url):
    """Fetch HTML from a URL."""
    if HAS_REQUESTS:
        try:
            resp = requests.get(
                url,
                timeout=15,
                headers={
                    "User-Agent": "MagpieBot/1.0 (compatible; Magpie CRM Crawler; +https://github.com/rghgovid-cpu/magpie)"
                },
            )
            resp.raise_for_status()
            return resp.text, resp.url
        except Exception as e:
            print(f"Error fetching {url}: {e}", file=sys.stderr)
            return None, url
    else:
        # Fallback using urllib (standard library)
        try:
            import urllib.request
            req = urllib.request.Request(
                url,
                headers={
                    "User-Agent": "MagpieBot/1.0 (compatible; Magpie CRM Crawler)"
                },
            )
            with urllib.request.urlopen(req, timeout=15) as resp:
                final_url = resp.geturl()
                html = resp.read().decode("utf-8", errors="replace")
                return html, final_url
        except Exception as e:
            print(f"Error fetching {url}: {e}", file=sys.stderr)
            return None, url


def clean_text(text):
    """Clean and normalize text for CRM fields."""
    if not text:
        return ""
    # Unescape HTML entities
    text = unescape(text)
    # Replace whitespace sequences with single space
    text = re.sub(r"\s+", " ", text)
    return text.strip()


def extract_emails(text):
    """Extract email addresses from text."""
    emails = re.findall(
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", text
    )
    return list(dict.fromkeys(emails))  # deduplicate, preserve order


def extract_phones(text):
    """Extract phone numbers from text."""
    # Match common phone patterns
    phones = re.findall(
        r"(?:\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}", text
    )
    return list(dict.fromkeys(phones))


def extract_address(soup):
    """Attempt to extract a physical address from the page."""
    # Look for address tag
    addr_tag = soup.find("address")
    if addr_tag:
        return clean_text(addr_tag.get_text())

    # Look for microdata or common address patterns
    for selector in [
        '[itemtype*="PostalAddress"]',
        '.address',
        '#address',
        '.contact-address',
        '.location',
    ]:
        el = soup.select_one(selector)
        if el:
            return clean_text(el.get_text())

    return ""


def extract_company(soup, url):
    """Extract company name from page."""
    # Try og:site_name
    og_site = soup.find("meta", property="og:site_name")
    if og_site and og_site.get("content"):
        return clean_text(og_site["content"])

    # Try schema.org organization
    org = soup.find(itemtype=re.compile("Organization"))
    if org:
        name_el = org.find(itemprop="name")
        if name_el:
            return clean_text(name_el.get_text())

    # Fallback: use domain name
    domain = urlparse(url).netloc
    # Remove www. and TLD for a readable company guess
    name = domain.replace("www.", "").split(".")[0]
    return name.title()


def extract_contact_name(soup, text):
    """Attempt to extract a contact person name."""
    # Look for common patterns
    patterns = [
        r"contact:\s*([A-Z][a-z]+ [A-Z][a-z]+)",
        r"contact person:\s*([A-Z][a-z]+ [A-Z][a-z]+)",
        r"attn:\s*([A-Z][a-z]+ [A-Z][a-z]+)",
    ]
    for pattern in patterns:
        m = re.search(pattern, text, re.IGNORECASE)
        if m:
            return clean_text(m.group(1))

    # Look for author meta
    author = soup.find("meta", {"name": "author"})
    if author and author.get("content"):
        return clean_text(author["content"])

    return ""


def crawl_url(url):
    """Crawl a single URL and return a CRM-ready record."""
    html, final_url = fetch_html(url)
    if html is None:
        return {
            "url": url,
            "final_url": final_url,
            "title": "",
            "company": "",
            "description": "",
            "email": "",
            "phone": "",
            "address": "",
            "contact_name": "",
            "links": "",
            "crawl_date": datetime.now(timezone.utc).isoformat(),
            "status": "error",
        }

    soup = BeautifulSoup(html, "html.parser")

    # Remove scripts and styles for cleaner text
    for tag in soup(["script", "style", "nav", "footer", "header"]):
        tag.decompose()

    # Title
    title_tag = soup.find("title")
    title = clean_text(title_tag.get_text()) if title_tag else ""

    # Meta description
    desc_tag = soup.find("meta", attrs={"name": "description"})
    description = clean_text(desc_tag.get("content", "")) if desc_tag else ""

    # Full text for email/phone extraction
    full_text = soup.get_text(" ", strip=True)

    # Emails
    emails = extract_emails(full_text)
    # Also check mailto: links
    for link in soup.find_all("a", href=True):
        href = link["href"]
        if href.startswith("mailto:"):
            email = href[7:].split("?")[0].strip()
            if email and email not in emails:
                emails.append(email)

    # Phones
    phones = extract_phones(full_text)
    # Also check tel: links
    for link in soup.find_all("a", href=True):
        href = link["href"]
        if href.startswith("tel:"):
            phone = href[4:].strip()
            if phone and phone not in phones:
                phones.append(phone)

    # Company
    company = extract_company(soup, url)

    # Address
    address = extract_address(soup)

    # Contact name
    contact_name = extract_contact_name(soup, full_text)

    # Links
    links = []
    for link in soup.find_all("a", href=True):
        href = link["href"]
        full_url = urljoin(final_url, href)
        if full_url.startswith("http"):
            links.append(full_url)
    links = list(dict.fromkeys(links))  # deduplicate

    return {
        "url": url,
        "final_url": final_url,
        "title": title,
        "company": company,
        "description": description[:500],  # truncate for CRM
        "email": "; ".join(emails),
        "phone": "; ".join(phones),
        "address": address,
        "contact_name": contact_name,
        "links": "; ".join(links[:20]),  # limit to 20 links
        "crawl_date": datetime.now(timezone.utc).isoformat(),
        "status": "success",
    }


def write_csv(records, output_file):
    """Write records to CSV file."""
    fieldnames = [
        "url",
        "final_url",
        "title",
        "company",
        "description",
        "email",
        "phone",
        "address",
        "contact_name",
        "links",
        "crawl_date",
        "status",
    ]
    with open(output_file, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for record in records:
            writer.writerow(record)


def write_json(records, output_file):
    """Write records to JSON file."""
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(records, f, indent=2, ensure_ascii=False)


def main():
    parser = argparse.ArgumentParser(
        description="Magpie Web Crawler — CRM-ready data extraction"
    )
    parser.add_argument("urls", nargs="+", help="URL(s) to crawl")
    parser.add_argument(
        "--format",
        choices=["csv", "json"],
        default="csv",
        help="Output format (default: csv)",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="Output file (default: leads.csv or leads.json)",
    )
    args = parser.parse_args()

    # Crawl all URLs
    records = []
    for url in args.urls:
        record = crawl_url(url)
        records.append(record)
        status = record["status"]
        print(f"[{status}] {url}", file=sys.stderr)

    # Determine output file
    if args.out:
        output_file = args.out
    else:
        output_file = "leads.csv" if args.format == "csv" else "leads.json"

    # Write output
    if args.format == "csv":
        write_csv(records, output_file)
    else:
        write_json(records, output_file)

    print(f"\n{len(records)} record(s) written to {output_file}", file=sys.stderr)
    print(
        "CRM import tip: Import the CSV into HubSpot, Salesforce, Pipedrive, or Zoho.",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
