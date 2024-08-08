use crate::{
    cli::{log::clilog, theme::ThemedObject},
    db::diesel::model::{Epoch, ServiceType, ServiceUpgradePath, ServiceVersion},
};
use color_eyre::eyre::{bail, eyre, Error, Result};
use console::style;
use diesel::IntoSql;
use octocrate::{repos, APIConfig as GithubApiConfig, GitHubAPI as GithubApi};
use stackify_common::{
    types::{GitTarget, GitTargetKind},
    util::to_alphanumeric_snake,
};

use crate::{
    cli::context::CliContext,
    util::{FilterByServiceType, FilterByServiceVersion, FindById},
};

use super::args::{
    AddServiceVersionArgs, RemoveServiceVersionArgs, ServiceSubCommands, ServicesArgs,
};

pub async fn exec_services(ctx: &CliContext, args: ServicesArgs) -> Result<()> {
    match args.subcommands {
        ServiceSubCommands::List => exec_list_services(ctx),
        ServiceSubCommands::Add => exec_add_service_version(ctx).await,
        ServiceSubCommands::RemoveVersion(inner_args) => {
            exec_remove_service_version(ctx, inner_args)
        }
        ServiceSubCommands::Inspect => exec_inspect_service(ctx),
    }
}

pub async fn exec_add_service_version(ctx: &CliContext) -> Result<()> {
    use stackify_common::{
        ServiceType::BitcoinFollower, ServiceType::BitcoinMiner, ServiceType::StacksFollower,
        ServiceType::StacksMiner, ServiceType::StacksSigner,
    };

    cliclack::intro("Create new service configuration".bold())?;
    cliclack::log::remark(
        "This will add a new service configuration to your Stackify installation 
which you can use in your environments.",
    )?;

    let service_types = ctx.db.list_service_types()?;

    // Collect service type
    let service_type = cliclack::select("Which type of service would you like to add?")
        .items(
            &service_types
                .iter()
                .map(|st| (st.clone(), &st.name, ""))
                .collect::<Vec<_>>(),
        )
        .interact()
        .map(|st| stackify_common::ServiceType::from_i32(st.id))??;

    // For Stacks services, collect the git target, otherwise `None`.
    let git_target = if [StacksMiner, StacksFollower, StacksSigner].contains(&service_type) {
        Some(collect_stacks_git_target().await?)
    } else {
        None
    };

    // For Bitcoin services, collect the version, otherwise `None`.
    let version_str = if [BitcoinMiner, BitcoinFollower].contains(&service_type) {
        Some(collect_bitcoin_version().await?)
    } else {
        None
    };

    let existing_versions = ctx
        .db
        .list_service_versions()?
        .into_iter()
        .filter(|sv| sv.service_type_id == (service_type as i32))
        .collect::<Vec<_>>();

    let allow_upgrade_from = if !existing_versions.is_empty() {
        cliclack::multiselect("Select the version(s) to allow upgrades from:")
            .items(
                &existing_versions
                    .iter()
                    .map(|sv| (sv.clone(), &sv.version, ""))
                    .collect::<Vec<_>>(),
            )
            .required(false)
            .interact()?
    } else {
        vec![]
    };

    let allow_upgrade_to = if !existing_versions.is_empty() {
        cliclack::multiselect("Select the version(s) to allow upgrades to:")
            .items(
                &existing_versions
                    .iter()
                    .map(|sv| (sv.clone(), &sv.version, ""))
                    .collect::<Vec<_>>(),
            )
            .required(false)
            .interact()?
    } else {
        vec![]
    };

    let comment: String = cliclack::input("Comment:")
        .placeholder("Write a short comment about this configuration (optional):")
        .required(false)
        .interact()?;

    let comment = if comment.is_empty() {
        None
    } else {
        Some(comment)
    };

    todo!();
    // let service_types = ctx.db.list_service_types()?;

    // // Collect service type
    // let service_type_names = service_types
    //     .iter()
    //     .map(|st| st.name.clone())
    //     .collect::<Vec<_>>();
    // let service_type = Select::new("Select a service type", service_type_names).prompt()?;
    // let service_type = service_types
    //     .iter()
    //     .find(|st| st.name == service_type)
    //     .ok_or(eyre!("Service type not found"))?;

    // // Collect service version
    // let all_service_versions = ctx.db.list_service_versions()?;
    // let service_versions = all_service_versions.filter_by_service_type(service_type.id);
    // let service_version_names = service_versions
    //     .iter()
    //     .map(|sv| sv.version.clone())
    //     .collect::<Vec<_>>();
    // let service_version =
    //     Select::new("Select a service version", service_version_names).prompt()?;
    // let service_version_id = service_versions
    //     .iter()
    //     .find(|sv| sv.version == service_version)
    //     .ok_or(eyre!("Service version not found"))?;

    // if service_type.allow_git_target {
    //     let git_target_type = Select::new(
    //         "What kind of git target do you want to use?",
    //         vec!["Tag", "Branch", "Commit hash"],
    //     )
    //     .prompt()?;
    //     let git_target = match git_target_type {
    //         "Tag" => {
    //             Text::new("Enter the git target")
    //                 .with_help_message("This is the tag of the 'stacks-core' Github repository from which the service should be built.")
    //                 .prompt()?
    //         }
    //         "Branch" => {
    //             Text::new("Enter the git target")
    //                 .with_help_message("This is the branch of the 'stacks-core' Github repository from which the service should be built.")
    //                 .prompt()?
    //         }
    //         "Commit hash" => {
    //             Text::new("Enter the git target")
    //                 .with_help_message("This is the commit hash of the 'stacks-core' Github repository from which the service should be built.")
    //                 .prompt()?
    //         }
    //         _ => bail!("Invalid git target type"),
    //     };
    // }

    // todo!()
}

