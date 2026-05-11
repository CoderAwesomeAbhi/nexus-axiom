// Enterprise RBAC (Role-Based Access Control)
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    ViewDashboard,
    ViewMetrics,
    ViewLogs,
    ManageAllowlist,
    ManagePolicies,
    ManageUsers,
    ManageRoles,
    BlockExploits,
    KillProcesses,
    ModifyConfig,
    ViewAuditLogs,
    ExportData,
    ManageIntegrations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub tenant_id: String,
    pub created_at: u64,
    pub last_login: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub users: Vec<String>,
    pub quota: TenantQuota,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    pub max_servers: usize,
    pub max_events_per_day: usize,
    pub max_users: usize,
    pub retention_days: usize,
}

pub struct RBACManager {
    roles: HashMap<String, Role>,
    users: HashMap<String, User>,
    tenants: HashMap<String, Tenant>,
    db: Option<crate::database::Database>,
}

impl RBACManager {
    pub fn new() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
            users: HashMap::new(),
            tenants: HashMap::new(),
            db: None,
        };
        manager.init_default_roles();
        manager
    }

    pub fn with_database(db_path: &str) -> anyhow::Result<Self> {
        let db = crate::database::Database::new(db_path)?;
        let mut manager = Self {
            roles: HashMap::new(),
            users: HashMap::new(),
            tenants: HashMap::new(),
            db: Some(db),
        };
        manager.init_default_roles();
        manager.load_from_database()?;
        Ok(manager)
    }

    fn load_from_database(&mut self) -> anyhow::Result<()> {
        if let Some(db) = &self.db {
            // Load roles
            for role_name in &["admin", "analyst", "operator"] {
                if let Some(role) = db.get_role(role_name)? {
                    self.roles.insert(role.name.clone(), role);
                }
            }
            log::info!("Loaded {} roles from database", self.roles.len());
        }
        Ok(())
    }

    fn init_default_roles(&mut self) {
        // Admin role
        let mut admin_perms = HashSet::new();
        admin_perms.insert(Permission::ViewDashboard);
        admin_perms.insert(Permission::ViewMetrics);
        admin_perms.insert(Permission::ViewLogs);
        admin_perms.insert(Permission::ManageAllowlist);
        admin_perms.insert(Permission::ManagePolicies);
        admin_perms.insert(Permission::ManageUsers);
        admin_perms.insert(Permission::ManageRoles);
        admin_perms.insert(Permission::BlockExploits);
        admin_perms.insert(Permission::KillProcesses);
        admin_perms.insert(Permission::ModifyConfig);
        admin_perms.insert(Permission::ViewAuditLogs);
        admin_perms.insert(Permission::ExportData);
        admin_perms.insert(Permission::ManageIntegrations);

        self.roles.insert(
            "admin".to_string(),
            Role {
                name: "admin".to_string(),
                permissions: admin_perms,
                description: "Full system access".to_string(),
            },
        );

        // Security Analyst role
        let mut analyst_perms = HashSet::new();
        analyst_perms.insert(Permission::ViewDashboard);
        analyst_perms.insert(Permission::ViewMetrics);
        analyst_perms.insert(Permission::ViewLogs);
        analyst_perms.insert(Permission::ViewAuditLogs);
        analyst_perms.insert(Permission::ExportData);

        self.roles.insert(
            "analyst".to_string(),
            Role {
                name: "analyst".to_string(),
                permissions: analyst_perms,
                description: "Read-only access for security analysis".to_string(),
            },
        );

        // Operator role
        let mut operator_perms = HashSet::new();
        operator_perms.insert(Permission::ViewDashboard);
        operator_perms.insert(Permission::ViewMetrics);
        operator_perms.insert(Permission::ManageAllowlist);
        operator_perms.insert(Permission::BlockExploits);

        self.roles.insert(
            "operator".to_string(),
            Role {
                name: "operator".to_string(),
                permissions: operator_perms,
                description: "Day-to-day operations".to_string(),
            },
        );
    }

    pub fn check_permission(&self, user_id: &str, permission: Permission) -> bool {
        if let Some(user) = self.users.get(user_id) {
            for role_name in &user.roles {
                if let Some(role) = self.roles.get(role_name) {
                    if role.permissions.contains(&permission) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn add_user(&mut self, user: User) -> anyhow::Result<()> {
        // Validate input
        crate::validation::Validator::validate_user_id(&user.id)?;
        crate::validation::Validator::validate_email(&user.email)?;
        crate::validation::Validator::validate_tenant_id(&user.tenant_id)?;

        // Check tenant exists and quota
        if let Some(tenant) = self.tenants.get(&user.tenant_id) {
            let current_users = self
                .users
                .values()
                .filter(|u| u.tenant_id == user.tenant_id)
                .count();

            if current_users >= tenant.quota.max_users {
                anyhow::bail!("Tenant user quota exceeded");
            }
        }

        // Save to database first
        if let Some(db) = &self.db {
            db.save_user(&user)?;
        }

        // Then update in-memory
        self.users.insert(user.id.clone(), user);
        Ok(())
    }

    pub fn add_tenant(&mut self, tenant: Tenant) -> anyhow::Result<()> {
        // Validate input
        crate::validation::Validator::validate_tenant_id(&tenant.id)?;

        if tenant.name.is_empty() || tenant.name.len() > 100 {
            anyhow::bail!("Invalid tenant name");
        }

        // Save to database first
        if let Some(db) = &self.db {
            db.save_tenant(&tenant)?;
        }

        // Then update in-memory
        self.tenants.insert(tenant.id.clone(), tenant);
        Ok(())
    }

    pub fn check_tenant_quota(&self, tenant_id: &str, servers: usize) -> bool {
        if let Some(tenant) = self.tenants.get(tenant_id) {
            return servers <= tenant.quota.max_servers;
        }
        false
    }
}

impl Default for RBACManager {
    fn default() -> Self {
        Self::new()
    }
}
