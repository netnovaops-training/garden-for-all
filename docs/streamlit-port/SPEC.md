<!-- Written 2026-10-07 from the Streamlit source (plant_orchestrator), every file read in full. Secrets shown as [secret]. -->
# plant_orchestrator (Streamlit): functional spec for the Rust rewrite

All files were in a working copy of the old Streamlit app, and I read every requested file in full. Labels below are copied character for character, including emoji and double spaces. Secrets are written as [secret].

The most important points for the port:
- **`tab_gemini_export.py` is never mounted.** `app.py` does not import it.
- **`pdf_export.py` is dead code.** Nothing calls it.
- **`backend_from_container.py` is an old copy of `app.py`.** It is a single-file app with 8 tabs and is not used.
- **The AI goes to the local model by default, not Claude.** `USE_LOCAL_AI` defaults to on, so `ask_claude_ai` calls Ollama with `phi4:latest`. The sidebar "Claude Model Name" is only used when `USE_LOCAL_AI=0`, plus one direct Anthropic call in the "🔒 AI — Missing Only" tab.
- **There is no login in the app.** Nothing ever sets `st.session_state['user']` or `['authenticated']`. The Logout button sets a flag nobody reads. Access control is presumably htpasswd in nginx in front of the app.
- **Several save paths silently lose data.** See "Bugs" in section 2.0.

---

## 1. app.py (91 lines)

**Page config**
- `st.set_page_config(page_title="Garden Data Orchestrator", layout="wide")`.
- No custom CSS anywhere in `app.py`. Inline HTML is only produced in `components.py` and `tab_json_pipeline.py`.
- `.streamlit/config.toml`: `headless=true`, `enableCORS=false`, `enableXsrfProtection=false`, `baseUrlPath="streamlit"`, `serverAddress="junsgarden.zapto.org"`, `gatherUsageStats=false`.

**Startup side effects.** These run on every Streamlit rerun, which means on every click:
1. `os.makedirs(IMAGE_DIR)`. `IMAGE_DIR` defaults to `/var/www/images/Database`.
2. `setup_database()`:
   - `ALTER TABLE data_source MODIFY COLUMN picture TEXT`.
   - Checks `INFORMATION_SCHEMA.COLUMNS` for 44 columns and adds any that are missing.
   - `CREATE OR REPLACE VIEW Annuals` (`plant_type='Annual'`) and `CREATE OR REPLACE VIEW Perennials` (`plant_type='Perennial'`).
3. `migrate_legacy_sizes()`: splits `size` into `plant_length` / `plant_width` where `plant_length` is empty.
4. `sync_existing_images()`:
   - Scans every row and the image folder.
   - For a row whose `picture` files don't exist on disk, it links files matching `_{id}.webp`, `_{id}.jpg`, `_{id}_manual.webp`, `_{id}_manual.jpg`, `_{id}_manual_`, or starting with the plant name with punctuation and spaces stripped, lowercased.
   - Writes `picture=<comma list>, image_verified=1`.
   - The name-prefix match can over-match ("rose" matches "rosemary…").

The Rust version should do steps 2–4 once, as a migration or startup job.

**Sidebar**
- `st.sidebar.header("⚙️ Settings")`
- `st.sidebar.markdown("👤 **Logged in as: {st.session_state.get('user','')}**")`. This is always blank.
- Button `"Logout"`: sets `session_state["authenticated"]=False`, then reruns. It has no effect.
- `text_input "Claude Model Name"`, default `"claude-sonnet-4-6"` (variable `selected_model`). It is passed to the Add Plant, Manual Review, Fill Missing, AI — Missing Only and JSON Pipeline tabs.

**Title:** `st.title("🌱 Jun & Kona's Garden Orchestrator")`

**Tabs, in order** (`st.tabs`; every tab body executes on every rerun):
1. `🌱 Add Plant`
2. `📝 Manual Review (AI)`
3. `✨ Fill Missing Data (AI)`
4. `✍️ Manual Editor`
5. `📸 Manual Image Replace`
6. `⚠️ Partials`
7. `🔍 Awaiting Approval`
8. `🏆 Fully Approved & Verified`
9. `🚩 Flagged for Review`
10. `🔒 AI — Missing Only`
11. `🔄 JSON Pipeline`
12. `🗺️ Bed Assignment`
13. `🌟 Wishlist`
14. `🏷️ Barcodes`

**Global session state:** `user` (read), `authenticated` (written). There are no global filters.

**config.py (177 lines)**
- Environment variables:
  - `DB_HOST` (default `junsgarden-db`), `DB_USER` [secret], `DB_PASS` [secret], `DB_NAME` (default `juns_garden_db`).
  - `DB_TABLE="data_source"`.
  - `OLLAMA_URL` (the old default pointed at a computer on the old home network), `OLLAMA_MODEL` (default `phi4:latest`).
  - `USE_LOCAL_AI` (default `"1"`).
  - `PERENUAL_KEY` [secret], `ANTHROPIC_API_KEY` [secret].
  - `IMAGE_DIR`.
- Option lists. The Rust version must reproduce them exactly; `""` means a leading blank option.
  - `PLANT_TYPES = ["", "Perennial", "Annual", "Biennial", "Shrub", "Tree", "Succulent", "Vine", "Unknown"]`
  - `EVERGREEN_OPTIONS = ["Evergreen", "Semi-evergreen", "Deciduous", "Deciduous Conifer", "Herbaceous", "Spring Ephemeral", "Summer Dormant"]`
  - `LEAF_SHAPES = ["", "Linear (Long, narrow)", "Lanceolate (Long tapering tip)", "Ovate (Egg-shaped)", "Elliptic (Oval tapering ends)", "Cordate (Heart-shaped)", "Palmate (Hand-like lobes)", "Deltoid (Triangular)"]`
  - `LEAF_TYPES = ["", "Simple (Single undivided blade)", "Pinnately Compound (Feather-like leaflets)", "Palmately Compound (Radiating leaflets)"]`
  - `LEAF_MARGINS = ["", "Entire (Smooth, no teeth)", "Serrated (Saw-toothed edges)", "Shallowly Serrate", "Lobed (Deep indentations)"]`
  - `FLOWER_SYMMETRIES = ["", "None", "Actinomorphic (Radial symmetry)", "Zygomorphic (Bilateral symmetry)"]`
  - `FLOWER_SHAPES = ["", "None", "Rotate (Flat wheel-shaped)", "Stellate (Star-shaped)", "Stellate (Chionodoxa)", "Cruciate (Cross-shaped)", "Campanulate (Bell-shaped)", "Infundibuliform (Funnel-shaped)", "Tubular (Cylindrical tube)", "Salverform (Tube opening flat)", "Bilabiate (Two-lipped)", "Urceolate (Urn-shaped)", "Trumpet/Cup-shaped (Daffodil)", "High-centered", "Double Blossoms"]`
  - `FLOWER_SIZES = ["", "None", "Tiny (<1 in)", "Small (1-2 in)", "Small to Large (1-5 in)", "Medium (2-4 in)", "Large (4-6 in)", "Massive (>6 in)"]`
  - `REPRODUCTION_OPTIONS = ["", "Hermaphrodite", "Monoecious", "Dioecious", "Polygamous", "Unknown"]`
  - `GROWTH_RATES = ["", "Slow", "Moderate", "Fast"]`
  - `ROOT_SYSTEM_OPTIONS = ["", "Taproot Systems", "Fibrous/Adventitious Systems", "Specialized Systems", "Unknown"]`
  - `ROOT_MORPHOLOGY_OPTIONS = ["", "Fine Roots (< 2 mm)", "Coarse Roots (> 2 mm)", "Mixed Fine & Coarse", "Unknown"]`
  - `ROOT_FUNCTIONAL_OPTIONS = ["", "Absorptive Roots (Lower Orders)", "Transport Roots (Higher Orders)", "Mixed Orders", "Unknown"]`
  - `INVASIVE_OPTIONS = ["", "No", "Yes", "Watchlist"]`
  - `NATIVE_STATUS_OPTIONS = ["", "Native", "Non-native", "Introduced", "Cultivar", "Unknown"]`
  - `FRUIT_EDIBILITY_OPTIONS = ["", "Edible", "Inedible", "Toxic", "Wildlife Only", "None"]`
  - `SUN_OPTIONS = ["", "Full Sun", "Partial", "Shade", "Full Sun - Partial", "Full Shade", "Full Sun - Full Shade", "Partial Shade - Full Shade", "Full Shade - Partial Shade"]`
  - `RESISTANCE_OPTIONS = ["", "Deer Resistant", "Rabbit Resistant", "Both", "Not Resistant"]`
  - `TOXICITY_OPTIONS = ["", "Non-toxic", "Toxic", "Toxic to Dogs/Cats", "Toxic to Humans", "Toxic to Humans & Pets", "Unknown"]`
  - `YEARS_ADDED = ["2022" … "2027"]`
  - `FERTILIZER_OPTIONS = ["Azalea-tone", "Berry-tone", "Bio-tone", "Bloom Booster", "Bulb-tone", "Citrus-tone", "Evergreen-tone", "Flower-tone", "Garden-tone", "Holly-tone", "Iron-tone", "Palm-tone", "Plant-tone", "Rose-tone", "Tomato-tone", "Tree-tone", "None"]`
  - `GARDEN_BEDS` (24 entries; uses curly apostrophes ’): `"To Be Sorted", "All Animals Go to Heaven", "Annuals", "Aunt Sharon’s Garden", "Barwaz Garden", "Boo Boo’s Garden", "China Town Garden", "Dead or Removed", "Grandma Emily Barwaz Garden", "Grandpa Duke of Earl Middleton Garden", "Jason’s Terrace", "Jimmy John’s Blue’s Garden", "Karta’s Corner Garden", "Ko-Ja Garden", "Kona’s Garden", "Korea Garden", "Orchard Grove", "River Garden", "Sister Elizabeth’s Garden", "The 4 Season Garden", "The Royal Run", "Uncle Dennis Garden", "Want (Wishlist)", "Zen Garden"`