pub fn exec_remove_service_version(
    _ctx: &CliContext,
    args: RemoveServiceVersionArgs,
) -> Result<()> {
    let _cli_name = to_alphanumeric_snake(&args.svc_name);

    todo!()
}

pub fn exec_inspect_service(_ctx: &CliContext) -> Result<()> {
    todo!()
}

pub fn exec_list_services(ctx: &CliContext) -> Result<()> {
    // Fetch service data from the database.
    let epochs = ctx.db.list_epochs()?;
    let service_types = ctx.db.list_service_types()?;
    let service_versions = ctx.db.list_service_versions()?;
    let service_upgrade_paths = ctx.db.list_service_upgrade_paths()?;

    // Header
    println!("{}", style("Supported Services:").bold().white());

    // Iterate over available service types and print their details.
    for service_type in service_types.iter() {
        // Print the service type name header.
        println!("‣ {}", style(&service_type.name).magenta());

        println!(
            "  {} {} {}",
            style("ƒ").white(),
            to_alphanumeric_snake(&service_type.name),
            style("(cli name)").dim()
        );

        // Get the available versions for this service type.
        let versions = service_versions.filter_by_service_type(service_type.id);

        // Iterate over the available versions and print their details.
        for i in 0..versions.len() {
            let version = versions[i];

            // Print the version header.
            println!(
                "  {} {} {}",
                style("◆").yellow(),
                style(&version.version).cyan(),
                style("(version)").dim()
            );

            println!(
                "  {} {} {} {}",
                style("┆").dim(),
                //style("␂").white(),
                style("ƒ").white(),
                &version.cli_name,
                style("(cli name)").dim()
            );

            // If there is a git target, print it.
            if let Some(target) = GitTarget::parse_opt(version.git_target.clone()) {
                print_git_target(&target);
            }

            // If there is a minimum epoch, print it.
            if let Some(epoch) = epochs.find_by_id_opt(version.minimum_epoch_id) {
                print_minimum_epoch(epoch);
            }

            // If there is a maximum epoch, print it.
            if let Some(epoch) = epochs.find_by_id_opt(version.maximum_epoch_id) {
                print_maximum_epoch(epoch);
            }

            // Print the available upgrade paths.
            print_upgrade_paths(
                &service_types,
                &service_versions,
                &service_upgrade_paths,
                version.id,
            )?;
        }
        //println!("");
    }
    Ok(())
}

fn print_minimum_epoch(epoch: &Epoch) {
    println!(
        "  {} {} {} {}",
        style("┆").dim(),
        style("▼").green(),
        style(&epoch.name),
        style("(minimum epoch)").dim()
    );
}

fn print_maximum_epoch(epoch: &Epoch) {
    println!(
        "  {} {} {} {}",
        style("┆").dim(),
        style("▲").red(),
        style(&epoch.name),
        style("(maximum epoch)").dim()
    );
}

fn print_git_target(target: &GitTarget) {
    let git_type = match target.target_type {
        GitTargetKind::Tag => format!("{}", style("(git tag)").dim()),
        GitTargetKind::Branch => format!("{}", style("(git branch)").dim()),
        GitTargetKind::Commit => format!("{}", style("(git commit)").dim()),
    };

    println!(
        "  {} {} {} {}",
        style("┆").dim(),
        style("☉").bright().blue(),
        target.target,
        style(git_type).dim()
    );
}

fn print_upgrade_paths(
    service_types: &[ServiceType],
    service_versions: &[ServiceVersion],
    service_upgrade_paths: &[ServiceUpgradePath],
    version_id: i32,
) -> Result<()> {
    // Print the available upgrade paths.
    for path in service_upgrade_paths.filter_by_service_version(version_id) {
        let to_service_version = service_versions
            .find_by_id(path.to_service_version_id)
            .ok_or(eyre!("Failed to find service version"))?;
        let to_service_type = service_types
            .find_by_id(to_service_version.service_type_id)
            .ok_or(eyre!("Failed to find service type"))?;
        println!(
            "  {} {} {}: {} {}",
            style("┆").dim(),
            style("⤑").green(),
            style(&to_service_type.name),
            style(&to_service_version.version).green(),
            style("(upgradable to)").dim()
        );
    }

    Ok(())
}

