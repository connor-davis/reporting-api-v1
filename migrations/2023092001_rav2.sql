CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE
    IF NOT EXISTS cybercns_agents (
        id TEXT PRIMARY KEY NOT NULL,
        host_name TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS cybercns_hosts (id SERIAL PRIMARY KEY NOT NULL, host_name TEXT);

CREATE TABLE
    IF NOT EXISTS cybercns_security_report_card_evidence (
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

CREATE TABLE
    IF NOT EXISTS cybercns_security_report_card (
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

CREATE TABLE
    IF NOT EXISTS cybercns_vulnerabilities (
        id TEXT PRIMARY KEY NOT NULL,
        title TEXT DEFAULT 'Data not found from CyberCNS.',
        severity TEXT DEFAULT 'Data not found from CyberCNS.',
        vector TEXT DEFAULT 'Data not found from CyberCNS.',
        product TEXT DEFAULT 'Data not found from CyberCNS.',
        base_score DOUBLE PRECISION DEFAULT 0.0,
        impact_score DOUBLE PRECISION DEFAULT 0.0,
        exploit_score DOUBLE PRECISION DEFAULT 0.0,
        cvss_score DOUBLE PRECISION DEFAULT 0.0,
        asset_id TEXT NOT NULL,
        company_id TEXT NOT NULL,
        company_name TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS cybercns_companies (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL);

CREATE TABLE
    IF NOT EXISTS cybercns_assets (
        id TEXT PRIMARY KEY NOT NULL,
        host SERIAL,
        security_report_card SERIAL,
        company TEXT
    );

CREATE TABLE
    IF NOT EXISTS rocketcyber_accounts (
        id SERIAL PRIMARY KEY NOT NULL,
        account_id BIGINT NOT NULL,
        account_name TEXT NOT NULL,
        account_path TEXT NOT NULL,
        status TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS rocketcyber_agents (
        id TEXT PRIMARY KEY NOT NULL,
        customer_id BIGINT NOT NULL,
        hostname TEXT NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW (),
        operating_system TEXT NOT NULL,
        account_path TEXT NOT NULL,
        agent_version TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS rocketcyber_incidents (
        id BIGINT PRIMARY KEY NOT NULL,
        title TEXT NOT NULL,
        description TEXT NOT NULL,
        remediation TEXT NOT NULL,
        resolved_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW (),
            published_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW (),
            created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW (),
            status TEXT NOT NULL,
            account_id BIGINT NOT NULL,
            event_count BIGINT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS vsa_agents (
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
        last_patch TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW (),
            next_patch TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW ()
    );

--  {
--  "PartitionId": "string",
--  "AssetAgentId": "string",
--  "AssetMachineGroupName": "string",
--  "TicketId": "string",
--  "TicketNumber": "string",
--  "ServiceDesk": "string",
--  "Summary": "string",
--  "Status": "string",
--  "Stage": "string",
--  "Priority": "string",
--  "Severity": "string",
--  "Category": "string",
--  "Resolution": "string",
--  "ResolutionDescription": "string",
--  "SubmitterType": "string",
--  "SubmitterName": "string",
--  "SubmitterEmail": "string",
--  "SubmitterPhone": "string",
--  "ContactName": "string",
--  "ContactEmail": "string",
--  "ContactPhone": "string",
--  "Assignee": "string",
--  "Owner": "string",
--  "Organization": "string",
--  "CreationDateTime": "2022-01-14T07:39:46.706Z",
--  "ModificationDateTime": "2022-01-14T07:39:46.706Z",
--  "ClosedDateTime": "2022-01-14T07:39:46.706Z",
--  "DueDate": "2022-01-14T07:39:46.706Z",
--  "ProjectedDate": "2022-01-14T07:39:46.706Z",
--  "LockedBy": "string",
--  "LockedOnDateTime": "2022-01-14T07:39:46.706Z",
--  "SourceType": "string",
--  "LastPublicUpdateDate": "2022-01-14T07:39:46.706Z",
--  "ResolutionDate": "2022-01-14T07:39:46.706Z",
--  "Policy": "string",
--  "Description": "string",
--  "IsArchived": "s"
--  }
CREATE TABLE
    IF NOT EXISTS vsa_tickets (
        id SERIAL PRIMARY KEY NOT NULL,
        partition_id TEXT NOT NULL DEFAULT 'N/A',
        asset_agent_id TEXT NOT NULL DEFAULT 'N/A',
        asset_machine_group_name TEXT NOT NULL DEFAULT 'N/A',
        ticket_id TEXT NOT NULL DEFAULT 'N/A',
        ticket_number TEXT NOT NULL DEFAULT 'N/A',
        service_desk TEXT NOT NULL DEFAULT 'N/A',
        summary TEXT NOT NULL DEFAULT 'N/A',
        status TEXT NOT NULL DEFAULT 'N/A',
        stage TEXT NOT NULL DEFAULT 'N/A',
        priority TEXT NOT NULL DEFAULT 'N/A',
        severity TEXT NOT NULL DEFAULT 'N/A',
        category TEXT NOT NULL DEFAULT 'N/A',
        resolution TEXT NOT NULL DEFAULT 'N/A',
        resolution_description TEXT NOT NULL DEFAULT 'N/A',
        submitter_type TEXT NOT NULL DEFAULT 'N/A',
        submitter_name TEXT NOT NULL DEFAULT 'N/A',
        submitter_email TEXT NOT NULL DEFAULT 'N/A',
        submitter_phone TEXT NOT NULL DEFAULT 'N/A',
        contact_name TEXT NOT NULL DEFAULT 'N/A',
        contact_email TEXT NOT NULL DEFAULT 'N/A',
        contact_phone TEXT NOT NULL DEFAULT 'N/A',
        assignee TEXT NOT NULL DEFAULT 'N/A',
        owner TEXT NOT NULL DEFAULT 'N/A',
        organization TEXT NOT NULL DEFAULT 'N/A',
        creation_date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            modification_date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            closed_date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            due_date TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            projected_date TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            locked_by TEXT NOT NULL DEFAULT 'N/A',
            locked_on_date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            source_type TEXT NOT NULL DEFAULT 'N/A',
            last_public_update_date TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            resolution_date TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            policy TEXT NOT NULL DEFAULT 'N/A',
            description TEXT NOT NULL DEFAULT 'N/A',
            is_archived TEXT NOT NULL DEFAULT 'N/A'
    );

CREATE TABLE
    IF NOT EXISTS tenants (
        id SERIAL PRIMARY KEY NOT NULL,
        tenant_name TEXT NOT NULL,
        vsa_name TEXT,
        cyber_cns_name TEXT,
        rocket_cyber_name TEXT,
        spanning_name TEXT,
        spanning_key TEXT,
        veeam_url TEXT,
        veeam_key TEXT
    );

CREATE TABLE
    IF NOT EXISTS tenants_external_scan_host_names (
        id SERIAL PRIMARY KEY NOT NULL,
        tenant_id SERIAL NOT NULL,
        host_name TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS spanning_backups (
        id SERIAL PRIMARY KEY NOT NULL,
        user_principal_name TEXT NOT NULL DEFAULT 'N/A',
        user_display_name TEXT NOT NULL DEFAULT 'N/A',
        email TEXT NOT NULL,
        ms_id TEXT NOT NULL,
        assigned BOOLEAN NOT NULL,
        is_admin BOOLEAN NOT NULL,
        is_deleted BOOLEAN NOT NULL,
        company_name TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS spanning_backups_summaries (
        id SERIAL PRIMARY KEY NOT NULL,
        date TEXT NOT NULL,
        backup_type TEXT NOT NULL,
        total TEXT NOT NULL,
        partial BIGINT NOT NULL DEFAULT 0,
        failed BIGINT NOT NULL DEFAULT 0,
        successful BIGINT NOT NULL DEFAULT 0,
        data_created BIGINT NOT NULL DEFAULT 0,
        data_deleted BIGINT NOT NULL DEFAULT 0,
        data_failed BIGINT NOT NULL DEFAULT 0,
        data_total BIGINT NOT NULL DEFAULT 0,
        data_attempts BIGINT NOT NULL DEFAULT 0,
        backup SERIAL NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS veeam_agents (
        id SERIAL PRIMARY KEY NOT NULL,
        instance_uid TEXT NOT NULL DEFAULT 'N/A',
        agent_platform TEXT NOT NULL DEFAULT 'N/A',
        status TEXT NOT NULL DEFAULT 'N/A',
        management_agent_uid TEXT NOT NULL DEFAULT 'N/A',
        site_uid TEXT NOT NULL DEFAULT 'N/A',
        organization_uid TEXT NOT NULL DEFAULT 'N/A',
        name TEXT NOT NULL DEFAULT 'N/A',
        operation_mode TEXT NOT NULL DEFAULT 'N/A',
        gui_mode TEXT NOT NULL DEFAULT 'N/A',
        platform TEXT NOT NULL DEFAULT 'N/A',
        version TEXT NOT NULL DEFAULT 'N/A',
        activation_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            management_mode TEXT NOT NULL DEFAULT 'N/A',
            installation_type TEXT NOT NULL DEFAULT 'N/A',
            total_jobs_count BIGINT NOT NULL DEFAULT 0,
            running_jobs_count BIGINT NOT NULL DEFAULT 0,
            success_jobs_count BIGINT NOT NULL DEFAULT 0,
            company_name TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS veeam_agents_jobs (
        id SERIAL PRIMARY KEY NOT NULL,
        instance_uid TEXT NOT NULL DEFAULT 'N/A',
        backup_agent_uid TEXT NOT NULL DEFAULT 'N/A',
        organization_uid TEXT NOT NULL DEFAULT 'N/A',
        name TEXT NOT NULL DEFAULT 'N/A',
        description TEXT NOT NULL DEFAULT 'N/A',
        config_uid TEXT NOT NULL DEFAULT 'N/A',
        system_type TEXT NOT NULL DEFAULT 'N/A',
        backup_policy_uid TEXT NOT NULL DEFAULT 'N/A',
        backup_policy_failure_message TEXT NOT NULL DEFAULT 'N/A',
        status TEXT NOT NULL DEFAULT 'N/A',
        operation_mode TEXT NOT NULL DEFAULT 'N/A',
        destination TEXT NOT NULL DEFAULT 'N/A',
        restore_points BIGINT NOT NULL DEFAULT 0,
        last_run TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            last_end_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            last_duration BIGINT NOT NULL DEFAULT 0,
            next_run TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            avg_duration BIGINT NOT NULL DEFAULT 0,
            backup_mode TEXT NOT NULL DEFAULT 'N/A',
            target_type TEXT NOT NULL DEFAULT 'N/A',
            is_enabled BOOLEAN NOT NULL DEFAULT false,
            schedule_type TEXT NOT NULL DEFAULT 'N/A',
            failure_message TEXT NOT NULL DEFAULT 'N/A',
            backed_up_size BIGINT NOT NULL DEFAULT 0,
            company_name TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS veeam_servers (
        id SERIAL PRIMARY KEY NOT NULL,
        instance_uid TEXT NOT NULL DEFAULT 'N/A',
        name TEXT NOT NULL DEFAULT 'N/A',
        organization_uid TEXT NOT NULL DEFAULT 'N/A',
        location_uid TEXT NOT NULL DEFAULT 'N/A',
        management_agent_uid TEXT NOT NULL DEFAULT 'N/A',
        version TEXT NOT NULL DEFAULT 'N/A',
        display_version TEXT NOT NULL DEFAULT 'N/A',
        installation_uid TEXT NOT NULL DEFAULT 'N/A',
        backup_server_role_type TEXT NOT NULL DEFAULT 'N/A',
        status TEXT NOT NULL DEFAULT 'N/A',
        company_name TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS veeam_servers_jobs (
        id SERIAL PRIMARY KEY NOT NULL,
        instance_uid TEXT NOT NULL DEFAULT 'N/A',
        name TEXT NOT NULL DEFAULT 'N/A',
        backup_server_uid TEXT NOT NULL DEFAULT 'N/A',
        location_uid TEXT NOT NULL DEFAULT 'N/A',
        site_uid TEXT NOT NULL DEFAULT 'N/A',
        organization_uid TEXT NOT NULL DEFAULT 'N/A',
        status TEXT NOT NULL DEFAULT 'N/A',
        type TEXT NOT NULL DEFAULT 'N/A',
        last_run TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            last_end_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            last_duration BIGINT NOT NULL DEFAULT 0,
            processing_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
            avg_duration BIGINT NOT NULL DEFAULT 0,
            transferred_data BIGINT NOT NULL DEFAULT 0,
            bottleneck TEXT NOT NULL DEFAULT 'N/A',
            is_enabled BOOLEAN NOT NULL DEFAULT false,
            schedule_type TEXT NOT NULL DEFAULT 'N/A',
            failure_message TEXT NOT NULL DEFAULT 'N/A',
            target_type TEXT NOT NULL DEFAULT 'N/A',
            destination TEXT NOT NULL DEFAULT 'N/A',
            retention_limit BIGINT NOT NULL DEFAULT 0,
            retention_limit_type TEXT NOT NULL DEFAULT 'N/A',
            is_gfs_option_enabled BOOLEAN NOT NULL DEFAULT false,
            company_name TEXT NOT NULL
    );