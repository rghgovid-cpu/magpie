#!/usr/bin/env python3
"""
magpie-rag.py — Retrieval-Augmented Generation for Magpie.

Ingests company documents from knowledge/ and retrieves relevant context
to refine marketing outputs. Uses TF-IDF cosine similarity for retrieval
(scikit-learn if available, otherwise falls back to keyword matching).

Usage:
    python3 magpie-rag.py ingest                 # Index all documents in knowledge/
    python3 magpie-rag.py search "query text"     # Retrieve top relevant passages
    python3 magpie-rag.py ingest-and-search "query"  # Ingest then search
    python3 magpie-rag.py stats                   # Show index statistics

Output:
    Prints retrieved passages to stdout, one per paragraph, separated by ---.
    Each passage is prefixed with its source filename.
"""

import sys
import os
import re
import math
import json
import argparse
from pathlib import Path
from datetime import datetime, timezone

# Try to import scikit-learn for TF-IDF; fall back to keyword matching
try:
    from sklearn.feature_extraction.text import TfidfVectorizer
    from sklearn.metrics.pairwise import cosine_similarity
    HAS_SKLEARN = True
except ImportError:
    HAS_SKLEARN = False

# Try to import PDF support
try:
    import pypdf
    HAS_PYPDF = True
except ImportError:
    try:
        import PyPDF2 as pypdf
        HAS_PYPDF = True
    except ImportError:
        HAS_PYPDF = False


KNOWLEDGE_DIR = Path(__file__).resolve().parent.parent.parent / "knowledge"
INDEX_FILE = Path(__file__).resolve().parent.parent.parent / ".rag-index.json"
CHUNK_SIZE = 500  # characters per chunk
CHUNK_OVERLAP = 50  # overlap between chunks
TOP_K = 5  # number of passages to retrieve


def clean_text(text):
    """Clean and normalize text."""
    if not text:
        return ""
    text = re.sub(r"\s+", " ", text)
    return text.strip()


def chunk_text(text, chunk_size=CHUNK_SIZE, overlap=CHUNK_OVERLAP):
    """Split text into overlapping chunks."""
    if not text:
        return []
    text = clean_text(text)
    if len(text) <= chunk_size:
        return [text] if text else []

    chunks = []
    start = 0
    while start < len(text):
        end = start + chunk_size
        chunk = text[start:end]
        # Try to break at a sentence boundary
        if end < len(text):
            last_period = chunk.rfind(". ")
            last_newline = chunk.rfind("\n")
            break_point = max(last_period, last_newline)
            if break_point > start + chunk_size // 2:
                chunk = text[start:break_point + 1]
                end = break_point + 1
        chunks.append(chunk.strip())
        if end >= len(text):
            break
        start = end - overlap
        if start < 0:
            start = 0
    return [c for c in chunks if c]


def read_pdf(filepath):
    """Extract text from a PDF file."""
    if not HAS_PYPDF:
        print(f"Warning: pypdf/PyPDF2 not installed, skipping PDF: {filepath}", file=sys.stderr)
        return ""
    try:
        reader = pypdf.PdfReader(str(filepath))
        text_parts = []
        for page in reader.pages:
            try:
                text_parts.append(page.extract_text() or "")
            except Exception:
                continue
        return "\n".join(text_parts)
    except Exception as e:
        print(f"Error reading PDF {filepath}: {e}", file=sys.stderr)
        return ""


def read_file(filepath):
    """Read text from a file based on extension."""
    ext = filepath.suffix.lower()
    if ext == ".pdf":
        return read_pdf(filepath)
    else:
        try:
            with open(filepath, "r", encoding="utf-8", errors="replace") as f:
                return f.read()
        except Exception as e:
            print(f"Error reading {filepath}: {e}", file=sys.stderr)
            return ""


def load_documents(knowledge_dir):
    """Load all supported documents from the knowledge directory."""
    documents = []
    if not knowledge_dir.exists():
        return documents

    supported = {".md", ".txt", ".pdf"}
    for filepath in sorted(knowledge_dir.rglob("*")):
        if filepath.is_file() and filepath.suffix.lower() in supported:
            text = read_file(filepath)
            if text and text.strip():
                documents.append({
                    "source": str(filepath.relative_to(knowledge_dir)),
                    "full_path": str(filepath),
                    "text": text,
                })
    return documents


def build_index(documents):
    """Build a search index from documents."""
    chunks = []
    for doc in documents:
        text_chunks = chunk_text(doc["text"])
        for i, chunk in enumerate(text_chunks):
            chunks.append({
                "source": doc["source"],
                "chunk_index": i,
                "text": chunk,
            })
    return chunks


def keyword_search(chunks, query, top_k=TOP_K):
    """Simple keyword-based search (fallback when sklearn is unavailable)."""
    query_terms = set(re.findall(r"\w+", query.lower()))
    if not query_terms or not chunks:
        return []

    scored = []
    for chunk in chunks:
        text_lower = chunk["text"].lower()
        matches = sum(1 for term in query_terms if term in text_lower)
        if matches > 0:
            score = matches / len(query_terms)
            scored.append((score, chunk))

    scored.sort(key=lambda x: x[0], reverse=True)
    return scored[:top_k]


