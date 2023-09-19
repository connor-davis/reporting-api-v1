CREATE TABLE IF NOT EXISTS cybercns_agents (
    id TEXT PRIMARY KEY,
    host_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cybercns_hosts (
    id SERIAL PRIMARY KEY,
    host_name TEXT
);

CREATE TABLE IF NOT EXISTS cybercns_security_report_card_evidence (
    id SERIAL PRIMARY KEY,
    anti_virus TEXT DEFAULT 'Data not found from CyberCNS.',
    local_firewall TEXT DEFAULT 'Data not found from CyberCNS.',
    insecure_listening_ports TEXT DEFAULT 'Data not found from CyberCNS.',
    failed_login TEXT DEFAULT 'Data not found from CyberCNS.',
    network_vulnerabilities TEXT DEFAULT 'Data not found from CyberCNS.',
    system_aging TEXT DEFAULT 'Data not found from CyberCNS.',
    supported_os TEXT DEFAULT 'Data not found from CyberCNS.',
    backup_softwares TEXT DEFAULT 'Data not found from CyberCNS.'
);

CREATE TABLE IF NOT EXISTS cybercns_security_report_card (
    id SERIAL PRIMARY KEY,
    anti_virus DOUBLE PRECISION DEFAULT 0.0,
    local_firewall DOUBLE PRECISION DEFAULT 0.0,
    insecure_listening_ports DOUBLE PRECISION DEFAULT 0.0,
    failed_login DOUBLE PRECISION DEFAULT 0.0,
    network_vulnerabilities DOUBLE PRECISION DEFAULT 0.0,
    system_aging DOUBLE PRECISION DEFAULT 0.0,
    supported_os DOUBLE PRECISION DEFAULT 0.0,
    backup_softwares DOUBLE PRECISION DEFAULT 0.0,
    evidence BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS cybercns_companies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cybercns_assets (
    id TEXT PRIMARY KEY,
    host SERIAL,
    security_report_card SERIAL,
    company TEXT
);

CREATE TABLE rocketcyber_agents (
    id TEXT PRIMARY KEY,
    customer_id BIGINT NOT NULL,
    hostname TEXT NOT NULL,
    account_path TEXT NOT NULL,
    agent_version TEXT NOT NULL
);

CREATE TABLE rocketcyber_incidents (
    id BIGINT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    remediation TEXT NOT NULL,
    resolved_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    published_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    status TEXT NOT NULL,
    account_id BIGINT NOT NULL,
    event_count BIGINT NOT NULL
);