- Helpers:
  - `parse_m(str)`: 12-character bitmap → list of month numbers (1–12); any character other than `.` counts as on.
  - `format_m(list)`: list → `"JFMAMJJASOND"` with `.` for off months; returns `"............"` if the input is not a list.
  - `get_month_range(start_abbr, end_abbr)`: handles wrapping past December.
  - `MONTHS_MAP` / `INV_MONTHS`.
- `get_missing_fields(p)`. Used by Partials, Awaiting, Approved, the JSON Pipeline and Gemini. It checks 56 required fields:
  - `botanical_name, company, family, genus, species, plant_type, plant_length, plant_width, spacing, zones, sun_exposure, water_needs, soil_ph, soil_type, bloom_color, foliage_color, stem_color, attracts, deer_resistance, toxicity, care_notes, maintenance_notes, cleanup_notes, disease_susceptibility, extra_details, fertilizer, propagation_method, cuttings, picture, grow_months, bloom_months, leaf_shape, leaf_type, leaf_margin, flower_symmetry, flower_shape, flower_size, fruit_type, fruit_shape, fruit_color, fruit_size, fruit_edibility, invasive_local, native_status, growth_rate, planting_depth, chilling_hours, garden_beds, aromatics, is_evergreen, life_expectancy, reproduction_type, root_system, root_morphology, root_function, average_cost`.
  - A field is missing if its value is blank or one of `''`, `unknown`, `unknown only`, `[]`, `............`.
  - `average_cost` is also missing if it parses to 0.
  - If `bloom_color` (lowercased) is one of `not ornamental`, `none`, `insignificant`, `non-flowering`, `inconspicuous`, then `bloom_months` and `flower_symmetry`/`flower_shape`/`flower_size` are not required.
  - `NONE_OK_FIELDS` and `_NONE_OR_ZERO` are defined, but the code treats "none"/"0" as filled for every field anyway.

---

## 2.0 Shared components (used by most tabs)

### `components.render_plant_form(p, data_dict, idx_prefix, is_ai=False, new_fields=None, changed_fields=None)` (≈540 lines)

`p` is the original database row and `data_dict` holds the values shown (the database row or the AI result). Widget keys are `{idx_prefix}_<suffix>`.

**Label decoration** (only when `is_ai`; never for quantity, year_added or garden_beds):
1. Field in `new_fields` → `"✨ {label} (New)"`
2. Field in `changed_fields` → `"🔄 {label} (Was: {old[:20]})"`
3. Field has any value in `data_dict` → `"✨ {label} [AI: {val[:20]}]"`

**Selectbox options** (`get_safe_opts`): the current value goes first. If the value is not in the list, it is prepended so legacy values survive.

**Widgets in order**

- **Section 1** — `#### 🧬 Section 1: Taxonomy & Core Identity` (4 columns)
  - text "Botanical Name" (`_b`)
  - text "Company / Brand" (`_cmp`)
  - text "Family" (`_f`)
  - text "Genus" (`_g`)
  - text "Species" (`_s`)
  - select "Plant Type" (`PLANT_TYPES`, default `Perennial`, `_pt`)
  - select "Native Status" (`_nat`)
- **Section 2** — `#### 📏 Section 2: Growth Metrics, Spacing & Beds`
  - text "Target Length / Height" (`_len`)
  - text "Target Width / Spread" (`_wid`)
  - text "Life Expectancy" (`_life`)
  - text "Plant Spacing Limits" (`_spc`)
  - text "Global Species Hardiness Zones spectrum" (`_z`)
  - select "Growth Rate Assumption" (`_grate`)
- **Section 3** — `#### 🌦️ Section 3: Microclimatic Profiles & Soil Requirements`
  - select "Sun Exposure Requirement" (`_su`)
  - text "Hydration Needs" (`_w`)
  - text "Soil Target pH Scale" (`_sph`)
  - text "Soil Consistency Profile" (`_styp`)
  - text "Planting Depth Requirement" (`_pdep`)
  - text "Chilling Hours Requirement" (`_chill`)
- **Section 4** — `#### 💵 Section 4: Investment & Procurement Logistics`
  - number "Estimated 1-Gallon Regional Retail Cost ($)" (float, `_c`)
  - number "Quantity Planted (Manual Override Only)" (min 0; default from `p.qty`, falling back to `p.quantity`, then 1; `_qty`)
  - multiselect "Year(s) Added to Garden (Manual Override Only)" (options = sorted `YEARS_ADDED` ∪ existing; default parsed from `p.year_planted` split on " / ", falling back to `year_added` split on ","; `_yrs`)
  - text "Year Plant Died (optional, leave blank if alive)" (placeholder "e.g. 2024", `_died_year`; saved as int or NULL)
- **Section 5** — `#### 📍 Section 5: Extended Human Bed Location Manager`
  - multiselect "Assign Plant to Bed (AI Prohibited Field)" (options `GARDEN_BEDS`; default from `p.garden_beds` filtered to known beds, otherwise `["To Be Sorted"]`; `_beds`)
- **Section 6** — `#### 🌿 Section 6: Foliage & Stem Morphologies`
  - text "Foliage Color Spec" (`_fc`)
  - text "Stem Color Spec" (`_stc`)
  - multiselect "Foliage Retention (Tags)" (`EVERGREEN_OPTIONS` ∪ existing, `_evg`)
  - select "Leaf Structural Shape" (`_lshape`)
  - select "Leaf Blade Type" (`_ltype`)
  - select "Leaf Margin / Edge Profile" (`_lmargin`)
- **Section 7** — `#### 🌸 Section 7: Flower, Scent & Reproduction`
  - text "Flower/Bloom Color" (`_bc`)
  - text "Aromatics / Scent Descriptors" (`_arom`)
  - select "Flower Symmetry Layout" (`_fsym`)
  - select "Flower / Corolla Shape" (`_fshape`)
  - select "Flower Size Class" (`_flsz`)
  - select "Reproduction Mechanism" (`_repro`)
- **Section 8** — `#### 🍒 Section 8: Fruit & Berry Yields`
  - text "Fruit/Berry Type" (`_fruit`)
  - text "Fruit Structural Shape" (`_fshp`)
  - text "Fruit Color" (`_fcol`)
  - text "Fruit Dimension Size" (`_fsz`)
  - select "Fruit Edibility Class" (`_fedib`)
- **Section 9** — `#### 🪹 Section 9: Subterranean Anatomy & Root Systems`
  - select "Primary Root Structure" (`_rt_sys`)
  - select "Root Dimensional Morphology" (`_rt_mor`)
  - select "Root Order & Function Limits" (`_rt_func`)
- **Section 10** — `#### 🛡️ Section 10: Care, Hazards, Propagation & Companions`
  - text "Attracts Pollinators/Wildlife" (`_att`)
  - select "Deer/Rabbit Pest Resistance" (`_dr`)
  - select "Toxicity Verification Classification" (`_tox`)
  - select "Local Invasive Status" (`_inv`)
  - text "Propagation Method" (`_prop`)
  - text "Cuttings Instructions" (`_cut`)
  - multiselect "Assigned Companion Plant Tokens" (options = all `data_source` `plant_name` values ∪ current, sorted; `_comp`)
  - multiselect "Assigned Fertilizer Regimens" (`FERTILIZER_OPTIONS`, `_fert`)
