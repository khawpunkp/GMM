pub const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS categories (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    slug TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS agents (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    slug        TEXT UNIQUE NOT NULL,
    description TEXT,
    details     TEXT,
    base_image  TEXT,
    is_builtin  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS agent_aliases (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id INTEGER NOT NULL,
    alias    TEXT NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    UNIQUE (agent_id, alias)
);

CREATE TABLE IF NOT EXISTS mods (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id       INTEGER,   -- nullable: UI/uncategorized mods
    category_id    INTEGER,
    name           TEXT NOT NULL,
    description    TEXT,
    folder_name    TEXT NOT NULL UNIQUE,
    image_filename TEXT,
    author         TEXT,
    FOREIGN KEY (agent_id)    REFERENCES agents(id)     ON DELETE SET NULL,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS mod_groups (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS mod_group_members (
    group_id INTEGER NOT NULL,
    mod_id   INTEGER NOT NULL,
    PRIMARY KEY (group_id, mod_id),
    FOREIGN KEY (group_id) REFERENCES mod_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (mod_id)   REFERENCES mods(id)        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS presets (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT UNIQUE NOT NULL,
    is_favorite INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS preset_mods (
    preset_id  INTEGER NOT NULL,
    mod_id     INTEGER NOT NULL,
    is_enabled INTEGER NOT NULL,
    PRIMARY KEY (preset_id, mod_id),
    FOREIGN KEY (preset_id) REFERENCES presets(id) ON DELETE CASCADE,
    FOREIGN KEY (mod_id)    REFERENCES mods(id)     ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
"#;
