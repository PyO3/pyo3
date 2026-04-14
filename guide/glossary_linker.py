"""mdbook preprocessor to auto-link glossary terms.

Parses glossary.md for terms defined using definition list syntax:

    term A
      : This is a definition of term A.

    term B
      : This is a definition of term B.

For each chapter (except the glossary itself), the first occurrence of each
term is replaced with a link to the glossary entry.

An HTML comment block at the bottom of glossary.md maps additional
terms to arbitrary URLs (e.g. upstream Python/Rust glossary entries):

Tested against mdbook 0.5.0.
"""

import json
import re
import sys


def slugify(term):
    """Convert a term to a URL-friendly anchor slug."""
    return re.sub(r"[^\w\s-]", "", term.lower()).strip().replace(" ", "-")


def parse_glossary(content):
    """Parse glossary.md and return (local_terms, external_terms).

    local_terms: dict of {term: url} for definition-list entries (link to glossary anchors).
    external_terms: dict of {term: url} from the <!-- external-glossary-links --> comment.
    """
    local_terms = {}
    external_terms = {}

    # Parse definition-list terms: a non-indented line followed by a line
    # starting with "  : " (the definition).
    for m in re.finditer(r"(?m)^([^\s#<>!`\[].+)\n  : ", content):
        term = m.group(1).strip()
        local_terms[term] = f"glossary.md#{slugify(term)}"

    # Parse <!-- external-glossary-links ... --> block for external URLs.
    link_block = re.search(
        r"<!--\s*external-glossary-links\s*\n(.*?)-->", content, re.DOTALL
    )
    if not link_block:
        raise ValueError("Glossary is missing <!-- external-glossary-links --> block")

    for line in link_block.group(1).strip().splitlines():
        line = line.strip()
        if not line:
            continue
        # "term: url" or "term | url"
        parts = re.split(r"\s*[:|]\s*", line, maxsplit=1)
        if len(parts) == 2:
            term, url = parts[0].strip(), parts[1].strip()
            if term and url:
                external_terms[term] = url

    return local_terms, external_terms


# ---------------------------------------------------------------------------
# Replacement logic
# ---------------------------------------------------------------------------

# Matches fenced code blocks, inline code, and markdown links so we can skip
# them.  Everything else is "plain text" we can scan for terms.
_SKIP_PATTERN = re.compile(
    r"^```[^\n]*\n[\s\S]*?^```\s*$"  # fenced code blocks (``` to ```, multiline)
    r"|`[^`\n]+`"  # inline code (single line only)
    r"|\[[^\]]*\]\([^\)]*\)"  # markdown links (entire [...](...))
    r"|<[^>\n]+>"  # HTML tags (single line only)
    r"|^\s*>.*$"  # block quotes (single line)
    r"|^.+(?=\n  : )"  # definition list term lines
    r"|^#{1,6}\s+.*$",  # headings
    re.MULTILINE,
)


