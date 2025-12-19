use rusqlite_migration::{M, Migrations};

// 1️⃣ Define migrations
const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up("CREATE TABLE friend(name TEXT NOT NULL);"),
    // In the future, add more migrations here:
    //M::up("ALTER TABLE friend ADD COLUMN email TEXT;"),
];
pub const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);
