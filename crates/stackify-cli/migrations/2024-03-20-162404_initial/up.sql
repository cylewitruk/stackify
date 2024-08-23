-- This is for an SQLite database

-- Enumeration table holding the different allowed Stacks epochs.
CREATE TABLE epoch (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    default_block_height INTEGER NOT NULL
);

INSERT INTO epoch (id, name, default_block_height) 
    VALUES 
        (0, '1.0', 0),
        (1, '2.0', 1),
        (2, '2.05', 2),
        (3, '2.1', 3),
        (4, '2.2', 4),
        (5, '2.3', 5),
        (6, '2.4', 6),
        (7, '2.5', 10),
        (8, '3.0', 15)
    ;

CREATE TABLE environment_status (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
) WITHOUT ROWID;

INSERT INTO environment_status (id, name) 
    VALUES 
        (0, 'stopped'),
        (1, 'running'),
        (2, 'error')
    ;

CREATE TABLE network_protocol (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
) WITHOUT ROWID;

INSERT INTO network_protocol (id, name) 
    VALUES 
        (0, 'tcp'),
        (1, 'udp'),
        (2, 'sctp')
    ;

CREATE TABLE environment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    bitcoin_block_speed INTEGER NOT NULL,

    UNIQUE (name)
);

CREATE TABLE environment_epoch (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_id INTEGER NOT NULL,
    epoch_id INTEGER NOT NULL,
    starts_at_block_height INTEGER NOT NULL,
    ends_at_block_height INTEGER NULL,

    UNIQUE (environment_id, epoch_id),
    FOREIGN KEY (environment_id) REFERENCES environment (id),
    FOREIGN KEY (epoch_id) REFERENCES epoch (id)
);

CREATE TABLE service_type (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    cli_name TEXT NOT NULL,
    allow_minimum_epoch BOOLEAN NOT NULL DEFAULT 1,
    allow_maximum_epoch BOOLEAN NOT NULL DEFAULT 1,
    allow_git_target BOOLEAN NOT NULL DEFAULT 1
) WITHOUT ROWID;

INSERT INTO service_type (id, name, cli_name, allow_git_target)
    VALUES (0, 'Bitcoin Miner', 'bitcoin-miner', 0);
INSERT INTO service_type (id, name, cli_name, allow_git_target)
    VALUES (1, 'Bitcoin Follower', 'bitcoin-follower', 0);
INSERT INTO service_type (id, name, cli_name)
    VALUES (2, 'Stacks Miner', 'stacks-miner');
INSERT INTO service_type (id, name, cli_name)
    VALUES (3, 'Stacks Follower', 'stacks-follower');
INSERT INTO service_type (id, name, cli_name)
    VALUES (4, 'Stacks Signer', 'stacks-signer'); -- Minimum epoch 2.5
INSERT INTO service_type (id, name, cli_name)
    VALUES (5, 'Stacks Stacker (Self)', 'stacks-stacker-self');
INSERT INTO service_type (id, name, cli_name) 
    VALUES (6, 'Stacks Stacker (Pool)', 'stacks-stacker-pool');
INSERT INTO service_type (id, name, cli_name)
    VALUES (9, 'Stacks Transaction Generator', 'stacks-tx-generator');

CREATE TABLE file_type (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,

    UNIQUE (name)
) WITHOUT ROWID;

INSERT INTO file_type (id, name) 
    VALUES 
        (0, 'Binary'),
        (1, 'Plain-Text'),
        (2, 'Handlebars Template')
    ;

CREATE TABLE value_type (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,

    UNIQUE (name)
) WITHOUT ROWID;

INSERT INTO value_type (id, name)
    VALUES 
        (0, 'String'),
        (1, 'Integer'),
        (2, 'Boolean'),
        (3, 'Enum'),
        (4, 'Stacks Keychain'),
        (5, 'Service')
    ;

