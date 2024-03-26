# Command-Line Help for `stackify-cli`

This document contains the help content for the `stackify-cli` command-line program.

**Command Overview:**

* [`stackify-cli`↴](#stackify-cli)
* [`stackify-cli initialize`↴](#stackify-cli-initialize)
* [`stackify-cli environment`↴](#stackify-cli-environment)
* [`stackify-cli environment list`↴](#stackify-cli-environment-list)
* [`stackify-cli environment create`↴](#stackify-cli-environment-create)
* [`stackify-cli environment build`↴](#stackify-cli-environment-build)
* [`stackify-cli environment inspect`↴](#stackify-cli-environment-inspect)
* [`stackify-cli environment delete`↴](#stackify-cli-environment-delete)
* [`stackify-cli environment start`↴](#stackify-cli-environment-start)
* [`stackify-cli environment stop`↴](#stackify-cli-environment-stop)
* [`stackify-cli environment down`↴](#stackify-cli-environment-down)
* [`stackify-cli environment service`↴](#stackify-cli-environment-service)
* [`stackify-cli environment service add`↴](#stackify-cli-environment-service-add)
* [`stackify-cli environment service remove`↴](#stackify-cli-environment-service-remove)
* [`stackify-cli environment service inspect`↴](#stackify-cli-environment-service-inspect)
* [`stackify-cli environment service list`↴](#stackify-cli-environment-service-list)
* [`stackify-cli environment service config`↴](#stackify-cli-environment-service-config)
* [`stackify-cli environment epoch`↴](#stackify-cli-environment-epoch)
* [`stackify-cli environment epoch list`↴](#stackify-cli-environment-epoch-list)
* [`stackify-cli environment epoch edit`↴](#stackify-cli-environment-epoch-edit)
* [`stackify-cli info`↴](#stackify-cli-info)
* [`stackify-cli clean`↴](#stackify-cli-clean)
* [`stackify-cli config`↴](#stackify-cli-config)
* [`stackify-cli config reset`↴](#stackify-cli-config-reset)
* [`stackify-cli config import`↴](#stackify-cli-config-import)
* [`stackify-cli config export`↴](#stackify-cli-config-export)
* [`stackify-cli config services`↴](#stackify-cli-config-services)
* [`stackify-cli config services add-version`↴](#stackify-cli-config-services-add-version)
* [`stackify-cli config services remove-version`↴](#stackify-cli-config-services-remove-version)
* [`stackify-cli config services list`↴](#stackify-cli-config-services-list)
* [`stackify-cli config services inspect`↴](#stackify-cli-config-services-inspect)
* [`stackify-cli config epochs`↴](#stackify-cli-config-epochs)
* [`stackify-cli config epochs list`↴](#stackify-cli-config-epochs-list)
* [`stackify-cli config epochs add`↴](#stackify-cli-config-epochs-add)
* [`stackify-cli config epochs remove`↴](#stackify-cli-config-epochs-remove)
* [`stackify-cli config epochs inspect`↴](#stackify-cli-config-epochs-inspect)
* [`stackify-cli completions`↴](#stackify-cli-completions)

## `stackify-cli`

  ____  _             _    _  __       
/ ___|| |_ __ _  ___| | _(_)/ _|_   _ 
\___ \| __/ _` |/ __| |/ / | |_| | | |
 ___) | || (_| | (__|   <| |  _| |_| |
|____/ \__\__,_|\___|_|\_\_|_|  \__, |
                                |___/ 

**Usage:** `stackify-cli [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `initialize` — Initializes the local environment in preparation for building & running Stacks environments. This will download several binaries and build several Docker images and will take some time
* `environment` — Commands for configuring, manipulating and interacting with environments
* `info` — Displays information about current environments and optionally other details
* `clean` — Cleans up resources created/used by stackify
* `config` — Commands for interacting with the stackify global configuration
* `completions` — 

###### **Options:**

* `--completion <COMPLETION>` — Generate completion scripts for the specified shell

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity
* `--color <WHEN>` — Controls when to use color

  Default value: `auto`

  Possible values: `auto`, `always`, `never`




## `stackify-cli initialize`

Initializes the local environment in preparation for building & running Stacks environments. This will download several binaries and build several Docker images and will take some time

**Usage:** `stackify-cli initialize [OPTIONS]`

###### **Options:**

* `--bitcoin-version <BITCOIN_VERSION>` — Specify the Bitcoin Core version to download

  Default value: `26.0`
* `--dasel-version <DASEL_VERSION>` — Specify the Dasel version to download

  Default value: `2.7.0`
* `--pre-compile` — Specifies whether or not Cargo projects should be initalized (pre-compiled) in the build image. This ensures that all dependencies are already compiled, but results in a much larger image (c.a. 9GB vs 2.5GB). The trade-off is between size vs. build speed. If you plan on building new runtime binaries often, this may be a good option

  Default value: `false`

  Possible values: `true`, `false`

* `--download-only` — Only download runtime binaries, do not build the images

  Default value: `false`

  Possible values: `true`, `false`

* `--build-only` — Only build the images, do not download runtime binaries

  Default value: `false`

  Possible values: `true`, `false`




## `stackify-cli environment`

Commands for configuring, manipulating and interacting with environments

**Usage:** `stackify-cli environment <COMMAND>`

###### **Subcommands:**

* `list` — Displays a list of created environments
* `create` — Create a new environment
* `build` — Builds the specified environment, compiling the necessary binaries for the services if needed and creating the Docker containers which will be used for runtime. The environment will not be started, however
* `inspect` — Displays detailed information about the specified environment
* `delete` — Removes the specified environment and all associated resources. This action is irreversible
* `start` — Starts the specified environment using its current configuration. If the environment has not yet been built, it will be built first, which may take some time. If the environment is already running, this command will have no effect
* `stop` — Stops the specified environment
* `down` — Stops the specified environment if it is running and removes all associated resources, without actually deleting the environment
* `service` — Commands for managing environments' services
* `epoch` — 



## `stackify-cli environment list`

Displays a list of created environments

**Usage:** `stackify-cli environment list`



## `stackify-cli environment create`

Create a new environment

**Usage:** `stackify-cli environment create [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the environment to create

###### **Options:**

* `-b`, `--bitcoin-block-speed <SECONDS>` — The speed at which blocks are mined in the Bitcoin network

  Default value: `30`



## `stackify-cli environment build`

Builds the specified environment, compiling the necessary binaries for the services if needed and creating the Docker containers which will be used for runtime. The environment will not be started, however

**Usage:** `stackify-cli environment build <NAME>`

###### **Arguments:**

* `<NAME>`



## `stackify-cli environment inspect`

Displays detailed information about the specified environment

**Usage:** `stackify-cli environment inspect <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the environment to inspect



## `stackify-cli environment delete`

Removes the specified environment and all associated resources. This action is irreversible

**Usage:** `stackify-cli environment delete <NAME>`

###### **Arguments:**

* `<NAME>`



## `stackify-cli environment start`

Starts the specified environment using its current configuration. If the environment has not yet been built, it will be built first, which may take some time. If the environment is already running, this command will have no effect

**Usage:** `stackify-cli environment start <NAME>`

###### **Arguments:**

* `<NAME>`



## `stackify-cli environment stop`

Stops the specified environment

**Usage:** `stackify-cli environment stop <NAME>`

###### **Arguments:**

* `<NAME>`



## `stackify-cli environment down`

Stops the specified environment if it is running and removes all associated resources, without actually deleting the environment

**Usage:** `stackify-cli environment down <NAME>`

###### **Arguments:**

* `<NAME>`



## `stackify-cli environment service`

Commands for managing environments' services

**Usage:** `stackify-cli environment service <COMMAND>`

###### **Subcommands:**

* `add` — Adds a new service to the specified environment
* `remove` — Remove a service from the specified environment
* `inspect` — Displays detailed information about the specified service
* `list` — Displays a list of services for the specified environment
* `config` — Commands for managing service configuration files. This will provide you with an editor to manually edit the configuration file for the specified service



## `stackify-cli environment service add`

Adds a new service to the specified environment

**Usage:** `stackify-cli environment service add [OPTIONS] --environment <NAME>`

###### **Options:**

* `-i` — Indicates whether or not an interactive prompt should be used for providing the required information for this command (recommended!). This flag is set by default

  Default value: `true`

  Possible values: `true`, `false`

* `-e`, `--environment <NAME>` — The name of the environment to which the service should be added



## `stackify-cli environment service remove`

Remove a service from the specified environment

**Usage:** `stackify-cli environment service remove [OPTIONS]`

###### **Options:**

* `-e`, `--environment <NAME>` — The name of the environment from which the service should be removed. You can omit this argument if the service is unique across all environments, otherwise you will receive an error



## `stackify-cli environment service inspect`

Displays detailed information about the specified service

**Usage:** `stackify-cli environment service inspect [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the service of which to inspect

###### **Options:**

* `-e`, `--environment <NAME>` — The name of the environment to which the service belongs. You can omit this argument if the service is unique across all environments, otherwise you will receive an error



## `stackify-cli environment service list`

Displays a list of services for the specified environment

**Usage:** `stackify-cli environment service list <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the environment to list services for



## `stackify-cli environment service config`

Commands for managing service configuration files. This will provide you with an editor to manually edit the configuration file for the specified service

**Usage:** `stackify-cli environment service config`



## `stackify-cli environment epoch`

**Usage:** `stackify-cli environment epoch [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `list` — Prints the current epoch-map for the specified environment
* `edit` — Edit the epoch-map for the specified environment

###### **Options:**

* `-e`, `--environment <NAME>` — The name of the environment to which the epoch-map belongs



## `stackify-cli environment epoch list`

Prints the current epoch-map for the specified environment

**Usage:** `stackify-cli environment epoch list`



## `stackify-cli environment epoch edit`

Edit the epoch-map for the specified environment

**Usage:** `stackify-cli environment epoch edit`



## `stackify-cli info`

Displays information about current environments and optionally other details

**Usage:** `stackify-cli info [OPTIONS]`

###### **Options:**

* `-d`, `--docker`

  Default value: `false`

  Possible values: `true`, `false`

* `-e`, `--epochs`

  Default value: `false`

  Possible values: `true`, `false`

* `-s`, `--services`

  Default value: `false`

  Possible values: `true`, `false`




## `stackify-cli clean`

Cleans up resources created/used by stackify

**Usage:** `stackify-cli clean`



## `stackify-cli config`

Commands for interacting with the stackify global configuration

**Usage:** `stackify-cli config <COMMAND>`

###### **Subcommands:**

* `reset` — This will completely remove all Stackify resources from the system
* `import` — Import a Stackify export file
* `export` — Export Stackify configuration, which can be imported later. This is useful for sharing configurations between different machines
* `services` — Commands for working with the services (i.e. Bitcoin nodes, Stacks nodes, etc.) and their configurations
* `epochs` — 



## `stackify-cli config reset`

This will completely remove all Stackify resources from the system

**Usage:** `stackify-cli config reset`



## `stackify-cli config import`

Import a Stackify export file

**Usage:** `stackify-cli config import --file <FILE>`

###### **Options:**

* `-f`, `--file <FILE>`



## `stackify-cli config export`

Export Stackify configuration, which can be imported later. This is useful for sharing configurations between different machines

**Usage:** `stackify-cli config export [OPTIONS] --environment <ENVIRONMENT>`

###### **Options:**

* `-e`, `--environment <ENVIRONMENT>`
* `--environments`

  Possible values: `true`, `false`

* `-s`, `--services`

  Possible values: `true`, `false`




## `stackify-cli config services`

Commands for working with the services (i.e. Bitcoin nodes, Stacks nodes, etc.) and their configurations

**Usage:** `stackify-cli config services <COMMAND>`

###### **Subcommands:**

* `add-version` — Add a new version to one of the available service types
* `remove-version` — Remove a service version from the available service versions. This command will fail if the calculated service + version name is already in use
* `list` — List all available services and their versions, plus their detailed information
* `inspect` — Display detailed information about a service and its versions



## `stackify-cli config services add-version`

Add a new version to one of the available service types

**Usage:** `stackify-cli config services add-version [OPTIONS] <VERSION>`

###### **Arguments:**

* `<VERSION>` — The version of the service to add, for example: `21.0`, 'PoX-5', etc. Note that different services types have different constraints regarding what can be used as a version

###### **Options:**

* `--min-epoch <EPOCH>` — The minimum epoch that this service version is compatible with. This must be a valid epoch name, for example: `2.05`, `2.4`, `3.0`, etc. Note that the service will not be prevented from being used on a lower epoch as that may be your intent, but it will generate a warning
* `--max-epoch <EPOCH>` — The maximum epoch that this service version is compatible with. This must be a valid epoch name, for example: `2.05`, `2.4`, `3.0`, etc. Note that the service will not be prevented from being used on a higher epoch as that may be your intent, but it will generate a warning. To view the available epochs, run `stackify config epochs list`
* `--git-target <BRANCH|TAG|COMMIT>` — The git target for this service version. This can be a branch, tag, or commit hash. This is conditionally required/allowed based on the service type.

The prefix defines the type of git target: 'branch:', 'tag:', 'commit:'. For example, 'branch:main', 'tag:v1.0.0', 'commit:abcdef1234567890'.

Required for: [stacks-miner, stacks-follower, stacks-signer]
Not allowed for: [bitcoin-miner, bitcoin-follower, stacks-stacker-self, stacks-stacker-pool]



## `stackify-cli config services remove-version`

Remove a service version from the available service versions. This command will fail if the calculated service + version name is already in use

**Usage:** `stackify-cli config services remove-version <SERVICE>`

###### **Arguments:**

* `<SERVICE>`



## `stackify-cli config services list`

List all available services and their versions, plus their detailed information

**Usage:** `stackify-cli config services list`



## `stackify-cli config services inspect`

Display detailed information about a service and its versions

**Usage:** `stackify-cli config services inspect`



## `stackify-cli config epochs`

**Usage:** `stackify-cli config epochs <COMMAND>`

###### **Subcommands:**

* `list` — Prints a list of all available epochs
* `add` — Add a new epoch to the list of available epochs. This is considered an expert feature and should only be used if you know what you are doing
* `remove` — Remove an epoch from the available epochs. Note that an epoch cannot be removed if it is in use by a service version. To see the usages of an epoch, run `stackify config epochs inspect`
* `inspect` — Display detailed information about an epoch, including its usages



## `stackify-cli config epochs list`

Prints a list of all available epochs

**Usage:** `stackify-cli config epochs list`



## `stackify-cli config epochs add`

Add a new epoch to the list of available epochs. This is considered an expert feature and should only be used if you know what you are doing

**Usage:** `stackify-cli config epochs add [OPTIONS] --force <EPOCH>`

###### **Arguments:**

* `<EPOCH>` — The name of the epoch to add. This must be a valid epoch name, for example: `2.05`, `2.4`, `3.0`, etc. The epoch must be unique, in decimal format, and greater than the current highest epoch

###### **Options:**

* `--block-height <BLOCK_HEIGHT>` — Optionally specifies the default block height for this epoch. If not provided, the default block height will be set to the current highest default block height + 3
* `--force` — As adding a new epoch is considered an "expert feature", this flag is required to be set to confirm that the user understands the implications of adding a new epoch

  Possible values: `true`, `false`




## `stackify-cli config epochs remove`

Remove an epoch from the available epochs. Note that an epoch cannot be removed if it is in use by a service version. To see the usages of an epoch, run `stackify config epochs inspect`

**Usage:** `stackify-cli config epochs remove`



## `stackify-cli config epochs inspect`

Display detailed information about an epoch, including its usages

**Usage:** `stackify-cli config epochs inspect`



## `stackify-cli completions`

**Usage:** `stackify-cli completions <SHELL>`

###### **Arguments:**

* `<SHELL>` — The shell to generate the completions for

  Possible values: `bash`, `elvish`, `fig`, `fish`, `nushell`, `powershell`, `zsh`




<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>


