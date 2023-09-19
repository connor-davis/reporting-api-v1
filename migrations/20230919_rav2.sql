CREATE TABLE rocketcyber_agents (
    id TEXT PRIMARY KEY,
    customer_id BIGINT NOT NULL,
    hostname TEXT NOT NULL,
    account_path TEXT NOT NULL,
    agent_version TEXT NOT NULL
)