"""Botanical names -> (genus, species) so different spellings of one plant compare equal.

"Salvia nemorosa 'Violet Profusion'" -> ("salvia", "nemorosa")
"Abelia x grandiflora 'Sherwoodii'"  -> ("abelia", "grandiflora")
"Allium 'Millennium'"                -> ("allium", None)        (cultivar only: no species)
"Lactuca virosa L."                  -> ("lactuca", "virosa")   (author names are ignored)
"""
import re

_CULTIVAR = re.compile(r"'[^']*'|\"[^\"]*\"|‘[^’]*’")
_SKIP = {"x", "var", "subsp", "ssp", "f", "cv", "hybrid", "sp", "spp"}


def taxon(name):
    """(genus, species or None), lower case; (None, None) when there is no usable word."""
    s = _CULTIVAR.sub(" ", (name or "").replace("�", " x ").replace("×", " x "))
    toks = [t for t in re.findall(r"[A-Za-z][A-Za-z-]*", s) if t.lower() not in _SKIP]
    if not toks:
        return None, None
    genus = toks[0].lower()
    # a species epithet is written in lower case; a capitalised second word is a cultivar or author
    species = toks[1].lower() if len(toks) > 1 and toks[1][0].islower() else None
    return genus, species


def label(genus, species):
    """'Salvia nemorosa' (or 'Salvia' with no species), as used in text prompts."""
    return f"{genus.capitalize()} {species}" if species else genus.capitalize()
