use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberSecurityReportCard {
    pub id: i32,
    pub anti_virus: f64,
    pub local_firewall: f64,
    pub insecure_listening_ports: f64,
    pub failed_login: f64,
    pub network_vulnerabilities: f64,
    pub system_aging: f64,
    pub supported_os: f64,
    pub backup_softwares: f64,
    pub evidence: i32,
}
