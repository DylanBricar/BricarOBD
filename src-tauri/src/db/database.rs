use rusqlite::{params, Connection};
use std::path::Path;
use tracing::info;

use crate::models::AppSettings;

/// SQLite database for BricarOBD
/// Ships with a pre-built DB containing 3.27M operations, 90 vehicle profiles, 4866 ECUs
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open pre-built database
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -32000;
             PRAGMA temp_store = MEMORY;",
        )
        .ok();

        // Create user tables if missing (operations/ecus are pre-built)
        if let Err(e) = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS dtc_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT, code TEXT NOT NULL, description TEXT,
                status TEXT NOT NULL, source TEXT, vehicle_vin TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP, cleared BOOLEAN DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT, vehicle_vin TEXT, vehicle_make TEXT,
                vehicle_model TEXT, dtc_count INTEGER DEFAULT 0, notes TEXT, data TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        ) {
            tracing::error!("User tables creation failed: {}", e);
            return Err(format!("User tables creation failed: {}", e));
        }

        let db = Self { conn };
        info!("Database opened: {}", path.display());
        Ok(db)
    }

    // ==================== OPERATIONS (3.27M) ====================

    /// Get total operations count
    pub fn operations_count(&self) -> u64 {
        self.conn
            .query_row("SELECT COUNT(*) FROM operations", [], |r| r.get(0))
            .unwrap_or(0)
    }

    /// Get database stats
    pub fn get_stats(&self) -> (u64, u64, u64) {
        let ops = self.operations_count();
        let profiles: u64 = self.conn
            .query_row("SELECT COUNT(DISTINCT profile_name) FROM vehicle_profiles", [], |r| r.get(0))
            .unwrap_or(0);
        let ecus: u64 = self.conn
            .query_row("SELECT COUNT(*) FROM ecu_catalog", [], |r| r.get(0))
            .unwrap_or(0);
        (ops, profiles, ecus)
    }

    /// Search operations by keyword (name, ECU, DID)
    pub fn search_operations(&self, query: &str, limit: u32) -> Result<Vec<serde_json::Value>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT o.id, n.name, o.name_fr, o.sentbytes, o.service, o.did, o.op_type,
                    e.name, o.ecu_tx, o.ecu_rx, v.name, o.risk
             FROM operations o
             LEFT JOIN names n ON o.name_id = n.id
             LEFT JOIN ecu_names e ON o.ecu_name_id = e.id
             LEFT JOIN vehicles v ON o.vehicle_id = v.id
             WHERE n.name LIKE ?1 ESCAPE '\\' OR e.name LIKE ?1 ESCAPE '\\' OR o.did LIKE ?1 ESCAPE '\\' OR o.sentbytes LIKE ?1 ESCAPE '\\'
             LIMIT ?2"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let escaped_query = query.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped_query);
        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0).unwrap_or_default(),
                "name": row.get::<_, String>(1).unwrap_or_default(),
                "name_fr": row.get::<_, String>(2).unwrap_or_default(),
                "sentbytes": row.get::<_, String>(3).unwrap_or_default(),
                "service": row.get::<_, String>(4).unwrap_or_default(),
                "did": row.get::<_, String>(5).unwrap_or_default(),
                "type": row.get::<_, String>(6).unwrap_or_default(),
                "ecu_name": row.get::<_, String>(7).unwrap_or_default(),
                "ecu_tx": row.get::<_, String>(8).unwrap_or_default(),
                "ecu_rx": row.get::<_, String>(9).unwrap_or_default(),
                "vehicle": row.get::<_, String>(10).unwrap_or_default(),
                "risk": row.get::<_, String>(11).unwrap_or_default(),
            }))
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get operations for a vehicle make (e.g. "Peugeot")
    pub fn get_operations_for_vehicle(&self, vehicle: &str, limit: u32) -> Result<Vec<serde_json::Value>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT o.id, n.name, o.name_fr, o.sentbytes, o.service, o.did, o.op_type,
                    e.name, o.ecu_tx, o.ecu_rx, o.risk
             FROM operations o
             LEFT JOIN names n ON o.name_id = n.id
             LEFT JOIN ecu_names e ON o.ecu_name_id = e.id
             JOIN vehicles v ON o.vehicle_id = v.id AND v.name LIKE ?1 ESCAPE '\\'
             LIMIT ?2"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let escaped_vehicle = vehicle.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped_vehicle);
        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0).unwrap_or_default(),
                "name": row.get::<_, String>(1).unwrap_or_default(),
                "name_fr": row.get::<_, String>(2).unwrap_or_default(),
                "sentbytes": row.get::<_, String>(3).unwrap_or_default(),
                "service": row.get::<_, String>(4).unwrap_or_default(),
                "did": row.get::<_, String>(5).unwrap_or_default(),
                "type": row.get::<_, String>(6).unwrap_or_default(),
                "ecu_name": row.get::<_, String>(7).unwrap_or_default(),
                "ecu_tx": row.get::<_, String>(8).unwrap_or_default(),
                "ecu_rx": row.get::<_, String>(9).unwrap_or_default(),
                "risk": row.get::<_, String>(10).unwrap_or_default(),
            }))
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get read operations for ECU + vehicle (used by Live Data & Dashboard)
    pub fn get_read_operations(&self, ecu_name: &str, vehicle: &str) -> Result<Vec<serde_json::Value>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT o.id, n.name, o.name_fr, o.sentbytes, o.did, o.ecu_tx, o.ecu_rx
             FROM operations o
             LEFT JOIN names n ON o.name_id = n.id
             JOIN ecu_names e ON o.ecu_name_id = e.id AND e.name LIKE ?1 ESCAPE '\\'
             JOIN vehicles v ON o.vehicle_id = v.id AND v.name LIKE ?2 ESCAPE '\\'
             WHERE o.op_type = 'read'
             LIMIT 500"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let escaped_ecu = ecu_name.replace('%', "\\%").replace('_', "\\_");
        let escaped_vehicle = vehicle.replace('%', "\\%").replace('_', "\\_");
        let ecu_pat = format!("%{}%", escaped_ecu);
        let veh_pat = format!("%{}%", escaped_vehicle);
        let rows = stmt.query_map(params![ecu_pat, veh_pat], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0).unwrap_or_default(),
                "name": row.get::<_, String>(1).unwrap_or_default(),
                "name_fr": row.get::<_, String>(2).unwrap_or_default(),
                "sentbytes": row.get::<_, String>(3).unwrap_or_default(),
                "did": row.get::<_, String>(4).unwrap_or_default(),
                "ecu_tx": row.get::<_, String>(5).unwrap_or_default(),
                "ecu_rx": row.get::<_, String>(6).unwrap_or_default(),
            }))
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get write operations for ECU (used by Advanced page)
    pub fn get_write_operations(&self, ecu_name: &str, vehicle: &str) -> Result<Vec<serde_json::Value>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT o.id, n.name, o.name_fr, o.sentbytes, o.did, o.ecu_tx, o.ecu_rx, o.risk
             FROM operations o
             LEFT JOIN names n ON o.name_id = n.id
             JOIN ecu_names e ON o.ecu_name_id = e.id AND e.name LIKE ?1 ESCAPE '\\'
             JOIN vehicles v ON o.vehicle_id = v.id AND v.name LIKE ?2 ESCAPE '\\'
             WHERE o.op_type = 'write'
             LIMIT 500"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let escaped_ecu = ecu_name.replace('%', "\\%").replace('_', "\\_");
        let escaped_vehicle = vehicle.replace('%', "\\%").replace('_', "\\_");
        let ecu_pat = format!("%{}%", escaped_ecu);
        let veh_pat = format!("%{}%", escaped_vehicle);
        let rows = stmt.query_map(params![ecu_pat, veh_pat], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0).unwrap_or_default(),
                "name": row.get::<_, String>(1).unwrap_or_default(),
                "name_fr": row.get::<_, String>(2).unwrap_or_default(),
                "sentbytes": row.get::<_, String>(3).unwrap_or_default(),
                "did": row.get::<_, String>(4).unwrap_or_default(),
                "ecu_tx": row.get::<_, String>(5).unwrap_or_default(),
                "ecu_rx": row.get::<_, String>(6).unwrap_or_default(),
                "risk": row.get::<_, String>(7).unwrap_or_default(),
            }))
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ==================== VEHICLE PROFILES ====================

    pub fn get_vehicle_profiles(&self) -> Result<Vec<String>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT profile_name FROM vehicle_profiles ORDER BY profile_name"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let rows = stmt.query_map([], |row| row.get(0))
            .map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_profile_ecus(&self, profile_name: &str) -> Result<Vec<serde_json::Value>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT ecu_name, ecu_name_fr, tx, rx FROM vehicle_profiles WHERE profile_name = ?1"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let rows = stmt.query_map(params![profile_name], |row| {
            Ok(serde_json::json!({
                "name": row.get::<_, String>(0).unwrap_or_default(),
                "name_fr": row.get::<_, String>(1).unwrap_or_default(),
                "tx": row.get::<_, String>(2).unwrap_or_default(),
                "rx": row.get::<_, String>(3).unwrap_or_default(),
            }))
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ==================== ECU CATALOG ====================

    pub fn search_ecu_catalog(&self, query: &str, limit: u32) -> Result<Vec<serde_json::Value>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT filename, ecuname, address, group_name, protocol, projects
             FROM ecu_catalog WHERE ecuname LIKE ?1 ESCAPE '\\' OR group_name LIKE ?1 ESCAPE '\\' LIMIT ?2"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let escaped_query = query.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped_query);
        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(serde_json::json!({
                "filename": row.get::<_, String>(0).unwrap_or_default(),
                "ecuname": row.get::<_, String>(1).unwrap_or_default(),
                "address": row.get::<_, String>(2).unwrap_or_default(),
                "group": row.get::<_, String>(3).unwrap_or_default(),
                "protocol": row.get::<_, String>(4).unwrap_or_default(),
                "projects": row.get::<_, String>(5).unwrap_or_default(),
            }))
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ==================== USER DATA ====================

    pub fn save_dtc(&self, code: &str, desc: &str, status: &str, source: &str, vin: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO dtc_history (code, description, status, source, vehicle_vin) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![code, desc, status, source, vin],
            )
            .map_err(|e| format!("Failed to save DTC: {}", e))?;
        Ok(())
    }

    pub fn save_session(&self, vin: &str, make: &str, model: &str, dtc_count: i32, notes: &str, data: &str) -> Result<i64, String> {
        self.conn
            .execute(
                "INSERT INTO sessions (vehicle_vin, vehicle_make, vehicle_model, dtc_count, notes, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![vin, make, model, dtc_count, notes, data],
            )
            .map_err(|e| format!("Failed to save session: {}", e))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_settings(&self) -> AppSettings {
        let mut settings = AppSettings::default();
        if let Ok(lang) = self.get_setting("language") { settings.language = lang; }
        if let Ok(baud) = self.get_setting("default_baud_rate") {
            if let Ok(v) = baud.parse() { settings.default_baud_rate = v; }
        }
        if let Ok(theme) = self.get_setting("theme") { settings.theme = theme; }
        if let Ok(auto_connect) = self.get_setting("auto_connect") { settings.auto_connect = auto_connect.parse().unwrap_or(false); }
        settings
    }

    fn get_setting(&self, key: &str) -> Result<String, String> {
        self.conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| row.get(0))
            .map_err(|e| format!("Setting '{}' not found: {}", key, e))
    }

    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)", params![key, value])
            .map_err(|e| format!("Failed to save setting: {}", e))?;
        Ok(())
    }

    pub fn save_settings_batch(&self, settings: Vec<(&str, &str)>) -> Result<(), String> {
        // Note: We use manual BEGIN/COMMIT because rusqlite's transaction() requires &mut self,
        // but this method only has &self (the Connection is behind a Mutex already).
        // ROLLBACK is handled explicitly on error.
        self.conn.execute_batch("BEGIN").map_err(|e| format!("Failed to begin: {}", e))?;
        for (key, value) in settings {
            if let Err(e) = self.conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)", params![key, value]) {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(format!("Failed to save '{}': {}", key, e));
            }
        }
        self.conn.execute_batch("COMMIT").map_err(|e| format!("Failed to commit: {}", e))?;
        Ok(())
    }

    // ==================== SESSIONS ====================

    pub fn get_sessions(&self) -> Result<Vec<serde_json::Value>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT id, vehicle_vin, vehicle_make, vehicle_model, dtc_count, notes, timestamp
             FROM sessions ORDER BY timestamp DESC LIMIT 100"
        ).map_err(|e| format!("Query failed: {}", e))?;

        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "vin": row.get::<_, String>(1).unwrap_or_default(),
                "make": row.get::<_, String>(2).unwrap_or_default(),
                "model": row.get::<_, String>(3).unwrap_or_default(),
                "dtc_count": row.get::<_, i32>(4).unwrap_or(0),
                "notes": row.get::<_, String>(5).unwrap_or_default(),
                "timestamp": row.get::<_, String>(6).unwrap_or_default(),
            }))
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_session(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete session: {}", e))?;
        Ok(())
    }

    /// VIN alone cannot reliably determine the exact vehicle model.
    /// The make (from WMI) is sufficient for display — returns None always.
    pub fn find_vehicle_model(&self, _make: &str) -> Option<String> {
        None
    }

    /// Look up DID info from the database for a specific vehicle
    /// Returns (name_en, name_fr, ecu_name) if found
    pub fn get_did_info(&self, did: &str, vehicle: &str) -> Option<(String, String, String)> {
        if did.is_empty() || vehicle.is_empty() {
            return None;
        }
        let escaped_vehicle = vehicle.replace('%', "\\%").replace('_', "\\_");
        let veh_pattern = format!("%{}%", escaped_vehicle);
        self.conn
            .query_row(
                "SELECT n.name, o.name_fr, e.name
                 FROM operations o
                 LEFT JOIN names n ON o.name_id = n.id
                 LEFT JOIN ecu_names e ON o.ecu_name_id = e.id
                 JOIN vehicles v ON o.vehicle_id = v.id AND v.name LIKE ?2 ESCAPE '\\'
                 WHERE o.service = '22' AND o.did = ?1 AND o.op_type = 'read'
                 LIMIT 1",
                params![did, veh_pattern],
                |row| Ok((
                    row.get::<_, String>(0).unwrap_or_default(),
                    row.get::<_, String>(1).unwrap_or_default(),
                    row.get::<_, String>(2).unwrap_or_default(),
                )),
            )
            .ok()
    }

    /// Search for DTC-related info from the operations database
    /// Looks for operations with service 19 (ReadDTCInformation) that might provide context
    pub fn search_dtc_context(&self, vehicle: &str) -> Vec<(String, String, String)> {
        if vehicle.is_empty() {
            return Vec::new();
        }
        let escaped_vehicle = vehicle.replace('%', "\\%").replace('_', "\\_");
        let veh_pattern = format!("%{}%", escaped_vehicle);
        let mut stmt = match self.conn.prepare(
            "SELECT DISTINCT n.name, o.name_fr, e.name
             FROM operations o
             LEFT JOIN names n ON o.name_id = n.id
             LEFT JOIN ecu_names e ON o.ecu_name_id = e.id
             JOIN vehicles v ON o.vehicle_id = v.id AND v.name LIKE ?1 ESCAPE '\\'
             WHERE o.service = '19'
             LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        stmt.query_map(params![veh_pattern], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
                row.get::<_, String>(2).unwrap_or_default(),
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }
}
