# The plant admin, converted from Streamlit to Rust

> **History note for Jun:** this page records how the site was converted, on
> the old server, in October 2026. The `tools/` scripts it mentions ran on that
> server and are not part of your copy. Your own copy is installed and looked
> after with `garden.sh` (see `START-HERE/`).

`https://junsgarden.zapto.org/streamlit/` is the garden's plant admin
("Jun & Kona's Garden Orchestrator"): 14 tabs for adding plants, filling in
missing data, photos, approvals, beds, the wishlist and barcodes. It used to
be a separate Python Streamlit app; since 2026-10-07 it is part of the Rust
site, drawn to look the same.

## Where things are

| What | Where |
|---|---|
| The code | `rust/junsgarden/src/orchestrator/` (start with `mod.rs`; each file says what it holds) |
| What each tab does | `SPEC.md` |
| Where it differs from Streamlit | `DEVIATIONS.md` |
| The original's text and screenshots | `baseline-text/`, `baseline-screens/` |
| The page-by-page comparison tool | `tools/compare_tabs.py` |

## Who can use it

Jun and Jason, signed in with the garden login. Anybody else is sent to
`/login.php`.

## Settings it needs

In the site's env file (never in the repo):

- `JUNSGARDEN_GEMINI_API_KEY`: Google's free Gemini key.
- `JUNSGARDEN_OPENROUTER_API_KEY`: OpenRouter key (free models only).
- `JUNSGARDEN_PERENUAL_KEY`: the Perenual plant database key.

Without the AI keys, the AI tabs say "No free AI key is set" and everything
else still works.

## Checking it against the original

`tools/compare_tabs.py` opens each tab, reads its text, and lists any line of
the original page that is missing from the Rust page:

```
python compare_tabs.py http://localhost:18030 "PHPSESSID=<cookie>" <baseline dir> <out dir> 1 2 3 ... 11:1 11:2 ...
```

Result on 2026-10-07: every tab and inner tab has 0 missing lines, except
Add Plant's "Account email" box, which was removed on purpose (S-11).
