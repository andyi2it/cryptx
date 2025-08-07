import Database from '@tauri-apps/plugin-sql';

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
    const tableInfo = await db.select("PRAGMA table_info(secrets)") as Array<{ name: string }>;
    const columns = tableInfo.map((col: any) => col.name);
    
    const requiredColumns = ['id', 'name', 'key', 'created_at', 'updated_at'];
    const missingColumns = requiredColumns.filter(col => !columns.includes(col));
    
    if (missingColumns.length > 0 && tableInfo.length > 0) {
      console.log("Missing columns detected:", missingColumns);
      console.log("Recreating secrets table with proper structure...");
      
      // Backup existing data
      const existingSecrets = await db.select("SELECT * FROM secrets") as Array<{ name: string; key: string }>;
      
      // Drop and recreate table
      await db.execute("DROP TABLE IF EXISTS secrets");
      await db.execute(`
        CREATE TABLE secrets (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL UNIQUE,
          key TEXT NOT NULL,
          created_at TEXT DEFAULT (datetime('now')),
          updated_at TEXT DEFAULT (datetime('now'))
        )
      `);
      
      // Restore data
      for (const secret of existingSecrets) {
        await db.execute(
          "INSERT INTO secrets (name, key, created_at, updated_at) VALUES ($1, $2, datetime('now'), datetime('now'))",
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
          updated_at TEXT DEFAULT (datetime('now'))
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
    const tableInfo = await db.select("PRAGMA table_info(secrets)") as Array<{ name: string }>;
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
    const tableInfo = await db.select("PRAGMA table_info(secrets)") as Array<{ name: string }>;
    const columns = tableInfo.map((col: any) => col.name);
    
    let query: string;
    let params: any[];
    
    if (columns.includes('updated_at')) {
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
      "DELETE FROM secrets WHERE id = $1", 
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
    const tableInfo = await db.select("PRAGMA table_info(secrets)") as Array<{ name: string }>;
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
    query += " FROM secrets ORDER BY id DESC";
    
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

// Add new functions for clearing all data

export async function deleteAllUsers(): Promise<void> {
  try {
    const db = await getDatabase();
    const result = await db.execute("DELETE FROM users");
    console.log("All users deleted successfully", result);
  } catch (error) {
    console.error("Failed to delete all users:", error);
    throw error;
  }
}

export async function deleteAllSecrets(): Promise<void> {
  try {
    const db = await getDatabase();
    const result = await db.execute("DELETE FROM secrets");
    console.log("All secrets deleted successfully", result);
  } catch (error) {
    console.error("Failed to delete all secrets:", error);
    throw error;
  }
}

export async function deleteAllData(): Promise<void> {
  try {
    const db = await getDatabase();
    await db.execute("DELETE FROM secrets");
    await db.execute("DELETE FROM users");
    console.log("All data deleted successfully - fresh start ready");
  } catch (error) {
    console.error("Failed to delete all data:", error);
    throw error;
  }
}

export async function dropAllTables(): Promise<void> {
  try {
    const db = await getDatabase();
    await db.execute("DROP TABLE IF EXISTS secrets");
    await db.execute("DROP TABLE IF EXISTS users");
    console.log("All tables dropped successfully - database reset complete");
  } catch (error) {
    console.error("Failed to drop tables:", error);
    throw error;
  }
}

export async function resetDatabase(): Promise<void> {
  try {
    console.log("Resetting database for fresh start...");
    
    // Drop all tables
    await dropAllTables();
    
    // Reinitialize database with fresh tables
    await initDatabase();
    
    console.log("Database reset complete - ready for fresh setup");
  } catch (error) {
    console.error("Failed to reset database:", error);
    throw error;
  }
}

// Helper function to get counts for verification
export async function getDatabaseStats(): Promise<{ userCount: number; secretCount: number }> {
  try {
    const db = await getDatabase();
    
    let userCount = 0;
    let secretCount = 0;
    
    try {
      const userResult = await db.select("SELECT COUNT(*) as count FROM users") as Array<{ count: number }>;
      userCount = userResult[0]?.count || 0;
    } catch (e) {
      console.log("Users table doesn't exist yet");
    }
    
    try {
      const secretResult = await db.select("SELECT COUNT(*) as count FROM secrets") as Array<{ count: number }>;
      secretCount = secretResult[0]?.count || 0;
    } catch (e) {
      console.log("Secrets table doesn't exist yet");
    }
    
    return { userCount, secretCount };
  } catch (error) {
    console.error("Failed to get database stats:", error);
    return { userCount: 0, secretCount: 0 };
  }
}

