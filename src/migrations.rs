use rusqlite_migration::{M, Migrations};

// 1️⃣ Define migrations
const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "CREATE TABLE imap_servers(
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           server TEXT NOT NULL,
           user TEXT NOT NULL,
           UNIQUE(server, user)
        );
        CREATE TABLE imap_folders(
           server_id INTEGER NOT NULL,
           name TEXT NOT NULL,
           last_seen_uid INTEGER,
           uid_validity INTEGER NOT NULL,
           PRIMARY KEY(server_id, name),
           FOREIGN KEY(server_id) REFERENCES imap_servers(id)
        );
",
    ),
    // In the future, add more migrations here:
    //M::up("ALTER TABLE friend ADD COLUMN email TEXT;"),
];
pub const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);
