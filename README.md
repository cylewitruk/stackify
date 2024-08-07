# Stackify

```text
 ____  _             _    _  __    
/ ___|| |_ __ _  ___| | _(_)/ _|_   _ 
\___ \| __/ _` |/ __| |/ / | |_| | | |
 ___) | || (_| | (__|   <| |  _| |_| |
|____/ \__\__,_|\___|_|\_\_|_|  \__, |
                                |___/ 
```

Welcome to Stackify! A tool which aims to enable _consistent_ and _reproducible_ Stacks
environments, including all dependencies, in **regtest** (_and possibly additional environments later on_). No more fiddling with Docker compose and configuration files - easily compose
environments with an interactive interface, and import/export environments for exact
reproduction on other machines.

## Feature Highlights

✨ Interactive, intuitive and pretty CLI interface (shoutout to [cliclack](https://github.com/fadeevab/cliclack))  
✨ Create one or many isolated Stacks environments, each with their own configurations  
✨ Orchestrate in-place binary upgrades (i.e. nodes, signers, etc.) based on either epoch or block height  
✨ Orchestrate starting and stopping of services at specific epochs or block heights  
✨ Easily configure multiple different versions of the same service within the same environment  
✨ Portable - Stackify uses its own build and runtime containers, so configurations are guaranteed to work across machines  

## System Requirements

Stackify requires that you have [Docker](https://www.docker.com/) installed on your machine and enough resources to run the environments that you create. Due to the nature of freedom in how these environments may be constructed, minimum specs can't be provided, however in general at least a quad-core machine, 8GB RAM and 20GB of storage is recommended for even the smallest of environments.

In general, the Stackify images including all needed source code and build tools will consume just under `3GB` of space, and resulting artifacts another `500MB`. There must be space remaining for service containers and their deltas, logs, etc.

Stackify _does not require_ that Rust is installed locally - it will be provided via its Docker containers.

## Getting Started

### Installation

First, please ensure that you've read [System Requirements](#system-requirements). Download the binary for your OS/arch or clone this repo and build the project yourself.

To install Stackify, run `stackify install` and follow the prompts.

This will create an application directory at `~/.stackify`, copy built-in assets such as configuration files, templates, Dockerfiles, scripts, etc., download [Bitcoin Core](https://bitcoincore.org/), build required Docker images and setup/configure the Stackify application database.

**Note** that Stackify does require ~3GB of space.

### Create your first environment

Stackify is built around the concept of **environments**, which allow you to create completely isolated environment configurations with different ccompositions of services, versions and configurations.

To create a new environment, run the following command (replacing `FOO` with your desired name for the environment):

```text
stackify environment create FOO
```

Once your environment is created, you will need to add services to it for it to be usable.

#### Add Services

Each environment can have one or many services. Stackify services represent a specific version and configuration of Stacks network components. Stackify provides an interactive prompt for configuring new services in environments.

When adding a service, there are a number of configuration options you can use to fine-tune the service's behavior, such as:

- The version of the service. Service versions are part of the Stackify configuration and must be managed separately. See [Configure Service Versions].
- Start block height OR epoch, immediately or never (start manually)
- Stop block height OR epoch, or never (stop manually)

**Note:** It is completely valid and _**part of the use case**_ to mix and match different versions of services and configurations, such as start/stop block heights/epochs, etc.!

To add a new service, run the following command (replacing `FOO` with the name of your environment):

```stackify environment service add --env FOO```

Which will present you with an interactive prompt of available services. Depending on the service you select, different configuration options will be available to you. For example, when configuring a [Stacks Signer](https://docs.stacks.co/nakamoto-upgrade/signing-and-stacking/running-a-signer), you will be prompted to specify which Stacks Node to receive events from:

![Add Service](docs/assets/add_service.gif)

#### Build the Environment

Before an environment can be launched, it must first be built. This will use Stackify's own Docker build containers to produce the required binaries of the correct targets and versions to be run within the Stackify runtime containers. _Note that since the runtime containers are the targets, the built binaries may not necessarily be runnable directly by you on your system._

![Build Environment](docs/assets/build_env.gif)

Once the build has completed, you will find the built binaries in `~/.stackify/bin/` and it should look something like the following (depending on the services configured):

![Post-Build](docs/assets/after_env_build.png)

#### Start the Environment

To start the environment, which will create the necessary Docker resources and containers, generate and install related configuration files, and start the containers. Start the environment using the `environment start` command (replacing `FOO` with the name of your environment):

```text
stackify env start FOO
```

![Start Environment](docs/assets/start_env.gif)

Once the environment has started, you should be able to see your environment running via `docker ps`, for example:

```text
➜ docker ps
CONTAINER ID   IMAGE                     COMMAND                  CREATED         STATUS         PORTS     NAMES
2257523b79b1   stackify-runtime:latest   "/bin/sh -c '/entryp…"   7 minutes ago   Up 7 minutes             stx-foo-stacks-signer-9fcfcbba
fc66e442d606   stackify-runtime:latest   "/bin/sh -c '/entryp…"   7 minutes ago   Up 7 minutes             stx-foo-stacks-signer-5c27527f
9233eaaa355d   stackify-runtime:latest   "/bin/sh -c '/entryp…"   7 minutes ago   Up 7 minutes             stx-foo-stacks-miner-acde5630
2d0ee8245cb8   stackify-runtime:latest   "/bin/sh /entrypoint…"   7 minutes ago   Up 7 minutes             stx-foo-bitcoin-miner-9e20268c
a71f95f0add9   busybox:latest            "/bin/sh -c 'while :…"   7 minutes ago   Up 7 minutes             stx-foo
```

#### Viewing Environments

You can view environments and their general configuration using the `stackify env ls` command:

![List Environments](docs/assets/list_env.png)

To view detailed information for specific services within an environment, use the `stackify env svc inspect` command.
