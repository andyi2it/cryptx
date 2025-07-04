import Database from '@tauri-apps/plugin-sql';

let db: Database | null = null;

export async function initDatabase() {
  if (!db) {
    db = await Database.load('sqlite:cryptx.db');
    
    // Create tables if they don't exist
    await db.execute(`
      CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        public_key TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )
    `);

    await db.execute(`
      CREATE TABLE IF NOT EXISTS secrets (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        key TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )
    `);
  }
  return db;
}

export async function getDatabase() {
  if (!db) {
    await initDatabase();
  }
  return db!;
}

// User operations
export async function addUser(name: string, publicKey: string) {
  const database = await getDatabase();
  return await database.execute(
    'INSERT INTO users (name, public_key) VALUES (?, ?)',
    [name, publicKey]
  );
}

export async function getUsers() {
  const database = await getDatabase();
  return await database.select<{ id: number; name: string; public_key: string }>('SELECT * FROM users ORDER BY name');
}

// Secret operations
export async function addSecret(name: string, key: string) {
  const database = await getDatabase();
  return await database.execute(
    'INSERT INTO secrets (name, key) VALUES (?, ?)',
    [name, key]
  );
}

export async function getSecrets() {
  const database = await getDatabase();
  return await database.select<{ id: number; name: string; key: string }>('SELECT * FROM secrets ORDER BY name');
}

// export { getDatabase };
