use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberSecurityReportCardEvidence {
    pub id: i32,
    pub anti_virus: String,
    pub local_firewall: String,
    pub insecure_listening_ports: String,
    pub failed_login: String,
    pub network_vulnerabilities: String,
    pub system_aging: String,
    pub supported_os: String,
    pub backup_softwares: String,
}