def tfidf_search(chunks, query, top_k=TOP_K):
    """TF-IDF cosine similarity search."""
    if not chunks:
        return []

    texts = [chunk["text"] for chunk in chunks]
    vectorizer = TfidfVectorizer(stop_words="english", ngram_range=(1, 2))
    tfidf_matrix = vectorizer.fit_transform(texts)
    query_vector = vectorizer.transform([query])

    similarities = cosine_similarity(query_vector, tfidf_matrix).flatten()
    scored = [(float(similarities[i]), chunks[i]) for i in range(len(chunks))]
    scored.sort(key=lambda x: x[0], reverse=True)
    return [s for s in scored if s[0] > 0][:top_k]


def search_chunks(chunks, query, top_k=TOP_K):
    """Search chunks using the best available method."""
    if HAS_SKLEARN:
        return tfidf_search(chunks, query, top_k)
    else:
        return keyword_search(chunks, query, top_k)


def ingest(knowledge_dir=KNOWLEDGE_DIR, index_file=INDEX_FILE):
    """Ingest all documents and build the search index."""
    documents = load_documents(knowledge_dir)
    if not documents:
        print("No documents found in knowledge/ directory.", file=sys.stderr)
        print(f"Add .md, .txt, or .pdf files to: {knowledge_dir}", file=sys.stderr)
        return False

    chunks = build_index(documents)

    index_data = {
        "created_at": datetime.now(timezone.utc).isoformat(),
        "document_count": len(documents),
        "chunk_count": len(chunks),
        "knowledge_dir": str(knowledge_dir),
        "chunks": chunks,
    }

    with open(index_file, "w", encoding="utf-8") as f:
        json.dump(index_data, f, indent=2, ensure_ascii=False)

    print(f"Ingested {len(documents)} document(s), {len(chunks)} chunk(s).", file=sys.stderr)
    print(f"Index saved to {index_file}", file=sys.stderr)
    return True


def load_index(index_file=INDEX_FILE):
    """Load the search index from disk."""
    if not index_file.exists():
        return None
    with open(index_file, "r", encoding="utf-8") as f:
        return json.load(f)


def search(query, index_file=INDEX_FILE, top_k=TOP_K):
    """Search the index for relevant passages."""
    index_data = load_index(index_file)
    if not index_data:
        print("No index found. Run 'ingest' first.", file=sys.stderr)
        return []

    chunks = index_data["chunks"]
    results = search_chunks(chunks, query, top_k)
    return results


def format_results(results):
    """Format search results for display."""
    if not results:
        return "No relevant passages found."

    output = []
    for score, chunk in results:
        output.append(f"[Source: {chunk['source']}]")
        output.append(chunk["text"])
        output.append("")
    return "\n".join(output)


def ingest_and_search(query, knowledge_dir=KNOWLEDGE_DIR, index_file=INDEX_FILE, top_k=TOP_K):
    """Ingest documents then search."""
    if not ingest(knowledge_dir, index_file):
        return []
    return search(query, index_file, top_k)


def stats(index_file=INDEX_FILE):
    """Show index statistics."""
    index_data = load_index(index_file)
    if not index_data:
        print("No index found. Run 'ingest' first.", file=sys.stderr)
        return
    print(f"Documents: {index_data.get('document_count', 0)}")
    print(f"Chunks: {index_data.get('chunk_count', 0)}")
    print(f"Created: {index_data.get('created_at', 'unknown')}")
    print(f"Knowledge dir: {index_data.get('knowledge_dir', 'unknown')}")


def main():
    parser = argparse.ArgumentParser(
        description="Magpie RAG — company knowledge retrieval for marketing"
    )
    subparsers = parser.add_subparsers(dest="command")

    subparsers.add_parser("ingest", help="Ingest all documents from knowledge/")
    subparsers.add_parser("stats", help="Show index statistics")

    search_parser = subparsers.add_parser("search", help="Search the knowledge index")
    search_parser.add_argument("query", help="Search query")
    search_parser.add_argument("--top-k", type=int, default=TOP_K, help="Number of results")

    ingest_search_parser = subparsers.add_parser("ingest-and-search", help="Ingest then search")
    ingest_search_parser.add_argument("query", help="Search query")
    ingest_search_parser.add_argument("--top-k", type=int, default=TOP_K, help="Number of results")

    args = parser.parse_args()

    if args.command == "ingest":
        ingest()
    elif args.command == "stats":
        stats()
    elif args.command == "search":
        results = search(args.query, top_k=args.top_k)
        print(format_results(results))
    elif args.command == "ingest-and-search":
        results = ingest_and_search(args.query, top_k=args.top_k)
        print(format_results(results))
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
