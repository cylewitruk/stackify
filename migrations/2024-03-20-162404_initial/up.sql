-- This is for an SQLite database

-- Enumeration table holding the different allowed Stacks epochs.
CREATE TABLE epoch (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

INSERT INTO epoch (id, name) VALUES (0, '1.0');
INSERT INTO epoch (id, name) VALUES (1, '2.0');
INSERT INTO epoch (id, name) VALUES (2, '2.05');
INSERT INTO epoch (id, name) VALUES (3, '2.1');
INSERT INTO epoch (id, name) VALUES (4, '2.2');
INSERT INTO epoch (id, name) VALUES (5, '2.3');
INSERT INTO epoch (id, name) VALUES (6, '2.4');
INSERT INTO epoch (id, name) VALUES (7, '2.5');
INSERT INTO epoch (id, name) VALUES (8, '3.0');

CREATE TABLE environment_status (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
) WITHOUT ROWID;

INSERT INTO environment_status (id, name) VALUES (0, 'stopped');
INSERT INTO environment_status (id, name) VALUES (1, 'running');
INSERT INTO environment_status (id, name) VALUES (2, 'error');

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
    allow_minimum_epoch BOOLEAN NOT NULL DEFAULT 1,
    allow_maximum_epoch BOOLEAN NOT NULL DEFAULT 1,
    allow_git_target BOOLEAN NOT NULL DEFAULT 1
) WITHOUT ROWID;

INSERT INTO service_type (id, name, allow_git_target) VALUES (0, 'Bitcoin Miner', 0);
INSERT INTO service_type (id, name, allow_git_target) VALUES (1, 'Bitcoin Follower', 0);
INSERT INTO service_type (id, name) VALUES (2, 'Stacks Miner');
INSERT INTO service_type (id, name) VALUES (3, 'Stacks Follower');
INSERT INTO service_type (id, name) VALUES (4, 'Stacks Signer'); -- Minimum epoch 2.5
INSERT INTO service_type (id, name, allow_git_target) VALUES (5, 'Stacks Stacker (Self)', 0);
INSERT INTO service_type (id, name, allow_git_target) VALUES (6, 'Stacks Stacker (Pool)', 0);

CREATE TABLE service_version (
    id INTEGER PRIMARY KEY,
    service_type_id INTEGER NOT NULL,
    version TEXT NOT NULL,
    minimum_epoch_id INTEGER NULL,
    maximum_epoch_id INTEGER NULL,
    git_target TEXT NULL,

    UNIQUE (service_type_id, version),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id)
) WITHOUT ROWID;

INSERT INTO service_version (id, service_type_id, version) VALUES (0, 0, '26.0');      -- Bitcoin Miner
INSERT INTO service_version (id, service_type_id, version) VALUES (1, 1, '26.0');      -- Bitcoin Follower
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target) VALUES (2, 2, '2.4', 6, 'tag:2.4.0.0.4');       -- Stacks Miner
INSERT INTO service_version (id, service_type_id, version, git_target) VALUES (3, 2, 'nakamoto', 'branch:next');  -- Stacks Miner
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target) VALUES (4, 3, '2.4', 6, 'tag:2.4.0.0.4');       -- Stacks Follower
INSERT INTO service_version (id, service_type_id, version, git_target) VALUES (5, 3, 'nakamoto', 'branch:next');  -- Stacks Follower
INSERT INTO service_version (id, service_type_id, version, git_target) VALUES (6, 4, 'nakamoto', 'branch:next');  -- Stacks Signer
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target) VALUES (7, 5, 'PoX-3', 6, 'tag:2.4.0.0.4');     -- Stacks Stacker (Self)
INSERT INTO service_version (id, service_type_id, version, minimum_epoch_id, git_target) VALUES (8, 5, 'PoX-4', 7, 'branch:next');     -- Stacks Stacker (Self)
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target) VALUES (9, 6, 'PoX-3', 6, 'tag:2.4.0.0.4');     -- Stacks Stacker (Pool)
INSERT INTO service_version (id, service_type_id, version, minimum_epoch_id, git_target) VALUES (10, 6, 'PoX-4', 7, 'branch:next');     -- Stacks Stacker (Pool)

CREATE TABLE service_upgrade_path (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    service_type_id INTEGER NOT NULL,
    from_service_version_id INTEGER NOT NULL,
    to_service_version_id INTEGER NOT NULL,
    minimum_epoch_id INTEGER NOT NULL DEFAULT 0,
    maximum_epoch_id INTEGER NULL
) WITHOUT ROWID;

INSERT INTO service_upgrade_path (id, name, service_type_id, from_service_version_id, to_service_version_id) 
    VALUES (0, 'Stacks Miner: 2.4 → Nakamoto', 2, 2, 3);
INSERT INTO service_upgrade_path (id, name, service_type_id, from_service_version_id, to_service_version_id)
    VALUES (1, 'Stacks Follower: 2.4 → Nakamoto', 3, 4, 5);

CREATE TABLE service (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    environment_id INTEGER NOT NULL,
    service_type_id INTEGER NOT NULL,
    start_at_block_height INTEGER NOT NULL DEFAULT 0,
    stop_at_block_height INTEGER NULL,

    UNIQUE (environment_id, name),
    FOREIGN KEY (environment_id) REFERENCES environment (id),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id)
);

CREATE TABLE service_upgrade (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL,
    service_upgrade_path_id INTEGER NOT NULL,
    at_block_height INTEGER NOT NULL,

    UNIQUE (service_id, service_upgrade_path_id),
    FOREIGN KEY (service_id) REFERENCES service (id),
    FOREIGN KEY (service_upgrade_path_id) REFERENCES service_upgrade_path (id)
);