-- Default service configuration file templates.
-- These will be populated by the application upon init since we use actual
-- files as the source.
CREATE TABLE service_type_file (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_type_id INTEGER NOT NULL,
    file_type_id INTEGER NOT NULL,
    filename TEXT NOT NULL,
    destination_dir TEXT NOT NULL,
    description TEXT NOT NULL,
    default_contents BINARY NOT NULL,

    UNIQUE (service_type_id, filename),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id)
);

CREATE TABLE service_type_param (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_type_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    key TEXT NOT NULL,
    description TEXT NOT NULL,
    default_value TEXT NULL,
    is_required BOOLEAN NOT NULL DEFAULT 0,
    value_type_id INTEGER NOT NULL,
    allowed_values TEXT NULL,

    UNIQUE (service_type_id, key),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id)
);

CREATE TABLE service_type_port (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_type_id INTEGER NOT NULL,
    network_protocol_id INTEGER NOT NULL,
    port INTEGER NOT NULL,
    remark TEXT NULL,

    UNIQUE (service_type_id, port),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id),
    FOREIGN KEY (network_protocol_id) REFERENCES network_protocol (id),
    CONSTRAINT ck_service_type_port_port CHECK (port > 0 AND port <= 65535)
);

INSERT INTO service_type_port (service_type_id, network_protocol_id, port, remark)
    VALUES
        -- Bitcoin Miner
        (0, 1, 18443, 'Bitcoin RPC'),
        (0, 1, 18444, 'Bitcoin P2P'),
        -- Bitcoin Follower
        (1, 1, 18443, 'Bitcoin RPC'),
        (1, 1, 18444, 'Bitcoin P2P'),
        -- Stacks Miner
        (2, 1, 20443, 'Stacks RPC'),
        (2, 1, 20444, 'Stacks P2P'),
        -- Stacks Follower
        (3, 1, 20443, 'Stacks RPC'),
        (3, 1, 20444, 'Stacks P2P')
    ;

CREATE TABLE service_version (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_type_id INTEGER NOT NULL,
    -- The version of the service. This is the version of the service's configuration,
    -- i.e. the display name shown in the CLI.
    version TEXT NOT NULL,
    minimum_epoch_id INTEGER NULL,
    maximum_epoch_id INTEGER NULL,
    -- The git target to use when building the service. Used for Stacks services.
    git_target TEXT NULL,
    -- The CLI name of the service. This is the name used in the CLI to refer to
    -- the service and doesn't contain spaces or special characters, except for '-'.
    cli_name TEXT NOT NULL,
    rebuild_required BOOLEAN NOT NULL DEFAULT 0,
    last_built_at TIMESTAMP NULL,
    -- The commit hash of the last build for when using the 'branch' git target.
    -- This helps us determine if we need to rebuild the service.
    last_build_commit_hash TEXT NULL,

    UNIQUE (service_type_id, version),
    UNIQUE (cli_name),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id)
);

INSERT INTO service_version (service_type_id, version, cli_name) 
    VALUES (0, '26.0', 'bitcoin-miner-26-0');      -- Bitcoin Miner
INSERT INTO service_version (service_type_id, version, cli_name) 
    VALUES (1, '26.0', 'bitcoin-follower-26-0');      -- Bitcoin Follower
