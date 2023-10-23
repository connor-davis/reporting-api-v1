CREATE EXTENSION pg_trgm;

CREATE TABLE IF NOT EXISTS cybercns_agents (
    id TEXT PRIMARY KEY NOT NULL,
    host_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cybercns_hosts (
    id SERIAL PRIMARY KEY NOT NULL,
    host_name TEXT
);

CREATE TABLE IF NOT EXISTS cybercns_security_report_card_evidence (
    id SERIAL PRIMARY KEY NOT NULL,
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
    id SERIAL PRIMARY KEY NOT NULL,
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
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cybercns_assets (
    id TEXT PRIMARY KEY NOT NULL,
    host SERIAL,
    security_report_card SERIAL,
    company TEXT
);

CREATE TABLE rocketcyber_accounts (
    id SERIAL PRIMARY KEY NOT NULL,
    account_id BIGINT NOT NULL,
    account_name TEXT NOT NULL,
    account_path TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rocketcyber_agents (
    id TEXT PRIMARY KEY NOT NULL,
    customer_id BIGINT NOT NULL,
    hostname TEXT NOT NULL,
    account_path TEXT NOT NULL,
    agent_version TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rocketcyber_incidents (
    id BIGINT PRIMARY KEY NOT NULL,
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

CREATE TABLE IF NOT EXISTS vsa_agents (
    id TEXT PRIMARY KEY NOT NULL,
    agent_name TEXT,
    computer_name TEXT,
    ip_address TEXT,
    system_serial_number TEXT,
    system_age TEXT,
    group_id TEXT,
    organization_name TEXT,
    anti_virus BOOLEAN DEFAULT false,
    os_name TEXT,
    free_space_in_gbytes DOUBLE PRECISION DEFAULT 0.0,
    used_space_in_gbytes DOUBLE PRECISION DEFAULT 0.0,
    total_size_in_gbytes DOUBLE PRECISION DEFAULT 0.0,
    cpu_speed DOUBLE PRECISION DEFAULT 0.0,
    cpu_count DOUBLE PRECISION DEFAULT 0.0,
    ram_size_in_mbytes DOUBLE PRECISION DEFAULT 0.0,
    total_patches DOUBLE PRECISION DEFAULT 0.0,
    installed_patches DOUBLE PRECISION DEFAULT 0.0,
    last_patch TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    next_patch TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tenants (
    id SERIAL PRIMARY KEY NOT NULL,
    tenant_name TEXT NOT NULL,
    vsa_name TEXT,
    cyber_cns_name TEXT,
    rocket_cyber_name TEXT
);

CREATE TABLE IF NOT EXISTS tenants_external_scan_host_names (
    id SERIAL PRIMARY KEY NOT NULL,
    tenant_id SERIAL NOT NULL,
    host_name TEXT NOT NULL
);