# Where the Rust plant admin differs from the Streamlit app

The Rust `/streamlit/` pages are a page-by-page copy of the Streamlit app
`plant_orchestrator` (backed up 2026-10-07 in
`~/backups/streamlit-orchestrator-20261007-*` on the NUC). Every tab was
compared with the original using `tools/compare_tabs.py`; every line of text
on the original page is on the Rust page, except where listed here.

Each difference has a number (S-1, S-2, ...). The code mentions the number
where the difference lives, so you can search for it.

## Fixes of things that were broken in Streamlit

| # | Where | Streamlit | Rust |
|---|---|---|---|
| S-1 | Plant editor, Save | `qty`, `year_planted` and `additional_pictures` were not saved, so the website kept showing old numbers and photos. | They are saved along with `quantity` and `year_added`. |
| S-3 | 🗺️ Bed Assignment | Wrote `garden_beds` only; the website reads `flower_bed`, so the two drifted apart. | Writes both. |
| S-5 | 📄 HTML exports | Expected the months as a list, so the stored 12-letter code (`..MAMJ......`) printed wrong. | Reads the 12-letter code. |
| S-6 | 🌟 Wishlist, Promote | "Yes, promote anyway" was a button inside a button and never worked. | It works. |
| S-9 | 🔒 AI — Missing Only | "Next Plant" fetched the same unsaved plant again. | Moves on to the next plant, and wraps around at the end. |

## Safety changes

| # | Where | Streamlit | Rust |
|---|---|---|---|
| S-2 | Plant editor, 🗑 Wipe File | Deleted the photo file from disk. | Takes the photo out of the plant's gallery but keeps the file (another plant, or the website's backups, may use it). |
| S-7 | 🌟 Wishlist, Remove | Deleted at once. | Asks "Remove … ? This cannot be undone." first. |
| S-13 | 🔄 JSON Pipeline imports | Wrote any key in the uploaded file as a column name. | Writes only columns the table really has; anything else is ignored. |

## The AI (free models only)

| # | Where | Streamlit | Rust |
|---|---|---|---|
| S-4 | Every AI tab, sidebar | Ollama `phi4` on the author's PC, or Anthropic with a paid key; a "Claude Model Name" box in the sidebar. | Free cloud models only: Google Gemini's free tier, then OpenRouter's free models (the same chain garden-ingest uses). Never the PC's graphics card, never a paid key. The sidebar box is shown greyed out with a note. Same prompt, same rules, same "never overwrite a person's value" safety net. |
| S-8 | 🔒 AI — Missing Only, companions | Asked Anthropic. | Asks the free models; answers are still checked against your own plant list. |
| S-10 | AI editors, 🌐 Fetch & Stage Image | Staged the photo, saved with Save Changes. | Attaches the photo to the plant at once (Save Changes still saves the AI's other values). |
| S-12 | 🌱 Add Plant, AI Enrich | When the AI failed it just said "no suggestions". | Also says why (for example, the free tier is busy for today). |
| S-14 | 🔄 Batch Enrich | An AI failure filled nothing, silently. | It is listed under "plants failed" with the reason. |

The AI tabs run in the background and the page reloads itself while it works,
like Streamlit's "⏳ 3 plants in queue…". Their results (and the pipeline's
"remembered for the session" lists) are kept in `data/orchestrator-ai/` on the
server, so Jun and the author see the same results; Streamlit kept them per browser
tab.

## Sign-in

| # | Where | Streamlit | Rust |
|---|---|---|---|
| S-11 | 🌱 Add Plant | Asked for an "Account email" (Jun's or the author's). | The garden login already says who you are, so the box is gone. The whole admin needs the garden login (Jun or the author); Streamlit sat behind a password page plus the author's household sign-in. |

## Small things you might notice

- Buttons that change something send the page back to the same tab and plant
  with the result shown at the top. That is what a Streamlit "rerun" looked
  like.
- Multi-pick boxes (bulk exports) reload the page when you add or remove a
  plant, as Streamlit did.
- The wishlist import also shows the amber "proposed overwrites" table that
  Phase 3 has, and a Skip button.
- New field `product_url`: the "Source" link to the vendor's product page, in the
  plant editor, the wishlist editor, the catalog page's edit form, the JSON Pipeline
  and the Field Reference.
  Only a full `https://` address is saved; anything else is refused with a message.
  새 칸 `product_url`: 판매처 상품 페이지로 가는 "Source" 링크예요. `https://`로 시작하는
  전체 주소만 저장되고, 다른 값은 안내 문구와 함께 거절돼요.
