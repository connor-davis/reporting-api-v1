use std::collections::HashMap;

use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
struct Tenant {
    tenant_name: String,
    tags: Vec<String>,
}

pub async fn all(State(pool): State<PgPool>) -> impl IntoResponse {
    let vsa_organizations_result = sqlx::query!("SELECT organization_name FROM vsa_agents;")
        .fetch_all(&pool)
        .await
        .expect("Failed to get vsa organization names from postgres.");
    let cybercns_companies_result = sqlx::query!("SELECT name FROM cybercns_companies;")
        .fetch_all(&pool)
        .await
        .expect("Failed to get cybercns company names from postgres.");
    let rocketcyber_accounts_result =
        sqlx::query!("SELECT account_name FROM rocketcyber_accounts;")
            .fetch_all(&pool)
            .await
            .expect("Failed to get rocketcyber account names from postgres.");

    let mut tenants_map: HashMap<String, Tenant> = HashMap::new();

    for vsa_organization in vsa_organizations_result {
        let organization_name = vsa_organization.organization_name.clone();

        if organization_name.is_none() {
            continue;
        }

        // Replace (Pty) Ltd in organization_name
        let organization_name = organization_name.unwrap().replace(" (Pty) Ltd", "");

        let tenant = Tenant {
            tenant_name: organization_name,
            tags: vec!["vsa".to_string()],
        };

        tenants_map.insert(tenant.tenant_name.clone(), tenant);
    }

    for cybercns_company in cybercns_companies_result {
        let company_name = cybercns_company.name.clone();

        // Replace (Pty) Ltd in company_name
        let company_name = company_name.replace(" (Pty) Ltd", "");

        if !tenants_map.contains_key(&company_name) {
            let tenant = Tenant {
                tenant_name: company_name,
                tags: vec!["cybercns".to_string()],
            };

            tenants_map.insert(tenant.tenant_name.clone(), tenant);
        } else {
            let tenant = tenants_map.get_mut(&company_name).unwrap();

            tenant.tags.push("cybercns".to_string());
        }
    }

    for rocketcyber_account in rocketcyber_accounts_result {
        let account_name = rocketcyber_account.account_name.clone();

        // Replace (Pty) Ltd in account_name
        let account_name = account_name.replace(" (Pty) Ltd", "");

        if !tenants_map.contains_key(&account_name) {
            let tenant = Tenant {
                tenant_name: account_name,
                tags: vec!["rocketcyber".to_string()],
            };

            tenants_map.insert(tenant.tenant_name.clone(), tenant);
        } else {
            let tenant = tenants_map.get_mut(&account_name).unwrap();

            tenant.tags.push("rocketcyber".to_string());
        }
    }

    let mut tenants: Vec<Tenant> = Vec::new();

    for (_, tenant) in tenants_map {
        tenants.push(tenant);
    }

    Json(tenants)
}
