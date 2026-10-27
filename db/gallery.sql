-- Gallery tables. ADDITIVE ONLY: new tables and new indexes on the new tables; nothing existing is altered.
-- Idempotent (IF NOT EXISTS). Run by gallery_db.py on the D: copy (jg-db / junsgarden_work) only.
-- The actual database gets this only after Jason's explicit OK (see REVIEW.md).

-- One row per picture, whatever folder it lives in. Tags attach by photo.id, never by folder.
CREATE TABLE IF NOT EXISTS photo (
  id            INT UNSIGNED NOT NULL AUTO_INCREMENT,
  sha256        CHAR(64)     NOT NULL,
  filename      VARCHAR(255) NOT NULL,
  folder_kind   ENUM('plant','future','other') NOT NULL,   -- plant = current garden, future = wishlist, other = cats/people/flowers not tied to a plant
  media         ENUM('image','gif','video') NOT NULL DEFAULT 'image',
  set_name      VARCHAR(40)  NOT NULL,                     -- which scanned folder (current, future-v1, not-in-database, ...)
  plant_id      INT          NULL,                         -- data_source.id (current plant); NULL for 'future' and 'other'
  future_plant_id INT        NULL,                         -- future_plants.id (wishlist); NULL otherwise
  taken_at      DATETIME     NULL,                         -- EXIF capture time when present
  width         INT UNSIGNED NULL,
  height        INT UNSIGNED NULL,
  bytes         BIGINT UNSIGNED NULL,
  full_path     VARCHAR(500) NOT NULL,                     -- the ORIGINAL picture (never a resized copy)
  full_rel      VARCHAR(300) NOT NULL,                     -- path inside the set folder; what the site joins to its base URL
  thumb_rel     VARCHAR(300) NULL,                         -- thumbs\<set>\<id>\<name>.webp, NULL when no thumbnail
  status        ENUM('active','quarantined','unreadable','duplicate','missing') NOT NULL DEFAULT 'active',   -- only 'active' shows in the gallery
  added_at      TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY uq_photo_file (set_name, full_rel),
  KEY idx_photo_plant (plant_id, id),                      -- a plant's pictures, keyset paged
  KEY idx_photo_future (future_plant_id, id),
  KEY idx_photo_list (status, id),                         -- the plain gallery page: WHERE status='active' AND id > ? ORDER BY id LIMIT n
  KEY idx_photo_sha (sha256),                              -- duplicate lookups
  KEY idx_photo_filename (filename(120)),                  -- tag importer: exact filename match
  CONSTRAINT fk_photo_plant  FOREIGN KEY (plant_id)        REFERENCES data_source (id)    ON DELETE SET NULL ON UPDATE CASCADE,
  CONSTRAINT fk_photo_future FOREIGN KEY (future_plant_id) REFERENCES future_plants (id)  ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Tags on a picture. source: auto (local tools), stag (the Windows tagger), manual.
CREATE TABLE IF NOT EXISTS photo_tags (
  photo_id    INT UNSIGNED NOT NULL,
  tag         VARCHAR(80)  NOT NULL,
  kind        ENUM('plant','color','part','season','place','subject','other') NOT NULL DEFAULT 'other',
  source      ENUM('auto','stag','manual','legacy-xmp','legacy-site') NOT NULL,
  confidence  DECIMAL(4,3) NULL,
  created_at  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (photo_id, tag, source),
  KEY idx_photo_tags_tag (tag, photo_id),                  -- search: WHERE tag = ? ORDER BY photo_id (keyset)
  CONSTRAINT fk_pt_photo FOREIGN KEY (photo_id) REFERENCES photo (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Tags on a plant itself.
CREATE TABLE IF NOT EXISTS plant_tags (
  plant_id    INT          NOT NULL,
  tag         VARCHAR(80)  NOT NULL,
  kind        ENUM('plant','color','part','season','place','subject','other') NOT NULL DEFAULT 'other',
  source      ENUM('auto','stag','manual','legacy-xmp','legacy-site') NOT NULL,
  confidence  DECIMAL(4,3) NULL,
  created_at  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (plant_id, tag, source),
  KEY idx_plant_tags_tag (tag, plant_id),
  CONSTRAINT fk_plt_plant FOREIGN KEY (plant_id) REFERENCES data_source (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Every tag row an import wrote, so a batch can be undone (DELETE ... JOIN on batch) without touching other tags.
CREATE TABLE IF NOT EXISTS photo_tags_rollback (
  id          INT UNSIGNED NOT NULL AUTO_INCREMENT,
  batch       VARCHAR(40)  NOT NULL,
  photo_id    INT UNSIGNED NOT NULL,
  tag         VARCHAR(80)  NOT NULL,
  source      ENUM('auto','stag','manual','legacy-xmp','legacy-site') NOT NULL,
  inserted_at TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_ptr_batch (batch)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- The old gallery's tags, every one kept as found, matched or not (the new gallery does not read this table):
-- origin 'site' = images/Gallery/tags_cache.csv, 'xmp' = the .xmp sidecar next to each picture.
CREATE TABLE IF NOT EXISTS gallery_tags_legacy (
  file_name   VARCHAR(255) NOT NULL,
  tag         VARCHAR(120) NOT NULL,
  origin      ENUM('site','xmp') NOT NULL,
  PRIMARY KEY (file_name, tag, origin)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
