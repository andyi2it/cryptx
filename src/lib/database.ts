import Database from '@tauri-apps/plugin-sql';
import { invoke } from '@tauri-apps/api/core';

let database: Database | null = null;

export async function getDatabase(): Promise<Database> {
  if (!database) {
    try {
      database = await Database.load("sqlite:cryptx.db");
      console.log("Database connection established");
    } catch (error) {
      console.error("Failed to connect to database:", error);
      throw new Error("Database connection failed");
    }
  }
  return database;
}

export async function initDatabase() {
  try {
    const db = await getDatabase();
    
    // Drop existing users table to add email field and remove old records
    // await db.execute('DROP TABLE IF EXISTS users');

    // Create users table with email field
    await db.execute(`
      CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        email TEXT NOT NULL,
        public_key TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )
    `);
    
    // Check if secrets table exists and has the right structure
    const tableInfo = await db.select("PRAGMA table_info(secrets)");
    const columns = tableInfo.map((col: any) => col.name);
    
    const requiredColumns = ['id', 'name', 'key', 'created_at', 'updated_at', 'deleted'];
    const missingColumns = requiredColumns.filter(col => !columns.includes(col));
    
    if (missingColumns.length > 0 && tableInfo.length > 0) {
      console.log("Missing columns detected:", missingColumns);
      console.log("Recreating secrets table with proper structure...");
      
      // Backup existing data
      const existingSecrets = await db.select("SELECT * FROM secrets");
      
      // Drop and recreate table
      await db.execute("DROP TABLE IF EXISTS secrets");
      await db.execute(`
        CREATE TABLE secrets (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL UNIQUE,
          key TEXT NOT NULL,
          created_at TEXT DEFAULT (datetime('now')),
          updated_at TEXT DEFAULT (datetime('now')),
          deleted INTEGER DEFAULT 0
        )
      `);
      
      // Restore data
      for (const secret of existingSecrets) {
        await db.execute(
          "INSERT INTO secrets (name, key, created_at, updated_at, deleted) VALUES ($1, $2, datetime('now'), datetime('now'), 0)",
          [secret.name, secret.key]
        );
      }
      console.log("Table recreated and data restored");
    } else if (tableInfo.length === 0) {
      // Create new table
      await db.execute(`
        CREATE TABLE secrets (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL UNIQUE,
          key TEXT NOT NULL,
          created_at TEXT DEFAULT (datetime('now')),
          updated_at TEXT DEFAULT (datetime('now')),
          deleted INTEGER DEFAULT 0
        )
      `);
      console.log("Created new secrets table");
    }
    
    console.log("Database initialization completed successfully");
    return db;
  } catch (error) {
    console.error('Database initialization failed:', error);
    throw error;
  }
}

export async function addSecret(name: string, key: string) {
  try {
    const db = await getDatabase();
    
    // Check what columns exist before inserting
    const tableInfo = await db.select("PRAGMA table_info(secrets)");
    const columns = tableInfo.map((col: any) => col.name);
    
    let query: string;
    let params: any[];
    
    if (columns.includes('created_at') && columns.includes('updated_at')) {
      query = "INSERT INTO secrets (name, key, created_at, updated_at) VALUES ($1, $2, datetime('now'), datetime('now'))";
      params = [name, key];
    } else {
      query = "INSERT INTO secrets (name, key) VALUES ($1, $2)";
      params = [name, key];
    }
    
    const result = await db.execute(query, params);
    console.log("Secret added successfully, result:", result);
    return result;
  } catch (error) {
    console.error("Failed to add secret:", error);
    throw new Error(`Failed to add secret: ${error}`);
  }
}

export async function updateSecret(id: number, name: string, key: string) {
  try {
    const db = await getDatabase();
    
    // Check what columns exist before updating
    const tableInfo = await db.select("PRAGMA table_info(secrets)");
    const columns = tableInfo.map((col: any) => col.name);
    
    let query: string;
    let params: any[];
    
    if (columns.includes('updated_at') && columns.includes('deleted')) {
      query = "UPDATE secrets SET name = $1, key = $2, updated_at = datetime('now') WHERE id = $3 AND deleted = 0";
      params = [name, key, id];
    } else if (columns.includes('updated_at')) {
      query = "UPDATE secrets SET name = $1, key = $2, updated_at = datetime('now') WHERE id = $3";
      params = [name, key, id];
    } else {
      query = "UPDATE secrets SET name = $1, key = $2 WHERE id = $3";
      params = [name, key, id];
    }
    
    const result = await db.execute(query, params);
    console.log("Secret updated successfully, result:", result);
    return result;
  } catch (error) {
    console.error("Failed to update secret:", error);
    throw new Error(`Failed to update secret: ${error}`);
  }
}

export async function deleteSecret(id: number) {
  try {
    const db = await getDatabase();
    const result = await db.execute(
      "UPDATE secrets SET deleted = 1, updated_at = datetime('now') WHERE id = $1", 
      [id]
    );
    console.log("Secret deleted successfully, result:", result);
    return result;
  } catch (error) {
    console.error("Failed to delete secret:", error);
    throw new Error(`Failed to delete secret: ${error}`);
  }
}

export async function getSecrets(): Promise<Array<{ id: number; name: string; key: string; created_at: string; updated_at: string }>> {
  try {
    const db = await getDatabase();
    
    // First check if the columns exist
    const tableInfo = await db.select("PRAGMA table_info(secrets)");
    const columns = tableInfo.map((col: any) => col.name);
    
    let query = "SELECT id, name, key";
    if (columns.includes('created_at')) {
      query += ", created_at";
    } else {
      query += ", datetime('now') as created_at";
    }
    if (columns.includes('updated_at')) {
      query += ", updated_at";
    } else {
      query += ", datetime('now') as updated_at";
    }
    query += " FROM secrets";
    
    if (columns.includes('deleted')) {
      query += " WHERE deleted = 0";
    }
    query += " ORDER BY id DESC";
    
    const result = await db.select<Array<{ id: number; name: string; key: string; created_at: string; updated_at: string }>>(query);
    console.log("Secrets retrieved successfully, count:", result.length);
    return result;
  } catch (error) {
    console.error("Failed to get secrets:", error);
    throw new Error(`Failed to get secrets: ${error}`);
  }
}

export async function addUser(name: string, email: string, publicKey: string): Promise<void> {
  try {
    const db = await getDatabase();
    await db.execute(
      'INSERT INTO users (name, email, public_key) VALUES ($1, $2, $3)',
      [name, email, publicKey]
    );
    console.log("User added successfully");
  } catch (error) {
    console.error('Failed to add user:', error);
    throw error;
  }
}

export async function getUsers(): Promise<Array<{ id: number; name: string; email: string; public_key: string }>> {
  try {
    const db = await getDatabase();
    const result = await db.select<Array<{ id: number; name: string; email: string; public_key: string }>>(
      'SELECT id, name, email, public_key FROM users ORDER BY name'
    );
    return result;
  } catch (error) {
    console.error('Failed to get users:', error);
    throw error;
  }
}


export async function deleteUser(id: number) {
  try {
    const db = await getDatabase();
    await db.execute("DELETE FROM users WHERE id = ?", [id]);
    console.log("User deleted successfully");
  } catch (error) {
    console.error("Failed to delete user:", error);
    throw error;
  }
}