- **Section 11** — `#### 📅 Section 11: Growth Calendars`
  - Toggle "🌸 Multi-Period Mode (non-contiguous months)" (default off; help "Use this for plants that bloom in separate periods, e.g. Apr–May then Jul–Oct"; `_multi`).
  - **Grow column.** Range mode: warning "⚠️ Grow months not set." when empty, then selects "Vegetative Grow Start" / "Vegetative Grow End" (Jan–Dec; `_gs`, `_ge`). Multi mode: markdown "**🌱 Grow Months** — tick each active month" and 12 checkboxes Jan…Dec (`_gm0`…`_gm11`).
  - **Bloom column.** Checkbox "🚫 Does not bloom" (default on if `bloom_color` is `not ornamental`/`none`/`insignificant`; help "Mark flowerless (conifer / fern / foliage-only). Bloom months are no longer required."; `_nobloom`). When ticked: `bloom_months=[]`, `bloom_color='Not ornamental'`, caption "🌲 Flowerless — bloom months cleared and no longer required."
  - Otherwise, range mode: warning "⚠️ Bloom months not set." and selects "Reproductive Bloom Start" / "Reproductive Bloom End" (`_bs`, `_be`). Multi mode: "**🌸 Bloom Months** — tick each active month" with 12 checkboxes (`_bm0..11`).
  - HTML preview bar. Dark box `#1e1e1e`. Each on-month is a 24px coloured cell: grow `#2e7d32`, bloom `#d81b60`. Labels are "🌱 Grow" and "🌸 Bloom", followed by the bitmap in `<code>`.
  - Caption "↑ Preview matches the website card. The `code` shows the DB bitmap value that will be saved."
- **Section 12** — `#### 📝 Section 12: Long-Form Maintenance Journals & Search Overrides`
  - Text areas (height 100): "Seasonal Care & Watering Notes" (`_cn`), "Pruning & Fertilizer Notes" (`_mn`), "Winterization & Cleanup Notes" (`_cln`), "Disease & Insect Vulnerabilities" (`_dis`), "Extra Details / Culinary Edibility Profile" (`_ex`).
  - If `is_ai`: text "Engine Image Search Target Override" (`_img_st`).
  - Checkbox "⚠️ Flag Entry for Further Technical Review" (value from `p.needs_review`; `_rev`).
- **Section 13** — `---` then `### 🖼️ Section 13: Digital Asset Gallery Management`
  - The gallery is `picture` followed by the `additional_pictures` comma list.
  - If there are images: "**Active Vault Graphics:**", then up to 4 columns, each with `st.image` (width 300, caption = filename) or error "Missing File: {f}".
  - Each image has button "🗑 *Wipe File from Vol*" (`_purge_img_{i}`). It **deletes the file from disk** (`os.remove`), runs `UPDATE data_source SET picture=%s, additional_pictures=%s WHERE id`, shows success "🗑 '{f}' removed from gallery.", then reruns.
  - If there are no images: info "📁 No images are currently tracked or linked for this record container row."
  - If `is_ai`: button "🌐 Fetch & Stage Image from Web Search" (`_fetch_ai_img_btn`):
    - Spinner "Searching Wikipedia and DuckDuckGo indexes...".
    - Calls `fetch_plant_image` and saves `{SafeName}_{id}_{epoch}.webp`.
    - Success "Staged image! Approve below or click Save Changes to commit." or error "No suitable imagery found online."
  - File uploader "Upload custom photo(s) to gallery collection" (jpg, jpeg, png, webp; multiple; `_upl`).
  - If uploads exist and the gallery is non-empty: radio "Upload Processing Engine Action" with options ["Append to existing photos", "Replace all existing photos"] (horizontal, `_up_act`).
- **Section 14** — `---` then `### ✅ Section 14: Validation Signing Authority`
  - Toggle "Author Signature Sign-Off" (`_jas_app`).
  - Toggle "Jun Rae Signature Sign-Off" (`_jun_app`).
- **Primary full-width button** "Save Data & Process Uploads" if there are uploads, otherwise "Save Changes" (`_save_btn`). Spinner "Processing & Saving...". It then:
  1. Writes each upload to `{SafeName}_{id}_manual_{epoch+i}.webp`. WebP files are written as-is; other formats are converted to WebP quality 87.
  2. Re-reads `SELECT picture, additional_pictures`.
     - Append mode keeps `picture` (or uses the first new file if there is none) and puts the rest into `additional_pictures`.
     - Replace mode makes the first new file the `picture` and the rest `additional_pictures`.
     - Sets `image_verified=1`.
  3. Joins list fields with ", ". Builds `year_planted = " / ".join(years)` and `qty = quantity`.
  4. Calls `save_to_db(id, data_dict)`.
  5. Read-back check: `SELECT grow_months, bloom_months`. Shows "✅ Verified — Grow: `{g}` | Bloom: `{b}` — both saved & will display on website." or the partial/empty warnings.
  6. Shows "Successfully committed to target instances!", sleeps 1 s, reruns.

**Bugs in the shared form (important for parity decisions)**
- **`year_planted`, `qty` and `additional_pictures` are never saved.** `save_to_db` does not include them.
  - Edits to Quantity are written to `quantity`, but the form reads `qty` first, so the edit appears not to stick.
  - Year edits go to `year_added`, but the form reads `year_planted` first.
  - Images uploaded in Append mode land in `additional_pictures`, so they are written to disk and then lost from the database.
- **"Fetch & Stage Image" never persists.** The staged `data_dict['picture']` is lost on rerun, and Save without uploads restores `p.picture`. The file is orphaned on disk.
- **Existing values are mislabelled as AI.** In AI mode, every non-empty existing value is labelled "✨ … [AI: …]", even human-entered values.
- **Wipe File deletes the file.** "Wipe File from Vol" removes the file from disk; the image_manager remove buttons only unlink it in the database.

### `save_to_db(plant_id, data)`
- One statement: `UPDATE data_source SET <63 fields>, grow_months, bloom_months WHERE id`.
  - The fields are: `company, family, genus, botanical_name, species, plant_type, sun_exposure, cleanup_notes, care_notes, cuttings, fertilizer, maintenance_notes, attracts, zones, water_needs, deer_resistance, soil_ph, soil_type, toxicity, bloom_color, foliage_color, stem_color, disease_susceptibility, propagation_method, plant_length, plant_width, average_cost, extra_details, image_search_term, spacing, leaf_shape, leaf_type, leaf_margin, flower_symmetry, flower_shape, flower_size, fruit_type, fruit_shape, fruit_color, fruit_size, fruit_edibility, invasive_local, native_status, growth_rate, planting_depth, chilling_hours, companion_plants, approved_jason, approved_jun, needs_review, garden_beds, aromatics, picture, image_verified, is_evergreen, quantity, year_added, life_expectancy, reproduction_type, root_system, root_morphology, root_function, died_year`.
- Coercion: booleans become 0/1; `quantity` becomes an int (default 1); `died_year` becomes an int or NULL; lists are joined with ", "; dicts become JSON; months go through `format_m`.
- If the error mentions `died_year`, it retries without that column.
- Then a second statement mirrors the beds: `UPDATE data_source SET flower_bed=<garden_beds> WHERE id`. The comment says the website reads `flower_bed`.

### `components.render_quantity_logger(plant_id, plant_name, key_prefix)`
- Expander "📊 Log a Quantity Event (optional)" (collapsed).
  - Caption "Record a purchase, loss, or removal for this plant. Updates the current qty automatically."
  - Select "Event type" with values `purchased|died|removed|other`, displayed as "🛒 Purchased / Added", "💀 Died", "🗑️ Removed", "📝 Other".
  - Number "How many plants" (min 1, default 1).
  - Select "Week": the last 172 weeks, Monday-based. Labels are "This week  (Mon d – Sun d)", "Last week  (…)", otherwise "Mon d – Mon d, YYYY".
  - Text "Notes (optional)" (placeholder "e.g. bought from Hewitt's").
- Button "💾 Save Quantity Event":
  - `INSERT INTO plant_quantity_log (plant_id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by='orchestrator')`. `event_week = %W + 1`.
  - `UPDATE data_source SET qty = GREATEST(0, qty + delta)`, where delta is + for purchased and − otherwise.
  - If `qty` is now 0 and the event was died/removed: `UPDATE data_source SET died=1`.
  - Success "✅ Logged: {type} ×{n} for **{name}** (week of …)".
  - If the table is missing: warning "⚠️ Quantity log table not found — run the migration first: `tools/plant_tracking_migration.sql`".
  - Uses `%-d` in `strftime`, which is Linux-only.

### `image_manager.render_image_manager(plant, key_prefix)` (307 lines; keys `{prefix}_{pid}_*`)
- A toast banner from `session_state[{kp}_toast]`.
- **Left column (1/3)**
  - Each file in the `picture` comma list (it ignores `additional_pictures`): image (width 280, caption = filename), or "⚠️ File not found: {f}" / "⚠️ Could not display `{f}` — file may be corrupted.".
  - Per-image button "🗑️ Remove this image": `UPDATE data_source SET picture=<remaining or NULL>, image_verified=<1 if remaining else 0>`. Toast "🗑️ Removed `{f}`" (+ " — gallery now empty.").
  - If verified: "❌ Mark as Unverified" → `SET image_verified=0`. Otherwise: "✅ Gallery Looks Good! (Mark as Verified)" → `SET image_verified=1`.
  - "🗑️ Wipe ALL images" opens a confirmation: warning "⚠️ Wipe ALL images for **{name}**? Cannot be undone." with "✅ Yes, wipe" (`SET picture=NULL, additional_pictures=NULL, image_verified=0`; toast "🗑️ All images wiped.") and "❌ Cancel".
  - With no images: info "No images yet — use the options on the right."
