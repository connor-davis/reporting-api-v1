DROP TABLE rocketcyber_incidents;

CREATE TABLE rocketcyber_incidents (
    id BIGINT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    remediation TEXT NOT NULL,
    resolved_at TIMESTAMP NOT NULL,
    published_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    status TEXT NOT NULL,
    account_id BIGINT NOT NULL,
    event_count BIGINT NOT NULL
);