async fn collect_stacks_git_target() -> Result<GitTarget> {
    let gh_config = GithubApiConfig::shared(GithubApiConfig::default());
    let gh_api = GithubApi::new(&gh_config);

    let kind = cliclack::select("What kind of git target do you want to use?")
        .item(GitTargetKind::Tag, "Tag", "")
        .item(GitTargetKind::Branch, "Branch", "")
        .item(GitTargetKind::Commit, "Commit hash", "")
        .interact()?;

    let target = match kind {
        GitTargetKind::Tag => {
            let spinner = cliclack::spinner();
            spinner.start("Fetching tags from the 'stacks-core' repository...");
            let mut query = repos::list_tags::Query::builder()
                .page(1)
                .per_page(100)
                .build();
            let mut tags: Vec<String> = Vec::new();
            loop {
                let gh_tags = gh_api
                    .repos
                    .list_tags("stacks-network", "stacks-core")
                    .query(&query)
                    .paginated_send()
                    .await?;
                let tag_names = gh_tags
                    .data
                    .iter()
                    .map(|tag| tag.name.clone())
                    .collect::<Vec<_>>();
                tags.extend(tag_names);
                if gh_tags.pages.next.is_none() {
                    break;
                } else {
                    query.page = gh_tags.pages.next;
                }
            }
            spinner.clear();

            let mut tag: Option<String> = None;
            while tag.is_none() {
                let tags = tags.clone();
                let git_tag: String = cliclack::input("Enter the name of the git tag:")
                    .validate(move |input: &String| {
                        if tags.contains(input) {
                            Ok::<(), _>(())
                        } else {
                            Err("The tag you entered does not exist in the 'stacks-core' repository. Please try again.".to_string())
                        }
                    })
                    .interact()?;

                tag = Some(git_tag);
            }

            GitTarget::new(GitTargetKind::Tag, &tag.unwrap())
        }
        GitTargetKind::Branch => {
            let spinner = cliclack::spinner();
            spinner.start("Fetching branches from the 'stacks-core' repository...");
            let mut query = repos::list_branches::Query::builder()
                .page(1)
                .per_page(100)
                .build();
            let mut branches: Vec<String> = Vec::new();
            loop {
                let gh_branches = gh_api
                    .repos
                    .list_branches("stacks-network", "stacks-core")
                    .query(&query)
                    .paginated_send()
                    .await?;
                let branch_names = gh_branches
                    .data
                    .iter()
                    .map(|tag| tag.name.clone())
                    .collect::<Vec<_>>();
                branches.extend(branch_names);
                if gh_branches.pages.next.is_none() {
                    break;
                } else {
                    query.page = gh_branches.pages.next;
                }
            }
            spinner.clear();

            let mut branch: Option<String> = None;
            while branch.is_none() {
                let branches = branches.clone();
                let git_branch: String = cliclack::input("Enter the name of the git branch:")
                    .validate(move |input: &String| {
                        if branches.contains(input) {
                            Ok::<(), _>(())
                        } else {
                            Err("The branch you entered does not exist in the 'stacks-core' repository. Please try again.".to_string())
                        }
                    })
                    .interact()?;

                branch = Some(git_branch);
            }

            GitTarget::new(GitTargetKind::Branch, &branch.unwrap())
        }
        GitTargetKind::Commit => {
            let git_commit: String = cliclack::input("Enter the commit hash:").interact()?;
            GitTarget::new(GitTargetKind::Commit, &git_commit)
        }
    };

    Ok(target)
}

async fn collect_bitcoin_version() -> Result<String> {
    use stackify_common::{ServiceType::BitcoinFollower, ServiceType::BitcoinMiner};

    let gh_config = GithubApiConfig::shared(GithubApiConfig::default());
    let gh_api = GithubApi::new(&gh_config);

    let mut releases = gh_api
        .repos
        .list_releases("bitcoin", "bitcoin")
        .send()
        .await?;

    releases.sort_by(|a, b| b.tag_name.cmp(&a.tag_name));

    let release_items: Vec<_> = releases
        .iter()
        .take(5)
        .map(|release| {
            (
                release.tag_name.clone(),
                release.tag_name.clone(),
                release.name.clone().unwrap_or_default(),
            )
        })
        .collect();

    let selected = cliclack::select("Select a Bitcoin Core release")
        .items(&release_items)
        .interact()
        .map(|release| release)?;

    Ok(selected)
}
