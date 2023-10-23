use std::collections::HashMap;

use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
struct Tenant {
    id: i64,
    tenant_name: String,
    vsa_name: Option<String>,
    cyber_cns_name: Option<String>,
    rocket_cyber_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct TaggedTenant {
    tenant_id: i64,
    tenant_name: String,
    tenant_tags: Vec<String>,
}

pub async fn all(State(pool): State<PgPool>) -> impl IntoResponse {
    let tenants_result = sqlx::query_as!(Tenant, "SELECT * FROM tenants;")
        .fetch_all(&pool)
        .await;
    let mut tagged_tenants: Vec<TaggedTenant> = Vec::new();

    match tenants_result {
        Ok(tenants) => {
            for tenant in tenants {
                let mut tenant_tags: Vec<String> = Vec::new();

                if tenant.vsa_name.is_some() {
                    tenant_tags.push("vsa".to_string())
                }

                if tenant.cyber_cns_name.is_some() {
                    tenant_tags.push("cybercns".to_string())
                }

                if tenant.rocket_cyber_name.is_some() {
                    tenant_tags.push("rocketcyber".to_string())
                }

                tagged_tenants.push(TaggedTenant {
                    tenant_id: tenant.id,
                    tenant_name: tenant.tenant_name,
                    tenant_tags,
                })
            }
        }
        Err(_) => {}
    }

    Json(tagged_tenants)
}

#[derive(Debug, Serialize)]
struct SmartTenant {
    tenant_name: String,
    vsa_name: Option<String>,
    cns_name: Option<String>,
    rocket_name: Option<String>,
    tags: Vec<String>,
}

pub async fn smart_find(State(pool): State<PgPool>) -> impl IntoResponse {
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

    let mut tenants_map: HashMap<String, SmartTenant> = HashMap::new();

    for vsa_organization in vsa_organizations_result {
        let organization_name = vsa_organization.organization_name.clone();

        if organization_name.is_none() {
            continue;
        }

        let tenant = SmartTenant {
            tenant_name: organization_name.clone().unwrap().replace(" (Pty) Ltd", ""),
            vsa_name: organization_name,
            cns_name: None,
            rocket_name: None,
            tags: vec!["vsa".to_string()],
        };

        tenants_map.insert(tenant.tenant_name.clone(), tenant);
    }

    for cybercns_company in cybercns_companies_result {
        let company_name = cybercns_company.name.clone();

        if !tenants_map.contains_key(&company_name) {
            let tenant = SmartTenant {
                tenant_name: company_name.replace(" (Pty) Ltd", ""),
                vsa_name: None,
                cns_name: Some(company_name),
                rocket_name: None,
                tags: vec!["cybercns".to_string()],
            };

            tenants_map.insert(tenant.tenant_name.clone(), tenant);
        } else {
            let tenant = tenants_map.get_mut(&company_name).unwrap();

            tenant.cns_name = Some(company_name);
            tenant.tags.push("cybercns".to_string());
        }
    }

    for rocketcyber_account in rocketcyber_accounts_result {
        let account_name = rocketcyber_account.account_name.clone();

        if !tenants_map.contains_key(&account_name) {
            let tenant = SmartTenant {
                tenant_name: account_name.replace(" (Pty) Ltd", ""),
                vsa_name: None,
                cns_name: None,
                rocket_name: Some(account_name),
                tags: vec!["rocketcyber".to_string()],
            };

            tenants_map.insert(tenant.tenant_name.clone(), tenant);
        } else {
            let tenant = tenants_map.get_mut(&account_name).unwrap();

            tenant.rocket_name = Some(account_name);
            tenant.tags.push("rocketcyber".to_string());
        }
    }

    let mut tenants: Vec<SmartTenant> = Vec::new();

    for (_, tenant) in tenants_map {
        tenants.push(tenant);
    }

    Json(tenants)
}
