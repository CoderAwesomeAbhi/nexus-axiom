// Database layer for RBAC persistence
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

// Using rusqlite for embedded database (no external dependencies)
use rusqlite::{params, Connection};

use crate::rbac::{Permission, Role, Tenant, User};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                roles TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_login INTEGER
            );
            
            CREATE TABLE IF NOT EXISTS roles (
                name TEXT PRIMARY KEY,
                permissions TEXT NOT NULL,
                description TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS tenants (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                users TEXT NOT NULL,
                max_servers INTEGER NOT NULL,
                max_events_per_day INTEGER NOT NULL,
                max_users INTEGER NOT NULL,
                retention_days INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
            CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);",
        )?;
        Ok(())
    }

    // User operations
    pub fn save_user(&self, user: &User) -> Result<()> {
        let roles_json = serde_json::to_string(&user.roles)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO users (id, email, roles, tenant_id, created_at, last_login)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &user.id,
                &user.email,
                &roles_json,
                &user.tenant_id,
                user.created_at as i64,
                user.last_login.map(|t| t as i64)
            ],
        )?;
        Ok(())
    }

    pub fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, email, roles, tenant_id, created_at, last_login FROM users WHERE id = ?1",
        )?;

        let user = stmt
            .query_row(params![user_id], |row| {
                let roles_json: String = row.get(2)?;
                let roles: Vec<String> = serde_json::from_str(&roles_json).unwrap_or_default();

                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    roles,
                    tenant_id: row.get(3)?,
                    created_at: row.get::<_, i64>(4)? as u64,
                    last_login: row.get::<_, Option<i64>>(5)?.map(|t| t as u64),
                })
            })
            .optional()?;

        Ok(user)
    }

    pub fn delete_user(&self, user_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM users WHERE id = ?1", params![user_id])?;
        Ok(())
    }

    pub fn list_users(&self, tenant_id: &str) -> Result<Vec<User>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, email, roles, tenant_id, created_at, last_login 
             FROM users WHERE tenant_id = ?1",
        )?;

        let users = stmt
            .query_map(params![tenant_id], |row| {
                let roles_json: String = row.get(2)?;
                let roles: Vec<String> = serde_json::from_str(&roles_json).unwrap_or_default();

                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    roles,
                    tenant_id: row.get(3)?,
                    created_at: row.get::<_, i64>(4)? as u64,
                    last_login: row.get::<_, Option<i64>>(5)?.map(|t| t as u64),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    // Role operations
    pub fn save_role(&self, role: &Role) -> Result<()> {
        let permissions_json = serde_json::to_string(&role.permissions)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO roles (name, permissions, description)
             VALUES (?1, ?2, ?3)",
            params![&role.name, &permissions_json, &role.description],
        )?;
        Ok(())
    }

    pub fn get_role(&self, name: &str) -> Result<Option<Role>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, permissions, description FROM roles WHERE name = ?1")?;

        let role = stmt
            .query_row(params![name], |row| {
                let permissions_json: String = row.get(1)?;
                let permissions = serde_json::from_str(&permissions_json).unwrap_or_default();

                Ok(Role {
                    name: row.get(0)?,
                    permissions,
                    description: row.get(2)?,
                })
            })
            .optional()?;

        Ok(role)
    }

    // Tenant operations
    pub fn save_tenant(&self, tenant: &Tenant) -> Result<()> {
        let users_json = serde_json::to_string(&tenant.users)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO tenants 
             (id, name, users, max_servers, max_events_per_day, max_users, retention_days, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &tenant.id,
                &tenant.name,
                &users_json,
                tenant.quota.max_servers as i64,
                tenant.quota.max_events_per_day as i64,
                tenant.quota.max_users as i64,
                tenant.quota.retention_days as i64,
                tenant.created_at as i64
            ],
        )?;
        Ok(())
    }

    pub fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, users, max_servers, max_events_per_day, max_users, retention_days, created_at
             FROM tenants WHERE id = ?1"
        )?;

        let tenant = stmt
            .query_row(params![tenant_id], |row| {
                let users_json: String = row.get(2)?;
                let users = serde_json::from_str(&users_json).unwrap_or_default();

                Ok(Tenant {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    users,
                    quota: crate::rbac::TenantQuota {
                        max_servers: row.get::<_, i64>(3)? as usize,
                        max_events_per_day: row.get::<_, i64>(4)? as usize,
                        max_users: row.get::<_, i64>(5)? as usize,
                        retention_days: row.get::<_, i64>(6)? as usize,
                    },
                    created_at: row.get::<_, i64>(7)? as u64,
                })
            })
            .optional()?;

        Ok(tenant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rbac::{Permission, Role, Tenant, TenantQuota, User};
    use std::collections::HashSet;

    #[test]
    fn test_user_crud() {
        let db = Database::new(":memory:").unwrap();

        let user = User {
            id: "user1".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["admin".to_string()],
            tenant_id: "tenant1".to_string(),
            created_at: 1234567890,
            last_login: Some(1234567900),
        };

        // Create
        db.save_user(&user).unwrap();

        // Read
        let loaded = db.get_user("user1").unwrap().unwrap();
        assert_eq!(loaded.email, "test@example.com");

        // Delete
        db.delete_user("user1").unwrap();
        assert!(db.get_user("user1").unwrap().is_none());
    }

    #[test]
    fn test_role_crud() {
        let db = Database::new(":memory:").unwrap();

        let mut permissions = HashSet::new();
        permissions.insert(Permission::ViewDashboard);

        let role = Role {
            name: "test_role".to_string(),
            permissions,
            description: "Test role".to_string(),
        };

        db.save_role(&role).unwrap();
        let loaded = db.get_role("test_role").unwrap().unwrap();
        assert_eq!(loaded.description, "Test role");
    }
}