INSERT INTO service_version (service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (2, '2.4.0.0.4', 6, 'tag:2.4.0.0.4', 'stacks-miner-2.4.0.0.4');       -- Stacks Miner
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (2, 'next', 'branch:next', 'stacks-miner-next');  -- Stacks Miner
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (2, 'develop', 'branch:develop', 'stacks-miner-develop');  -- Stacks Miner
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (2, '2.5.0.0.3', 'tag:2.5.0.0.3', 'stacks-miner-2.5.0.0.3');  -- Stacks Miner
INSERT INTO service_version (service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (3, '2.4.0.0.4', 6, 'tag:2.4.0.0.4', 'stacks-follower-2.4.0.0.4');       -- Stacks Follower
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (3, 'next', 'branch:next', 'stacks-follower-next');  -- Stacks Follower
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (3, 'develop', 'branch:develop', 'stacks-follower-develop');  -- Stacks Follower
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (3, '2.5.0.0.3', 'tag:2.5.0.0.3', 'stacks-follower-2.5.0.0.3');  -- Stacks Follower
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (4, 'next', 'branch:next', 'stacks-signer-next');  -- Stacks Signer
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (4, 'develop', 'branch:develop', 'stacks-signer-develop');  -- Stacks Signer
INSERT INTO service_version (service_type_id, version, git_target, cli_name) 
    VALUES (4, '2.5.0.0.3', 'tag:2.5.0.0.3', 'stacks-signer-2.5.0.0.3');  -- Stacks Signer
INSERT INTO service_version (service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (5, 'PoX-3', 6, 'tag:2.4.0.0.4', 'stacks-stacker-self-pox-3');     -- Stacks Stacker (Self)
INSERT INTO service_version (service_type_id, version, minimum_epoch_id, git_target, cli_name) 
    VALUES (5, 'PoX-4', 7, 'branch:next', 'stacks-stacker-self-pox-4');     -- Stacks Stacker (Self)
INSERT INTO service_version (service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (6, 'PoX-3', 6, 'tag:2.4.0.0.4', 'stacks-stacker-pool-pox-3');     -- Stacks Stacker (Pool)
INSERT INTO service_version (service_type_id, version, minimum_epoch_id, git_target, cli_name) 
    VALUES (6, 'PoX-4', 7, 'branch:next', 'stacks-stacker-pool-pox-4');     -- Stacks Stacker (Pool)

CREATE TABLE service_upgrade_path (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    service_type_id INTEGER NOT NULL,
    from_service_version_id INTEGER NOT NULL,
    to_service_version_id INTEGER NOT NULL,
    minimum_epoch_id INTEGER NOT NULL DEFAULT 0,
    maximum_epoch_id INTEGER NULL,

    UNIQUE (service_type_id, from_service_version_id, to_service_version_id),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id),
    FOREIGN KEY (from_service_version_id) REFERENCES service_version (id),
    FOREIGN KEY (to_service_version_id) REFERENCES service_version (id)
    FOREIGN KEY (minimum_epoch_id) REFERENCES epoch (id),
    FOREIGN KEY (maximum_epoch_id) REFERENCES epoch (id)
) WITHOUT ROWID;

INSERT INTO service_upgrade_path (id, name, service_type_id, from_service_version_id, to_service_version_id) 
    VALUES (0, 'Stacks Miner: 2.4 → Nakamoto', 2, 2, 3);
INSERT INTO service_upgrade_path (id, name, service_type_id, from_service_version_id, to_service_version_id)
    VALUES (1, 'Stacks Follower: 2.4 → Nakamoto', 3, 4, 5);

CREATE TABLE environment_service (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_id INTEGER NOT NULL,
    service_version_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    comment TEXT NULL,

    UNIQUE (name),
    FOREIGN KEY (environment_id) REFERENCES environment (id),
    FOREIGN KEY (service_version_id) REFERENCES service_version (id)
);

CREATE TABLE environment_service_port (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_service_id INTEGER NOT NULL,
    source_port INTEGER NOT NULL,
    publish_port INTEGER NOT NULL,
    network_protocol_id INTEGER NOT NULL,
    remark TEXT NULL,

    UNIQUE (environment_service_id, source_port),
    UNIQUE (publish_port),
    FOREIGN KEY (environment_service_id) REFERENCES environment_service (id),
    FOREIGN KEY (network_protocol_id) REFERENCES network_protocol (id),
    CONSTRAINT ck_environment_service_port_source_port CHECK (source_port > 0 AND source_port <= 65535),
    CONSTRAINT ck_environment_service_port_publish_port CHECK (publish_port > 0 AND publish_port <= 65535)
);

CREATE TABLE environment_service_file (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_id INTEGER NOT NULL,
    environment_service_id INTEGER NOT NULL,
    service_type_file_id INTEGER NOT NULL,
    contents BINARY NOT NULL,

    UNIQUE (environment_id, environment_service_id, service_type_file_id),
    FOREIGN KEY (environment_id) REFERENCES environment (id),
    FOREIGN KEY (environment_service_id) REFERENCES environment_service (id),
    FOREIGN KEY (service_type_file_id) REFERENCES service_type_file (id)
);

CREATE TABLE service_action_type (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    requires_running_service BOOLEAN NOT NULL DEFAULT false,
    requires_network BOOLEAN NOT NULL DEFAULT false
);

INSERT INTO service_action_type (id, name, requires_running_service, requires_network) 
    VALUES 
        (1, 'container start', 0, 0),
        (2, 'container stop', 0, 0),
        (3, 'upgrade service', 0, 0),
        (4, 'start service', 0, 0),
        (5, 'stop service', 1, 0),
        (6, 'start network', 0, 0),
        (7, 'stop network', 0, 1)
    ;

CREATE TABLE service_action_type_constraint (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_action_id INTEGER NOT NULL,
    allowed_after_service_action_id INTEGER NULL,

    UNIQUE (service_action_id, allowed_after_service_action_id),
    FOREIGN KEY (service_action_id) REFERENCES service_action_type (id),
    FOREIGN KEY (allowed_after_service_action_id) REFERENCES service_action_type (id)
);

INSERT INTO service_action_type_constraint (service_action_id, allowed_after_service_action_id) 
    VALUES 
        -- container stop, after: container start, start service, stop service, upgrade service
        (2, 1),
        (2, 3),
        (2, 4),
        (2, 5),
        -- upgrade service, after: container start, start service, stop service
        (3, 1),
        (3, 5),
        -- start service, after: container start, upgrade service, stop service
        (4, 1),
        (4, 3),
        (4, 5),
        -- stop service, after: start service
        (5, 4)
    ;

CREATE TABLE environment_service_action (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_service_id INTEGER NOT NULL,
    service_action_type_id INTEGER NOT NULL,
    at_block_height INTEGER NULL,
    at_epoch_id INTEGER NULL,
    data TEXT NULL,

    FOREIGN KEY (environment_service_id) REFERENCES environment_service (id),
    FOREIGN KEY (service_action_type_id) REFERENCES service_action_type (id)
);

CREATE TABLE environment_service_param (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_service_id INTEGER NOT NULL,
    service_type_param_id INTEGER NOT NULL,
    value TEXT NOT NULL,

    UNIQUE (environment_service_id, service_type_param_id),
    FOREIGN KEY (environment_service_id) REFERENCES environment_service (id),
    FOREIGN KEY (service_type_param_id) REFERENCES service_type_param (id)
);

CREATE TABLE environment_container (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_service_id INTEGER NOT NULL,
    container_id TEXT NOT NULL,
    service_version_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (environment_service_id, container_id),
    FOREIGN KEY (environment_service_id) REFERENCES environment_service (id),
    FOREIGN KEY (service_version_id) REFERENCES service_version (id)
);

CREATE TABLE environment_container_action_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_container_id INTEGER NOT NULL,
    service_action_type_id INTEGER NOT NULL,
    at_block_height INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    data TEXT NULL,

    FOREIGN KEY (environment_container_id) REFERENCES environment_container (id),
    FOREIGN KEY (service_action_type_id) REFERENCES service_action_type (id)
);

CREATE TABLE environment_keychain (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_id INTEGER NOT NULL,
    stx_address TEXT NOT NULL,
    amount INTEGER NOT NULL,
    mnemonic TEXT NOT NULL,
    private_key TEXT NOT NULL,
    public_key TEXT NOT NULL,
    btc_address TEXT NOT NULL,
    nonce INTEGER NOT NULL DEFAULT 0,
    remark TEXT NULL,

    UNIQUE (stx_address),
    UNIQUE (btc_address),
    UNIQUE (private_key),
    UNIQUE (mnemonic),

    FOREIGN KEY (environment_id) REFERENCES environment (id)
);