- **Right column (2/3)**
  - Header line "**{name}**  |  Botanical: {b or N/A}  |  Family: {f or N/A}".
  - `### Option 1: Fetch from Web`, caption "Searches Wikipedia first, then DuckDuckGo — no API key."
    - Text "Search term:" (default `image_search_term` or `plant_name`).
    - "🔍 Fetch Image Preview" calls `fetch_free_image`, resizes to max 900 px and stores WebP q87 bytes in session. Error if nothing found: "No image found — try a different search term."
    - Preview `#### 🖼️ Preview — does this look right?` (width 340).
    - "✅ Confirm & Save" writes `{Safe}_{pid}_{epoch}.webp`, appends it to `picture`, sets `image_verified=1`, and runs `UPDATE … SET image_search_term`. Toast "✅ Saved `{f}`".
    - "❌ Discard".
  - `### Option 2: Upload Your Own`
    - Uploader "Upload photos (multiple allowed)" (jpg, jpeg, png, webp; versioned key).
    - Radio "Upload action:" with ["Append to current gallery", "Wipe gallery and replace"], reversed order if there are no pictures.
    - Preview `#### 🖼️ Upload Preview` (width 220).
    - "✅ Process & Save": files `{Safe}_{pid}_manual_{epoch+i}.webp`, then `UPDATE picture=<list>, image_verified=1`. Toast "✅ {n} image(s) saved: …".
- **Inconsistency:** this widget stores every image comma-joined in `picture`, while `render_plant_form` treats `picture` as one image plus `additional_pictures`.

### 2.1 Tab 1 — `🌱 Add Plant` (`tab_add_plant.py`, 700 lines)

**Purpose:** create a new `data_source` row (or promote a wishlist plant), either by a quick stub or by AI enrichment.

**Access gate**
- Identity comes from session keys `email`, `user_email`, `user`, `username`.
- Allowed people: the two accounts of the old app (in your copy: GARDEN_USERS in .env).
- Since no identity is ever set, the tab always shows info "Enter your account email to add plants (Jun or the author only)." and text "Account email" (placeholder "you@gmail.com", key `apt_gate_email`).
- Typing an allowed email unlocks the tab. **There is no password**, so this is only a speed bump.
- Blocked: error "🔒 Adding plants is restricted to **Jun** or **the author**." or "That email isn't authorized to add plants."