def link_terms_in_content(content, terms, first_only=True, url_prefix=""):
    """Replace occurrences of glossary terms with markdown links.

    If first_only is True (default), only the first occurrence of each term is
    linked. If False, every occurrence is linked.

    Skips code blocks, inline code, existing links, and HTML tags.
    """
    linked = set()

    # Build a combined pattern matching any term, longest first so that
    # multi-word terms match before their sub-terms.
    sorted_terms = sorted(terms.keys(), key=len, reverse=True)
    if not sorted_terms:
        return content

    # Allow optional trailing "s" so plurals like "wheels" match "wheel".
    escaped = [re.escape(t) + r"s?" for t in sorted_terms]
    term_pattern = re.compile(r"\b(" + "|".join(escaped) + r")\b")

    # Find all protected spans.
    protected = []
    for m in _SKIP_PATTERN.finditer(content):
        protected.append((m.start(), m.end()))

    # Parse <!-- no-glossary:term --> comments to suppress specific terms.
    # Maps line number to set of suppressed term names (lowercased).
    _suppressed_re = re.compile(r"<!--\s*no-glossary-link:([\w\s-]+?)\s*-->")
    suppressed_at_line = {}
    for m in _suppressed_re.finditer(content):
        term_name = m.group(1).strip().lower()
        # Find the next line after the comment.
        next_line_start = content.find("\n", m.end())
        if next_line_start == -1:
            continue
        next_line_start += 1
        next_line_end = content.find("\n", next_line_start)
        if next_line_end == -1:
            next_line_end = len(content)
        suppressed_at_line.setdefault((next_line_start, next_line_end), set()).add(
            term_name
        )

    def in_protected(start, end):
        for ps, pe in protected:
            if start >= ps and end <= pe:
                return True
        return False

    def is_suppressed(canonical, start):
        """Check if this term is suppressed by a <!-- no-glossary:term --> on the same line."""
        for (ls, le), suppressed_terms in suppressed_at_line.items():
            if ls <= start < le and canonical.lower() in suppressed_terms:
                return True
        return False

    result = []
    last = 0

    for m in term_pattern.finditer(content):
        matched_term = m.group(1)

        # Find the canonical term. Try exact match, then case-insensitive,
        # then strip trailing "s" for plural forms.
        canonical = None
        for candidate in (matched_term, matched_term.rstrip("s")):
            if candidate in terms:
                canonical = candidate
                break
            for t in terms:
                if t.lower() == candidate.lower():
                    canonical = t
                    break
            if canonical is not None:
                break
        if canonical is None:
            continue

        if first_only and canonical in linked:
            continue

        if in_protected(m.start(), m.end()):
            continue

        if is_suppressed(canonical, m.start()):
            continue

        linked.add(canonical)
        result.append(content[last : m.start()])
        url = terms[canonical]
        if url_prefix and not url.startswith("http"):
            url = url_prefix + url
        result.append(f"[{matched_term}]({url})")
        last = m.end()

    result.append(content[last:])
    return "".join(result)


# ---------------------------------------------------------------------------
# mdbook preprocessor interface
# ---------------------------------------------------------------------------


def find_glossary_content(items):
    """Walk the book items to find and return the glossary chapter content."""
    for item in items:
        if not isinstance(item, dict) or "Chapter" not in item:
            continue
        ch = item["Chapter"]
        if ch.get("path") and ch["path"].endswith("glossary.md"):
            return ch["content"]
        result = find_glossary_content(ch.get("sub_items", []))
        if result is not None:
            return result
    return None


def process_item(item, local_terms, external_terms):
    """Recursively process a book item, linking glossary terms."""
    if not isinstance(item, dict) or "Chapter" not in item:
        return

    ch = item["Chapter"]
    path = ch.get("path", "")

    if path and path.endswith("glossary.md"):
        # On the glossary page itself, link external terms (all occurrences).
        ch["content"] = link_terms_in_content(
            ch["content"], external_terms, first_only=False
        )
    elif path:
        # Compute relative prefix for local glossary links based on depth.
        prefix = "../" * path.count("/")
        all_terms = {**local_terms, **external_terms}
        ch["content"] = link_terms_in_content(
            ch["content"], all_terms, url_prefix=prefix
        )

    for sub in ch.get("sub_items", []):
        process_item(sub, local_terms, external_terms)


def main():
    for line in sys.stdin:
        if not line.strip():
            continue
        [context, book] = json.loads(line)

        # Parse glossary terms from the glossary chapter.
        glossary_content = find_glossary_content(book["items"])
        if glossary_content is None:
            # No glossary found; pass through unchanged.
            json.dump(book, fp=sys.stdout)
            return

        local_terms, external_terms = parse_glossary(glossary_content)

        # Process all chapters.
        for item in book["items"]:
            process_item(item, local_terms, external_terms)

        json.dump(book, fp=sys.stdout)
        return


if __name__ == "__main__":
    main()
