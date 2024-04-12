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
INSERT INTO service_type (id, name, cli_name, allow_git_target)
    VALUES (5, 'Stacks Stacker (Self)', 'stacks-stacker-self', 0);
INSERT INTO service_type (id, name, cli_name, allow_git_target) 
    VALUES (6, 'Stacks Stacker (Pool)', 'stacks-stacker-pool', 0);

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
        (4, 'Keychain')
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

CREATE TABLE service_version (
    id INTEGER PRIMARY KEY,
    service_type_id INTEGER NOT NULL,
    version TEXT NOT NULL,
    minimum_epoch_id INTEGER NULL,
    maximum_epoch_id INTEGER NULL,
    git_target TEXT NULL,
    cli_name TEXT NOT NULL,
    rebuild_required BOOLEAN NOT NULL DEFAULT 0,

    UNIQUE (service_type_id, version),
    UNIQUE (cli_name),
    FOREIGN KEY (service_type_id) REFERENCES service_type (id)
) WITHOUT ROWID;

INSERT INTO service_version (id, service_type_id, version, cli_name) 
    VALUES (0, 0, '26.0', 'bitcoin-miner-26-0');      -- Bitcoin Miner
INSERT INTO service_version (id, service_type_id, version, cli_name) 
    VALUES (1, 1, '26.0', 'bitcoin-follower-26-0');      -- Bitcoin Follower
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (2, 2, '2.4', 6, 'tag:2.4.0.0.4', 'stacks-miner-2-4');       -- Stacks Miner
INSERT INTO service_version (id, service_type_id, version, git_target, cli_name) 
    VALUES (3, 2, 'nakamoto', 'branch:next', 'stacks-miner-nakamoto');  -- Stacks Miner
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (4, 3, '2.4', 6, 'tag:2.4.0.0.4', 'stacks-follower-2-4');       -- Stacks Follower
INSERT INTO service_version (id, service_type_id, version, git_target, cli_name) 
    VALUES (5, 3, 'nakamoto', 'branch:next', 'stacks-follower-nakamoto');  -- Stacks Follower
INSERT INTO service_version (id, service_type_id, version, git_target, cli_name) 
    VALUES (6, 4, 'nakamoto', 'branch:next', 'stacks-signer-nakamoto');  -- Stacks Signer
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (7, 5, 'PoX-3', 6, 'tag:2.4.0.0.4', 'stacks-stacker-self-pox-3');     -- Stacks Stacker (Self)
INSERT INTO service_version (id, service_type_id, version, minimum_epoch_id, git_target, cli_name) 
    VALUES (8, 5, 'PoX-4', 7, 'branch:next', 'stacks-stacker-self-pox-4');     -- Stacks Stacker (Self)
INSERT INTO service_version (id, service_type_id, version, maximum_epoch_id, git_target, cli_name) 
    VALUES (9, 6, 'PoX-3', 6, 'tag:2.4.0.0.4', 'stacks-stacker-pool-pox-3');     -- Stacks Stacker (Pool)
INSERT INTO service_version (id, service_type_id, version, minimum_epoch_id, git_target, cli_name) 
    VALUES (10, 6, 'PoX-4', 7, 'branch:next', 'stacks-stacker-pool-pox-4');     -- Stacks Stacker (Pool)

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

CREATE TABLE stacks_account (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL,
    amount INTEGER NOT NULL,
    mnemonic TEXT NOT NULL,
    private_key TEXT NOT NULL,
    btc_address TEXT NOT NULL,

    UNIQUE (address),
    UNIQUE (btc_address),
    UNIQUE (private_key),
    UNIQUE (mnemonic)
);

INSERT INTO stacks_account (address, amount, mnemonic, private_key, btc_address) 
    VALUES 
        (
            'ST3EQ88S02BXXD0T5ZVT3KW947CRMQ1C6DMQY8H19', 
            100000000000000, 
            'point approve language letter cargo rough similar wrap focus edge polar task olympic tobacco cinnamon drop lawn boring sort trade senior screen tiger climb',
            '539e35c740079b79f931036651ad01f76d8fe1496dbd840ba9e62c7e7b355db001',
            'n1htkoYKuLXzPbkn9avC2DJxt7X85qVNCK'
        ),
        (
            'ST3KCNDSWZSFZCC6BE4VA9AXWXC9KEB16FBTRK36T',
            100000000000000,
            'laugh capital express view pull vehicle cluster embark service clerk roast glance lumber glove purity project layer lyrics limb junior reduce apple method pear',
            '075754fb099a55e351fe87c68a73951836343865cd52c78ae4c0f6f48e234f3601',
            'n2ZGZ7Zau2Ca8CLHGh11YRnLw93b4ufsDR'
        ),
        (
            'STB2BWB0K5XZGS3FXVTG3TKS46CQVV66NAK3YVN8',
            100000000000000,
            'level garlic bean design maximum inhale daring alert case worry gift frequent floor utility crowd twenty burger place time fashion slow produce column prepare',
            '374b6734eaff979818c5f1367331c685459b03b1a2053310906d1408dc928a0001',
            'mhY4cbHAFoXNYvXdt82yobvVuvR6PHeghf'
        ),
        (
            'STSTW15D618BSZQB85R058DS46THH86YQQY6XCB7',
            100000000000000,
            'drop guess similar uphold alarm remove fossil riot leaf badge lobster ability mesh parent lawn today student olympic model assault syrup end scorpion lab',
            '26f235698d02803955b7418842affbee600fc308936a7ca48bf5778d1ceef9df01',
            'mkEDDqbELrKYGUmUbTAyQnmBAEz4V1MAro'
        );

CREATE TABLE environment_stacks_account (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_id INTEGER NOT NULL,
    stacks_account_id INTEGER NOT NULL,
    remark TEXT NULL,

    UNIQUE (environment_id, stacks_account_id),
    FOREIGN KEY (environment_id) REFERENCES environment (id),
    FOREIGN KEY (stacks_account_id) REFERENCES stacks_account (id)
);