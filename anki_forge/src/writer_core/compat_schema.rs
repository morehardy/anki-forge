//! Minimal Anki collection schema required by the writer.
//!
//! This is a project-owned compatibility implementation of the documented
//! collection layout. The upstream source mirror remains under `docs/source`
//! for conformance review and is not copied into the Rust Distribution.

pub(super) const SCHEMA11_SQL: &str = r#"
CREATE TABLE col (
  id integer PRIMARY KEY, crt integer NOT NULL, mod integer NOT NULL,
  scm integer NOT NULL, ver integer NOT NULL, dty integer NOT NULL,
  usn integer NOT NULL, ls integer NOT NULL, conf text NOT NULL,
  models text NOT NULL, decks text NOT NULL, dconf text NOT NULL,
  tags text NOT NULL
);
CREATE TABLE notes (
  id integer PRIMARY KEY, guid text NOT NULL, mid integer NOT NULL,
  mod integer NOT NULL, usn integer NOT NULL, tags text NOT NULL,
  flds text NOT NULL, sfld integer NOT NULL, csum integer NOT NULL,
  flags integer NOT NULL, data text NOT NULL
);
CREATE TABLE cards (
  id integer PRIMARY KEY, nid integer NOT NULL, did integer NOT NULL,
  ord integer NOT NULL, mod integer NOT NULL, usn integer NOT NULL,
  type integer NOT NULL, queue integer NOT NULL, due integer NOT NULL,
  ivl integer NOT NULL, factor integer NOT NULL, reps integer NOT NULL,
  lapses integer NOT NULL, left integer NOT NULL, odue integer NOT NULL,
  odid integer NOT NULL, flags integer NOT NULL, data text NOT NULL
);
CREATE TABLE revlog (
  id integer PRIMARY KEY, cid integer NOT NULL, usn integer NOT NULL,
  ease integer NOT NULL, ivl integer NOT NULL, lastIvl integer NOT NULL,
  factor integer NOT NULL, time integer NOT NULL, type integer NOT NULL
);
CREATE TABLE graves (usn integer NOT NULL, oid integer NOT NULL, type integer NOT NULL);
CREATE INDEX ix_notes_usn ON notes (usn);
CREATE INDEX ix_cards_usn ON cards (usn);
CREATE INDEX ix_revlog_usn ON revlog (usn);
CREATE INDEX ix_cards_nid ON cards (nid);
CREATE INDEX ix_cards_sched ON cards (did, queue, due);
CREATE INDEX ix_revlog_cid ON revlog (cid);
CREATE INDEX ix_notes_csum ON notes (csum);
INSERT INTO col VALUES (1, 0, 0, 0, 0, 0, 0, 0, '{}', '{}', '{}', '{}', '{}');
"#;

pub(super) const SCHEMA14_UPGRADE_SQL: &str = r#"
CREATE TABLE deck_config (
  id integer PRIMARY KEY NOT NULL, name text NOT NULL,
  mtime_secs integer NOT NULL, usn integer NOT NULL, config blob NOT NULL
);
CREATE TABLE config (
  key text NOT NULL PRIMARY KEY, usn integer NOT NULL,
  mtime_secs integer NOT NULL, val blob NOT NULL
) without rowid;
CREATE TABLE tags (
  tag text NOT NULL PRIMARY KEY, usn integer NOT NULL
) without rowid;
UPDATE col SET ver = 14;
"#;

pub(super) const SCHEMA15_UPGRADE_SQL: &str = r#"
CREATE TABLE fields (
  ntid integer NOT NULL, ord integer NOT NULL, name text NOT NULL,
  config blob NOT NULL, PRIMARY KEY (ntid, ord)
) without rowid;
CREATE UNIQUE INDEX idx_fields_name_ntid ON fields (name, ntid);
CREATE TABLE templates (
  ntid integer NOT NULL, ord integer NOT NULL, name text NOT NULL,
  mtime_secs integer NOT NULL, usn integer NOT NULL, config blob NOT NULL,
  PRIMARY KEY (ntid, ord)
) without rowid;
CREATE UNIQUE INDEX idx_templates_name_ntid ON templates (name, ntid);
CREATE INDEX idx_templates_usn ON templates (usn);
CREATE TABLE notetypes (
  id integer NOT NULL PRIMARY KEY, name text NOT NULL,
  mtime_secs integer NOT NULL, usn integer NOT NULL, config blob NOT NULL
);
CREATE UNIQUE INDEX idx_notetypes_name ON notetypes (name);
CREATE INDEX idx_notetypes_usn ON notetypes (usn);
CREATE TABLE decks (
  id integer PRIMARY KEY NOT NULL, name text NOT NULL,
  mtime_secs integer NOT NULL, usn integer NOT NULL,
  common blob NOT NULL, kind blob NOT NULL
);
CREATE UNIQUE INDEX idx_decks_name ON decks (name);
CREATE INDEX idx_notes_mid ON notes (mid);
CREATE INDEX idx_cards_odid ON cards (odid) WHERE odid != 0;
UPDATE col SET ver = 15;
ANALYZE;
"#;

pub(super) const SCHEMA17_UPGRADE_SQL: &str = r#"
DROP TABLE tags;
CREATE TABLE tags (
  tag text NOT NULL PRIMARY KEY, usn integer NOT NULL,
  collapsed boolean NOT NULL, config blob NULL
) without rowid;
"#;

pub(super) const SCHEMA18_UPGRADE_SQL: &str = r#"
ALTER TABLE graves RENAME TO graves_old;
CREATE TABLE graves (
  oid integer NOT NULL, type integer NOT NULL, usn integer NOT NULL,
  PRIMARY KEY (oid, type)
) without rowid;
INSERT OR IGNORE INTO graves (oid, type, usn) SELECT oid, type, usn FROM graves_old;
DROP TABLE graves_old;
CREATE INDEX idx_graves_pending ON graves (usn);
UPDATE col SET ver = 18;
"#;