**Widgets in order**
- Header "🌱 Add a New Plant".
- Expander "📋 How to read a plant tag — click to expand". It shows `static/plant_tag_guide.jpg` if present (it isn't in the folder). Otherwise a markdown table (Tag field / What it means) and the caption "💡 To show the visual photo here, save the image as `plant_orchestrator/static/plant_tag_guide.jpg` on the server."
- If `apt_new_id` is set, the tab jumps to the edit phase (below).
- **Step 1** — subheader "Step 1 · Name the Plant"
  - Text "Common / Display Name *" (placeholder "e.g.  Dwarf Hummingbird Mint", `apt_name_input`).
  - Text "Botanical Name *" (placeholder "e.g.  Agastache cana 'Sinning'", `apt_botanical_input`).
  - If either is blank: info "Enter both a common name and a botanical name to continue."
- **Step 2** — subheader "Step 2 · Check for Duplicates"
  - Button "🔍 Search Database for Existing Match" (secondary, full width).
    - `SELECT id, plant_name, botanical_name, family, garden_beds FROM data_source WHERE botanical_name LIKE %b% OR plant_name LIKE %c%`.
    - `SELECT id, plant_name, botanical_name, company, family, garden_beds FROM future_plants WHERE …` (same LIKE conditions).
    - Stores `apt_db_matches` and `apt_db_checked`.
  - Before searching: caption "Click **Search Database** above before continuing."
  - Garden matches: warning "⚠️  **{n} match(es) already in your garden database.** Review them carefully before adding a duplicate." plus bullets "- **[ID x]  name**  ·  _botanical_  ·  Beds: …".
  - Wishlist matches: info "🌟 **{n} match(es) found in your Wishlist (future_plants).** You can promote one directly instead of creating a new record."
    - Each match has a row and a checkbox "Promote to garden" (`apt_promote_{id}`).
    - Ticking it shows button `🌱 Confirm: Move "{name}" to garden database`, which calls `promote_future_plant(id)`. Success "✅ **{name}** promoted to garden! New ID: {id}", then clears the cache and resets the check. Failure shows "Failed: …".
  - No matches: "✅  No matches found — this plant is new!"
  - Garden matches only: checkbox "✅  I've reviewed the matches — this is a genuinely different plant, continue." (`apt_dup_override`).
  - Matches in both tables: checkbox "✅  I've reviewed all matches — continue adding as a new plant."
- **Step 3** — subheader "Step 3 · How would you like to add this plant?"
  - Radio "Choose a path:" with ["⚡ Quick Create — enter basics now, complete data later via JSON", "✨ AI Enrich — auto-fill fields with Claude + Perenual + Wikipedia"] (horizontal, `apt_path_choice`).
- **Quick Create path**
  - Select "Year Added" ([""] + `YEARS_ADDED`).
  - Multiselect "Garden Bed(s)" (`GARDEN_BEDS`).
  - Number "Quantity" (min 0, default 1).
  - Button "📋 Create Stub & Download Schema":
    - `INSERT INTO data_source (plant_name, needs_review) VALUES (%s,1)`.
    - `UPDATE data_source SET botanical_name, year_planted, garden_beds, qty WHERE id`. **It does not set `flower_bed`.**
    - Builds a schema JSON object with `_id`, `_plant_name`, `_note`, `_missing_fields` and about 60 keys (including `local_store`, `year_planted`, `qty`).
- **AI path**
  - Markdown "**Claude + Perenual + Wikipedia** will research…".
  - Button "✨ Fetch Plant Data with AI": spinner "Researching **{name}** via Claude + Perenual + Wikipedia…  This may take 20–40 seconds." It calls `fetch_location_context`, then `fetch_perenual_data`, then `ask_claude_ai(skeleton, model, allow_proposals=True)`.
  - Button "⏭ Skip AI" (help "Skip AI and fill everything manually in the form.").
  - Results: success "✨  AI pre-filled **{n}** field(s) — review them in the form below." plus expander "Fields suggested by AI" (dataframe with columns Field / AI Suggestion), or info "AI returned no suggestions — you'll fill everything in manually."
- **Step 4** — subheader "Step 4 · Create the Record"
  - Button "📋 Create Record →": inserts the stub, then `UPDATE data_source SET <every AI key>`. AI keys are not whitelisted, so an unexpected key makes the UPDATE fail and leaves an orphan stub. Empty lists get written as "".
- **Edit phase**
  - Success "✅ ok — **{name}** added (ID {id}). All fields saved."
  - Button "➕ Add Another Plant" deletes every `apt_*` key.
  - Toggle "✏️ Open full editor for this plant" (`apt_show_full`). When on:
    - Caption "Fields pre-filled by AI are marked ✨.  Garden beds, company, quantity, year and cost are yours to set manually."
    - If created by Quick Create: download "⬇️ Download JSON Schema" (`{name}_schema.json`) and "📖 Download Field Reference Guide". `plant_schema_reference.txt` is missing from the folder, so the guide downloads as an empty file. Info "💡 Fill in the blank fields in the JSON, then head to **🔄 JSON Pipeline → 📥 Phase 3 — Import & Review** to bring the data in."
    - Expander "📋 Export missing-fields JSON  ({n} field(s) still empty)" with "⬇️ Download missing-fields JSON" (`{name}_new_plant_missing.json`), or "🎉 All required fields are filled — no missing data!"
    - `render_plant_form(plant, plant, f"add_{id}", is_ai=True, new_fields=ai_fields)` and `render_quantity_logger`.
    - Footer caption.
- **Not importable:** the schema and missing-fields downloads are JSON objects, not arrays, so JSON Pipeline Phase 3 rejects them.

**Session state:** `apt_*` (`name_input`, `botanical_input`, `db_matches`, `db_checked`, `promote_{id}`, `promote_ready_{id}`, `dup_override`, `path_choice`, `qc_year`, `qc_beds`, `qc_qty`, `new_id`, `ai_data`, `ai_fields`, `ai_done`, `qc_schema_id`, `qc_schema_data`, `show_full`, `gate_email`) plus `email`.

**AI / web:** `ask_claude_ai` (Ollama phi4 by default, else Anthropic), Perenual, DuckDuckGo text, and Wikipedia/Wikimedia/DuckDuckGo images through the form.

### 2.2 Tab 2 — `📝 Manual Review (AI)` (`tab_manual_review.py`, 58 lines)
- **Purpose:** AI-fill plants that have no botanical name, then review and save them in the form.
- Header "Search & Review Data (AI Only)".
- Button "Search Next 3 Plants" (primary): `SELECT * FROM data_source WHERE botanical_name IS NULL OR TRIM(botanical_name)='' LIMIT 3`, stored as `to_fetch_tab1`.
- Processing loop: one plant per rerun. Caption "⏳ {n} plants in queue...", status "🤖 Fetching {name} via {selected_model}..." (misleading when the local model is used). Calls `fetch_location_context`, `fetch_perenual_data` and `ask_claude_ai(allow_proposals=False)`, then appends to `batch_tab1`.
- Per plant: expander "⚠️ {name}" or "✅ {name}" (expanded).
  - On error: error text plus warning "Raw Output: …".
  - Otherwise: download "📄 Export this result as HTML (open in browser → Print → Save as PDF)" (`{name}_AI_Review.html`, `generate_plant_html`), then `render_plant_form(p, llm, f"t1_{i}", is_ai=True)`.
- **Session state:** `batch_tab1`, `to_fetch_tab1`.
- **Bugs:**
  - There is no ORDER BY and no skip. If a plant is not saved, or the AI leaves `botanical_name` blank, the same plant returns every time.
  - `fertilizer`, `companion_plants`, `is_evergreen` are excluded from the "keep existing" rule, so AI values replace existing ones in the form.

### 2.3 Tab 3 — `✨ Fill Missing Data (AI)` (`tab_fill_missing.py`, 91 lines)
- **Purpose:** AI-fill partially filled plants. The AI may also propose changes to existing values.
- Header "✨ Fill Missing Data (Review & Approve)".
- Button "Find 3 Partially Filled Plants": `SELECT * WHERE` any of `plant_length`, `plant_width`, `spacing`, `soil_ph`, `bloom_color` is empty, or `company` is empty/'unknown'/'unknown only'; `LIMIT 3`.
- Loop status "🤖 Finding missing data for {name}...". Calls `ask_claude_ai(allow_proposals=True)`, then computes:
  - `new_fields`: the old value was empty (`''`, `unknown`, `unknown only`, `[]`, `............`) and the new value is not.
  - `changed_fields`: `{field: old}` where both are non-empty and different.
- Expander "{⚠️|✨} {name} - {n} New, {m} Proposed".
  - Download "📄 Export this result as HTML  ({n} new · {m} changed)  — open in browser → Print → Save as PDF" (`{name}_Fill_Review.html`).
  - Form prefix `t2_{i}`, with `new_fields` and `changed_fields`.
- **Session state:** `batch_tab2`, `to_fetch_tab2`. It has the same re-selection issue as Tab 2.

### 2.4 Tab 4 — `✍️ Manual Editor` (`tab_manual_editor.py`, 72 lines)
- **Purpose:** pick any plant and edit it manually. No AI.
- Header "✍️ Global Manual Editor", text "Instantly find and edit any plant manually. No AI involved."
- `SELECT * FROM data_source ORDER BY plant_name ASC`.
- Select "Search database for a plant to edit:" with options "{name} (ID: {id})".
- Related plants: `SELECT id, plant_name, garden_beds FROM data_source WHERE botanical_name=%s AND id!=%s`, or by `genus` if there is no botanical name. Shown under `### 🌿 Related Plants in Database` in a disabled text area "Botanical Matches (Read Only)" with lines "• name (ID: x) - Location: beds", or "No other plants found with this Botanical Name or Genus."
- `### ✏️ Edit Active Plant`, download "📄 Export current data as HTML (open in browser → Print → Save as PDF)" (`{name}_Manual_Export.html`).
  - Bug: the export is generated before `parse_m`, so grow/bloom show "—".
- Form prefix `man_ed_{id}`, then the quantity logger (prefix `man_ed`).

### 2.5 Tab 5 — `📸 Manual Image Replace` (`tab_image_replace.py`, 85 lines)
- **Purpose:** filter plants by image status and manage their gallery.
- Header "📸 Manual Image Replacement Gallery".
- Radio "Show plants:" with ["All plants", "Unverified images", "Verified images", "No image yet", "Only one image"].
- Text "🔍 Filter by name / family / botanical name:" (placeholder "Start typing…"). Filtering is client-side over name, botanical name and family.
- Query: `SELECT id, plant_name, botanical_name, picture, additional_pictures, image_search_term, family, zones, image_verified FROM data_source`, with:
  - Verified: `image_verified=1`
  - Unverified: picture non-empty and not 'None', and `image_verified` 0 or NULL
  - No image: picture NULL, empty or 'None'
  - Only one image: picture non-empty, `NOT LIKE '%,%'`, and `additional_pictures` empty
  - All results `ORDER BY plant_name`
- Select "Select a plant ({n} shown):" with options "{name}  [ID x]  ✅|📷|❌ no image" (`img4_plant_select`).
- Then `render_image_manager(p, "img4")`.
- Empty result: info "No plants match that filter."

### 2.6 Tab 6 — `⚠️ Partials` (`tab_partials.py`, 119 lines)
- **Purpose:** list plants missing required fields and edit them.
- Header "⚠️ Incomplete Datasets (Partials)", text "Plants missing vital fields required for the catalog."
- `SELECT * FROM data_source`, then `get_missing_fields` in Python. A base64 80×80 JPEG thumbnail is generated per row on every rerun.
- Warning "Found {n} plants missing core data." and a dataframe with columns `id, Thumbnail` (ImageColumn "Image"), `plant_name, genus, species, Missing Data`.
- Subheader "Edit a Partial Plant", select "Select plant to manually edit:".
- Three buttons:
  - "📄 Export as HTML" (`{name}_Partial_Export.html`).
  - "📋 Export Missing Fields as JSON" (`{name}_missing_fields.json`; an object containing `_missing_fields`, `_note` and the full row; the help text wrongly claims "only the missing fields").
  - Link button "📥 Go to JSON Import →" with url `?tab=json_pipeline#phase3`. **This is broken**; nothing reads the query parameter.
- Info text, then form prefix `part_ed`, quantity logger `part_ed`, subheader "📸 Image Management" and `render_image_manager(act_p, "part_img")`.
- None found: "No partials found! Everything is complete."
- Bug: the static form prefix means widget state can carry over when switching plants. Tabs 4 and 10 avoid this by putting the id in the prefix, and their comments describe this exact problem.

### 2.7 Tab 7 — `🔍 Awaiting Approval` (`tab_awaiting_approval.py`, 74 lines)
- **Purpose:** complete plants not yet approved by both the author and Jun.
- Header "🔍 Awaiting Approval", text "These plants have 100% of their data complete, but are waiting on a sign-off from the author and/or Jun."
- Sub-tabs "🌷 Perennials" (`SELECT * FROM Perennials`) and "🌻 Annuals" (`SELECT * FROM Annuals`). Filter: no missing fields and not (`approved_jason` AND `approved_jun`).
- Dataframe columns: `Thumbnail, plant_name, botanical_name, approved_jason, approved_jun`.
- Select "Review & approve perennial:" (`await_peren_sel`) / "Review & approve annual:" (`await_annual_sel`).
- Form `await_p` / `await_a` plus quantity logger.
- Empty: "All complete perennials have been fully approved." / "All complete annuals have been fully approved."
- Shrubs, Trees, Vines and other types never appear here, or in Tab 8.

### 2.8 Tab 8 — `🏆 Fully Approved & Verified` (`tab_fully_approved.py`, 74 lines)
- Header "🏆 Fully Approved & Verified", text "Browse the 100% completed, print-ready catalog. Both the author and Jun have verified this data."
- Same Perennials/Annuals sub-tabs, filtered to complete and approved by both.
- Columns: `Thumbnail, plant_name, botanical_name, genus, species, plant_length, spacing`.
- Selects "Edit fully approved perennial:" (`comp_peren_sel`) / "Edit fully approved annual:" (`comp_annual_sel`). Form `comp_p` / `comp_a` plus logger.
- Empty: "No 100% approved perennials found yet." / "No 100% approved annuals found yet."

### 2.9 Tab 9 — `🚩 Flagged for Review` (`tab_flagged.py`, 40 lines)
- Header "🚩 Flagged for Review", text "Plants manually marked with the 'Needs Review' flag."
- `SELECT * FROM data_source WHERE needs_review = 1`. Dataframe columns: `id, Thumbnail, plant_name, botanical_name, needs_review`.
- Subheader "Edit a Flagged Plant", select "Select flagged plant to review:", form `flag_ed` plus logger.
- Empty: "No plants are currently flagged for review."

### 2.10 Tab 10 — `🔒 AI — Missing Only` (`tab_ai_safe_fill.py`, 285 lines)
- **Purpose:** one plant at a time, the AI fills only fields that are completely empty.
- Header "🔒 AI — Missing Fields Only". Info box (icon 🛡️): "**Strict read-protect mode.** The AI can only fill fields that are completely empty. Any field that already has a value is skipped — the AI never overwrites or proposes changes to existing data. Plants are reviewed **one at a time**. Save or discard, then fetch the next one."
- Button "🔍 Find Next Plant with Missing Fields", which reads "➡️ Next Plant" once a plant is loaded.
  - `SELECT * FROM data_source WHERE` any of `botanical_name, family, genus, species, plant_length, plant_width, spacing, soil_ph, bloom_color, sun_exposure, water_needs, foliage_color, zones` is empty, or `company` is empty/unknown; `LIMIT 1` with no ORDER BY.
  - **Stuck bug:** the same plant comes back forever if it isn't saved or can't be filled. The "discard" mentioned in the info box does not exist.
- Status "🔒 Safely filling missing fields for **{name}**..." calls `ask_claude_ai(allow_proposals=False)`, then merges only where the original value is empty (`''`, `null`, `[]`, `............`, `unknown`, `unknown only`).
- If `companion_plants` is still empty: status "🔗 Linking companion plants for **{name}** from your database..." calls `_suggest_companions_from_db`.
  - **Always calls Anthropic directly** (`anthropic.Anthropic`, sidebar model, `max_tokens` 512), even when the local model is on. With no key it silently returns [].
  - The prompt lists every `data_source` `plant_name` and asks for up to 8 exact names, one per line, or "NONE". Results are cross-validated case-insensitively.
- No plants left: success "🎉 No more plants with missing fields were found in the database!" plus "🔄 Check Again".
- On error: error "AI error for **{name}**: …" plus warning "Raw output: …".
- Display: subheader "🔒 {name}", "📄 Export as HTML" (`{name}_SafeFill.html`), and either success "✅ AI filled **{n}** previously empty field(s): `a`, `b`" or warning "The AI found no additional missing data to fill for this plant. You can still edit fields manually below, or click **Next Plant** to skip."
- Caption "🛡️ Fields with existing data were locked — the AI could not touch them. Only green-highlighted fields were filled by the AI."
- Form `t9_{id}`, `is_ai=True`.
- **Session state:** `tab9_plant`, `tab9_done`.

### 2.11 Tab 11 — `🔄 JSON Pipeline` (`tab_json_pipeline.py`, 1381 lines; the largest)
- Header "🔄 JSON Export / Enrich / Import Pipeline". Info: "**Four-phase workflow.**  **Phase 1** exports one plant at a time.  **Phase 2** bulk-exports up to 5 plants as one JSON for manual editing.  **Phase 2 Enrich** runs Claude + Perenual AI in-app.  **Phase 3** uploads any JSON and lets you review every change before committing."
- Sub-tabs: "📤 Phase 1 — Export", "📦 Phase 2 — Bulk Export 5", "🤖 Phase 2 — Batch Enrich", "📥 Phase 3 — Import & Review", "🌟 Wishlist (Future Plants)", "📋 Field Reference".

**Phase 1 — export one plant at a time**
- Subheader "📤 Export Plants with Missing Fields — One at a Time".
- Buttons "🔄 Reload List" and "🔍 Load Plants with Missing Fields". Loading runs `SELECT * FROM data_source` and keeps rows with `get_missing_fields` non-empty.
- Progress "✅ {done} done · {remaining} remaining  ({total} total)".
- Select "🌿 Choose a plant to export:", subheader, metric "Missing fields", "**Fields that need filling:**".
- Download "⬇️  Download JSON for {name}" (`{name}_missing_{ts}.json`). This is **a single object, which Phase 3 rejects**.
- Checkbox "✅ Mark as Done" removes the plant from the queue for this session.
- Expander "🔍 Preview all current field values" (Field / Current Value / Needs Fill).
- **Session state:** `p1_queue`, `p1_done_ids`, `p1_select`.

**Phase 2 — Bulk Export 5**
- "🔄 Reload" (`p2e_reload`) and "🔍 Load Plants with Missing Fields" (`p2e_load`; `SELECT * … ORDER BY plant_name`).
- Text "🔍 Filter:", multiselect "Select plants to export (max 5 — {n} shown):".
- Per plant, an HTML table of missing fields.
- Download "⬇️ Download bulk_export_{n}plants_{ts}.json  (…)", which is an array.

**Phase 2 — Batch Enrich**
- Load button `p2_load`. Multiselect "🌿 Select plants to enrich ({n} with missing fields):" (all selected by default).
- Checkbox "🌐 Include web search (DuckDuckGo)" (default off).
- Slider "⚡ Parallel workers" (1–16, default 8).
- Button "🚀 Start Batch Enrichment ({n} plants)" calls `batch_enrich_plants` with a thread pool and a progress bar "Enriching {d} / {t} plants…".
  - With Ollama, 8 parallel workers just queue on the GPU, with a 300 s timeout each.
- "🔄 Reset".
- Results: dataframe (Plant / Fields Filled / Fields), errors expander, download `plants_enriched_{ts}.json` (array with `_enriched_fields` and `_proposed_changes`).
- **Nothing is written to the database in this phase.**

**Phase 3 — Import & Review** (writes `data_source`)
- Uploader "Upload enriched JSON file". It must be a top-level array, otherwise "JSON must be a top-level array of plant objects."
- Loads `SELECT * FROM data_source`.
- Diff per record:
  - Skipped keys: `id`, `_*`, `plant_name`, `picture`, `quantity`, `year_added`, `garden_beds`, `approved_*`, `needs_review`, `image_verified`, `image_search_term`, `date_added`.
  - "New fill" when the database value is empty; "overwrite" when both differ (compared as numbers when possible).
- Checkbox "Also accept ALL overwrites" and button "✅ Approve ALL — commit {n} new fills[ + {m} overwrites] across {k} plants" run `UPDATE data_source SET <keys>`.
- Per-plant expander "🌿 {name}  (ID: {id})":
  - Green table of new fills (month bitmaps rendered as coloured cells).
  - Amber table of overwrites, with a checkbox per overwrite "Accept `{f}`: `{old}` → `{new}`".
  - Button "✅ Approve & Commit  (n new fills · m overwrites)" and "⏭ Skip" (display only).
- **Security:** column names come straight from uploaded JSON keys (backticked, not whitelisted). `flower_bed`, `qty`, `year_planted` and `died` are not protected. Values are inserted unescaped into `unsafe_allow_html`. The Rust version should whitelist columns.

**Wishlist sub-tab** (writes `future_plants` only)
- Info "Backfill placeholder data for plants you **don't** own yet. …"
- Sub-tabs "➕ Add a Plant", "📦 Bulk Export 20", "📥 Backfill Import & Review".
- **Add a Plant** (form `wl_add_form`, clears on submit):
  - "Plant Name *", "Company / Brand", "Botanical Name", "Family", select "Plant Type" (`PLANT_TYPES`), "Zones", select "Sun Exposure" (`SUN_OPTIONS`), "Bloom Color", "Approx. Cost ($)", text area "Notes" (saved into `extra_details`).
  - Submit "➕ Add to Wishlist" calls `add_future_plant` (INSERT; duplicate → "A wishlist plant with that name + company already exists").
  - Warns if the name already exists in `data_source`.
- **Bulk Export 20:** `get_future_partials()` (20 required fields), filter, multiselect max 20, download `wishlist_export_{n}plants_{ts}.json` (strips `qr_code` and `barcode_image`).
- **Backfill Import:** the same diff UI as Phase 3, writing through `save_future_plant_fields` (filtered to real `future_plants` columns).

**Field Reference**
- Text "🔍 Filter fields". Dataframe columns Field / Type / Allowed Values / Example (about 70 rows).
- Expanders listing the full options for each enum.
- Download "⬇️ Download plant_schema.json" (JSON Schema draft-07, array of objects).

**AI:** `batch_enrich_plants` → `ask_claude_ai` (Ollama or Anthropic) + Perenual + optional DuckDuckGo. `_enrich_one` is defined but unused.

### 2.12 Tab 12 — `🗺️ Bed Assignment` (`tab_bed_assign.py`, 133 lines)
- Header "🗺️ Garden Bed Assignment", plus explanatory markdown.
- `SELECT id, plant_name, botanical_name, family, garden_beds, flower_bed, picture FROM data_source ORDER BY plant_name`.
- Text "🔍 Filter plants:" (placeholder "Type a name, family, or botanical name…"). Select "Select a plant ({n} shown):" (`bed_assign_select`).
- Left column: thumbnail (width 200) or "📷 Image file not found on disk" / "📷 No image uploaded yet"; ID, Botanical, Family, and "**Legacy flower_bed:** `…`".
- Multiselect "📍 Assign to garden bed(s):" (`bed_multi_{id}`; help "Select one or more beds. Hold Ctrl / Cmd to pick multiple."). Unknown legacy values are kept and shown as a caption.
- Button "💾 Save Bed Assignment": `UPDATE data_source SET garden_beds=%s WHERE id` (default "To Be Sorted").
  - **It does not mirror to `flower_bed`**, which the website reads, so the site drifts out of sync. `save_to_db` does mirror it.

### 2.13 Tab 13 — `🌟 Wishlist` (`tab_wishlist.py`, 152 lines)
- Header "🌟 Wishlist / Future Plants", caption "Browse, edit, and promote plants from the future_plants table into your garden."
- `get_future_plants()` (`SELECT * FROM future_plants ORDER BY plant_name`). Empty: "No plants in the wishlist. Add plants via the INSERT SQL or web scraping."
- Metric "Wishlist Plants".
- Text "🔍 Filter by name" (`wl_search`; matches name, botanical name or company). Select "Select a plant" with options "{name}  ({company})  [ID x]" (`wl_select`).
- Display: Botanical, Company, Family, Added (`date_added`). Missing key fields (`botanical_name, company, family, genus, species, plant_type, zones, sun_exposure, water_needs, bloom_color, foliage_color, size`) are shown as a warning, or "All key fields populated".
- Expander "✏️ Edit Fields" (form `wl_edit_{id}`):
  - Text inputs "Plant Name", "Botanical Name", "Company", "Family", "Genus", "Species", "Plant Type", "Zones", "Water Needs", "Size", "Bloom Color", "Foliage Color".
  - Select "Sun Exposure" and select "Deer Resistance".
  - Select "Garden Bed" ([""] + `GARDEN_BEDS`).
  - Submit "💾 Save Changes" calls `update_future_plant` ('' becomes NULL; the bed sets both `flower_bed` and `garden_beds`).
- Button "🌱 Promote to Garden":
  - If the name exists in `data_source`, it shows a warning and a nested "Yes, promote anyway" button. **That button can never fire** (nested button), and `force` is never passed anyway.
  - Otherwise it calls `promote_future_plant(fp_id)`.
- Button "🗑️ Remove from Wishlist": `DELETE FROM future_plants WHERE id`, **with no confirmation**.
- Expander "📋 Full Wishlist Table": columns `id, plant_name, botanical_name, company, family, plant_type, zones, sun_exposure, date_added`.

### 2.14 Tab 14 — `🏷️ Barcodes` (`tab_barcode.py` 112 lines + `barcode_lookup.py` 345 lines)
- Header "🏷️ Barcode Lookup", caption "Automatically look up UPC barcodes from UPCitemdb (free: 100 requests/day). Results are cached locally so repeated lookups don't burn API calls."
- Metrics "Cached Lookups" / "Found" / "Not Found" from `barcode_cache.json`, plus "Cache last updated: …".
- Progress bars:
  - "Garden (data_source)": `COUNT(*)` and the count with a barcode, `WHERE died = 0` (rows with NULL `died` are excluded).
  - "Wishlist (future_plants)".
- **Single Plant Lookup:** select "Table" ["data_source", "future_plants"], number "Plant ID" (min 1), button "🔍 Look Up Barcode".
  - Messages: "Found: **{name}** → `{upc}`", caption "Product: …", "Error: …", or "No barcode found for this plant."
- **Batch Lookup (Missing Barcodes):** select "Table", slider "Max plants to look up" (5–90, default 30), caption "⚠️ Free API: 100 requests/day. ~2.5s delay between requests.", button "🚀 Start Batch Lookup".
  - Blocks for up to about 225 s, and the progress bar only jumps to 100% at the end.
  - Then "Done! Found: x, Not found: y, Errors: z" and a dataframe (id, name, upc, title).
- External call: `GET https://api.upcitemdb.com/prod/trial/search?s=<name>&match_mode=0&type=product[&brand=<company>]`, sleeping 2.5 s before each request, and stopping on HTTP 429. It prefers an item whose title contains the name, then takes the first EAN or UPC.
- Writes `UPDATE <table> SET barcode=%s WHERE id` and the cache file `barcode_cache.json` (keyed `normalisedname|normalisedcompany`; "not found" results are cached too).
- Uses its own pymysql connection rather than the pool. **It has hardcoded fallback DB user/password literals [secret].**

---

## 3. backend.py (986 lines) and components.py (664 lines)

| Function | Description | Used by | Tables |
|---|---|---|---|
| `get_db_connection()` | Pooled mysql-connector connection (pool 16, `DB_POOL_SIZE`) with ping; falls back to a direct connection | all | — |
| `get_all_plant_names()` | Sorted unique `plant_name` list; `st.cache_data` ttl 60 s | form (companions), Tab 10, JSON Wishlist, Tab 13 | data_source |
| `setup_database()` | Adds missing columns; recreates the `Annuals`/`Perennials` views | app | data_source, INFORMATION_SCHEMA, views |
| `migrate_legacy_sizes()` | `size` → `plant_length` / `plant_width` | app | data_source |
| `save_to_db(plant_id, data)` | Full-row UPDATE plus `flower_bed` mirror (see 2.0) | form, Gemini | data_source |
| `sync_existing_images()` | Links files on disk to `picture` | app | data_source |
| `get_thumbnail_b64(pic_name)` | 80 px JPEG data-URI of the first picture | Tabs 6–9 | — |
| `process_plant_image(url_or_bytes)` | Download, centre-crop square, 900×900 | fetch | — |
| `save_image_as_webp(img, path, quality=87)` | Save WebP (method 4) | form, image_manager | — |
| `fetch_plant_image(botanical, common, optimized_term=None)` | Wikimedia Commons → en.wikipedia (`piprop=original`, `gsrlimit=3`) → DuckDuckGo images (3) | form | — |
| `fetch_location_context(plant_name)` | DuckDuckGo text search, 2 results (query mentions Clifton Park NY zone 5b 6a …) | Tabs 1, 2, 3, 10, pipeline | — |
| `fetch_perenual_data(plant_name)` | `GET perenual.com/api/species-list?key&q`, then `/api/species/details/{id}`; returns a JSON string | same | — |
| `ask_claude_ai(existing, model, loc_ctx, perenual_context="", allow_proposals=False)` | Main AI call (details below) | Tabs 1, 2, 3, 10, pipeline | — |
| `_enrich_is_empty(val)` | Blank test | pipeline | — |
| `enrich_plant_fields(row, model, use_web_search=True)` | Concurrent DuckDuckGo + Perenual, then AI; returns only fields that were empty | pipeline | — |
| `batch_enrich_plants(plants, model, use_web_search, max_workers=8, progress_cb)` | Thread pool (cap 16); returns `(results, errors)` | Pipeline Phase 2 | — |
| `get_future_plants()` | `SELECT * ORDER BY plant_name` | Tab 13, pipeline | future_plants |
| `get_future_plant_by_id(id)` | Single row (imported, never called) | — | future_plants |
| `update_future_plant(id, updates)` | UPDATE from a dict ('' becomes NULL; skips `id`/`date_added`) | Tab 13 | future_plants |
| `_ensure_link_column(cursor)` | Adds `data_source.future_plant_id INT NULL` plus index if missing | promote | data_source |
| `find_in_garden(fp_id, plant_name=None)` | Match by `future_plant_id`, then by exact name | promote | data_source, future_plants |
| `garden_beds_in_use()` | Distinct `flower_bed` by count, "To Be Sorted" first (unused) | — | data_source |
| `promote_future_plant(fp_id, bed=None, quantity=None, year=None, force=False)` | INSERT a copy of the `future_plants` row into `data_source`; keeps the wishlist row; sets `flower_bed`/`garden_beds` (default "To Be Sorted"), `qty`/`quantity`, `year_planted`, `future_plant_id`; refuses if already in the garden | Tabs 1, 13 | both |
| `delete_future_plant(id)` | DELETE | Tab 13 | future_plants |
| `get_future_columns()` | Cached column set from INFORMATION_SCHEMA | wishlist pipeline | — |
| `_coerce_future_value(col, val)` | Lists → bitmap or comma string; '' → NULL | wishlist | — |
| `future_plant_exists(name, company=None)` | Existence check (unused) | — | future_plants |
| `add_future_plant(data)` | Filtered INSERT; handles the `uq_name_company` duplicate | Pipeline Wishlist | future_plants |
| `get_future_partials(required_fields=None)` | Rows with any of 20 required fields blank, plus `_missing_fields` | Pipeline Wishlist | future_plants |
| `save_future_plant_fields(id, fields)` | Filtered UPDATE | Pipeline Wishlist | future_plants |
| components `render_plant_form(...)` | Shared editor (2.0) | Tabs 1, 2, 3, 4, 6, 7, 8, 9, 10 | data_source |
| components `render_quantity_logger(plant_id, plant_name, key_prefix)` | Quantity event logger | Tabs 1, 4, 6, 7, 8, 9 | plant_quantity_log, data_source |
| image_manager `render_image_manager(plant, key_prefix)`, `fetch_free_image(botanical, plant_name, search_term)` | Gallery widget; Wikipedia (`srlimit` 1, `pithumbsize` 640) then DuckDuckGo images (8, safesearch moderate) | Tabs 5, 6 | data_source |

**`ask_claude_ai` details**
- **Prompt:** "Provide data for '{plant}'", location context (Clifton Park, New York, USDA Zone 5b/6a, plus DuckDuckGo text), Perenual JSON (or "No Perenual API data found…"), 28 numbered field rules with the allowed value lists, "CRITICAL RULE: DO NOT OVERWRITE EXISTING DATA…", the current data as JSON, and an exact JSON template.
- **System prompt:** "You are a strict botanical data-API. Your ONLY output must be a valid JSON object. Do not include markdown formatting or explanations."
- **Local route** (`USE_LOCAL_AI`, the default): `POST OLLAMA_URL` with `{model: OLLAMA_MODEL (phi4:latest), system, prompt, stream: false, format: "json", options: {temperature: 0.2, num_predict: 2000}}`, timeout 300 s.
- **Anthropic route:** `messages.create(model=<sidebar>, max_tokens=2000)`.
- **Post-processing:**
  1. Extract the outermost `{…}` and strip trailing commas.
  2. Cast `average_cost` to float and `quantity` to int.
  3. Restore existing non-empty values unless `allow_proposals` is on and the AI value is non-empty. This rule is skipped for `fertilizer`, `garden_beds`, `companion_plants`, `year_added`, `is_evergreen`, so the AI value always wins there.
  4. Apply the month merge rules.
- On error it returns `{"error":…, "raw_output":…}`.

---

## 4. Exports

**`html_export.generate_plant_html(plant_data, title_suffix="", new_fields=None, changed_fields=None, original_row=None) -> bytes`** (470 lines; stdlib only, plus `MONTHS_MAP`)
- A self-contained, print-optimised HTML page: green header (`#1B4332`) with "🌱 {name}" and "{suffix} · Generated: {Month dd, YYYY  hh:mm AM}".
- In AI mode it adds a draft banner "⚠️  DRAFT — PENDING YOUR VERIFICATION…" and a legend (green = AI filled, amber = AI proposed change, white = unchanged).
- Button "🖨️ Print / Save as PDF" (`window.print()`).
- 12 sections mirroring the form (Taxonomy, Growth, Soil, Investment, Beds, Foliage, Flower, Fruit, Roots, Care, Calendars shown as "Apr – Sep", Notes), plus "🤖 AI Session Metadata" in AI mode, and a footer.
- Rows are green (`row-new`, badge "✨ AI filled") or amber (`row-changed`, badge "🔄 Changed · was: …").
- Values of none/null/0 show as "—".
- Beds, qty and years come from `original_row`.
- Used by Tabs 2, 3, 4, 6 and 10.

**`pdf_export.generate_plant_pdf(plant_row, ai_data, new_fields=None, changed_fields=None) -> bytes`** (445 lines; **reportlab** platypus, Letter size)
- Same content as the HTML version, as a PDF verification document, with "Model used: {ai_data._model or 'Claude'}".
- **Not called anywhere; dead code.**

**`export_months_for_review.py`** (262 lines; standalone CLI; `mysql.connector`, `argparse`, `json`, `pathlib`; **hardcoded fallback DB user/password [secret]**)
- **Export mode** (`--only-empty`, `--ids`, `--batch-size` 25, `--location` default "Clifton Park, NY — USDA Hardiness Zone 6b", which disagrees with the 5b/6a used elsewhere):
  - Reads `id, plant_name, botanical_name, family, genus, species, plant_type, zones, sun_exposure, water_needs, bloom_color, foliage_color, grow_months, bloom_months, flower_bed, is_evergreen, extra_details, size, average_cost` `WHERE died=0`.
  - Writes `./month_review/batch_NNN.json` with `_instructions` (a verification prompt to paste into Gemini/ChatGPT), `_location`, `_batch`, `_count`, `_import_command`, and `plants[]` carrying the current bitmap and readable months, empty `*_verified` arrays and `notes`.
  - Bug: `is_evergreen` is exported as `bool(string)`, so "Deciduous" becomes true.
- **`--import <dir|file>`:** `UPDATE data_source SET grow_months / bloom_months` from the `*_verified` month-name lists.

**Also not mounted:** `tab_gemini_export.py` (560 lines; header "🤖 Gemini Plant Enrichment").
- Radio "Show" ["Plants with missing fields", "All plants"], text "🔍 Search by name", multiselect (max 5) sorted by missing count.
- "🚀 Build Prompt" builds a long Gemini prompt (location context, master field schema, 5-step pipeline, plant JSON). It shows a text area "📋 Copy everything below and paste into a new Gemini chat:" and offers .txt and .json downloads.
- Pasting the response diffs it against the originals (read-only fields skipped). "✅ Save All Changes to Database" calls `save_to_db`.
- No API call is made; it is manual copy-paste.

---

## 5. Summary table

| Tab | Reads | Writes | AI / web calls | Complexity |
|---|---|---|---|---|
| 🌱 Add Plant | data_source, future_plants | data_source (INSERT/UPDATE), future_plants→data_source (promote), plant_quantity_log, image files | ask_claude_ai (Ollama phi4 / Anthropic), Perenual, DuckDuckGo text, Wikimedia/Wikipedia/DuckDuckGo images | L |
| 📝 Manual Review (AI) | data_source | data_source, plant_quantity_log*, image files | ask_claude_ai, Perenual, DuckDuckGo, image search | M |
| ✨ Fill Missing Data (AI) | data_source | data_source, image files | same | M |
| ✍️ Manual Editor | data_source | data_source, plant_quantity_log, image files | none | S (plus the shared form, L) |
| 📸 Manual Image Replace | data_source | data_source (picture, additional_pictures, image_verified, image_search_term), image files | Wikipedia API, DuckDuckGo images | M |
| ⚠️ Partials | data_source | data_source, plant_quantity_log, image files | Wikipedia/DuckDuckGo (image manager) | M |
| 🔍 Awaiting Approval | Perennials, Annuals views | data_source, plant_quantity_log | none | S |
| 🏆 Fully Approved & Verified | Perennials, Annuals views | data_source, plant_quantity_log | none | S |
| 🚩 Flagged for Review | data_source | data_source, plant_quantity_log | none | S |
| 🔒 AI — Missing Only | data_source | data_source, image files | ask_claude_ai, Perenual, DuckDuckGo, **Anthropic always** (companions) | M |
| 🔄 JSON Pipeline | data_source, future_plants, INFORMATION_SCHEMA | data_source (Phase 3), future_plants (wishlist) | batch_enrich_plants → ask_claude_ai, Perenual, optional DuckDuckGo | L |
| 🗺️ Bed Assignment | data_source | data_source.garden_beds | none | S |
| 🌟 Wishlist | future_plants, data_source | future_plants (UPDATE/DELETE), data_source (promote INSERT, ALTER) | none | M |
| 🏷️ Barcodes | data_source, future_plants, barcode_cache.json | barcode column in either table, barcode_cache.json | UPCitemdb trial API | M |

\*The quantity logger is not rendered in Tabs 2, 3 and 10, only in Tabs 1, 4, 6, 7, 8 and 9.

---

## 6. Every table and column referenced

**data_source:** `id, plant_name, botanical_name, company, family, genus, species, plant_type, native_status, plant_length, plant_width, size, spacing, zones, life_expectancy, growth_rate, sun_exposure, water_needs, soil_ph, soil_type, planting_depth, chilling_hours, average_cost, quantity, qty, year_added, year_planted, died_year, died, garden_beds, flower_bed, foliage_color, stem_color, is_evergreen, leaf_shape, leaf_type, leaf_margin, bloom_color, aromatics, flower_symmetry, flower_shape, flower_size, reproduction_type, fruit_type, fruit_shape, fruit_color, fruit_size, fruit_edibility, root_system, root_morphology, root_function, attracts, deer_resistance, toxicity, invasive_local, propagation_method, cuttings, companion_plants, fertilizer, grow_months, bloom_months, care_notes, maintenance_notes, cleanup_notes, disease_susceptibility, extra_details, image_search_term, picture, additional_pictures, image_verified, approved_jason, approved_jun, needs_review, barcode, future_plant_id` (plus index `idx_future_plant_id`).
- `local_store` appears only as a key in the Add Plant schema JSON.

**future_plants:**
- `id, plant_name, botanical_name, company, family, genus, species, plant_type, zones, sun_exposure, water_needs, size, bloom_color, foliage_color, deer_resistance, flower_bed, garden_beds, date_added, average_cost, extra_details, barcode`.
- Columns used where present: `plant_length, plant_width, spacing, soil_type, soil_ph, toxicity, native_status, growth_rate, care_notes, maintenance_notes, grow_months, bloom_months, qr_code, barcode_image`.
- Unique key `uq_name_company (plant_name, company)`.
- Promotion copies all of its columns into `data_source`, so its columns must be a subset of `data_source`'s.

**plant_quantity_log:** `plant_id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by`.

**Views:** `Annuals`, `Perennials` (`SELECT * FROM data_source` filtered on `plant_type`).

**System:** `INFORMATION_SCHEMA.COLUMNS` (`TABLE_SCHEMA`, `TABLE_NAME`, `COLUMN_NAME`).

**Files**
- Image folder `IMAGE_DIR` (`*.webp` named `{Safe}_{id}_{epoch}.webp` or `{Safe}_{id}_manual_{epoch}.webp`).
- `barcode_cache.json` (app folder).
- `plant_schema_reference.txt` and `static/plant_tag_guide.jpg`: read by Add Plant, both missing from the folder.
- `month_review/batch_NNN.json` (CLI).

**Other notes**
- `requirements.txt` pins `streamlit==1.32.0` but leaves `duckduckgo-search` and `anthropic` unpinned. pandas is imported but comes in through streamlit.
- `st.image(..., use_container_width=True)` in Add Plant may not exist in Streamlit 1.32. It only matters if the tag image file is added.
