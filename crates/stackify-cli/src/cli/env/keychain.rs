use clap::{Args, Subcommand};
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use docker_api::conn::TtyChunk;
use futures_util::StreamExt;
use prettytable::row;
use stackify_common::types::EnvironmentName;
use textwrap::Options;

use crate::{
    cli::{
        context::CliContext,
        env::prompt_environment_name,
        log::clilog,
        theme::{ThemedObject, THEME},
    },
    db::cli_db::CliDatabase,
    errors::CliError,
    util::stacks_cli::MakeKeychainResult,
};

#[derive(Debug, Args)]
pub struct KeychainArgs {
    #[command(subcommand)]
    pub commands: KeychainSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum KeychainSubCommands {
    /// Adds a new Stacks keychain to the specified environment.
    New(KeychainNewArgs),
    /// Removes the specified keychain from the environment.
    #[clap(visible_alias = "rm")]
    Remove(KeychainRemoveArgs),
    /// List all keychains for the environment.
    List(KeychainListArgs),
}

#[derive(Debug, Args)]
pub struct KeychainNewArgs {
    /// The name of the environment.
    #[arg(required = false, value_name = "ENV_NAME")]
    pub env_name: Option<String>,
}

#[derive(Debug, Args)]
pub struct KeychainRemoveArgs {
    /// The stack address of the keychain to remove.
    #[arg(required = true, value_name = "STX_ADDRESS")]
    pub stx_address: String,
}

#[derive(Debug, Args)]
#[clap(visible_alias = "ls")]
pub struct KeychainListArgs {
    /// The name of the environment.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

pub async fn exec(ctx: &CliContext, args: KeychainArgs) -> Result<()> {
    match args.commands {
        KeychainSubCommands::List(inner_args) => exec_list(ctx, inner_args).await,
        KeychainSubCommands::New(inner_args) => exec_new(ctx, inner_args).await,
        KeychainSubCommands::Remove(inner_args) => exec_remove(ctx, inner_args).await,
    }
}

async fn exec_list(ctx: &CliContext, args: KeychainListArgs) -> Result<()> {
    let env = ctx.db.as_clidb().load_environment(&args.env_name)?;
    cliclack::intro(format!(
        "Listing keychains for environment '{}'",
        &args.env_name.bold().magenta()
    ))?;

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row!["Wallet".cyan().bold(), "Details".cyan().bold(),]);

    for kc in env.keychains.iter() {
        let mut details_table = prettytable::Table::new();
        details_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        let starting_balance = kc
            .amount
            .to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(std::str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap()
            .join(",");
        let starting_balance_qualifier = if kc.amount >= 1_000_000_000_000_000 {
            "(Quadrillion)"
        } else if kc.amount >= 1_000_000_000_000 {
            "(Trillion)"
        } else if kc.amount >= 1_000_000_000 {
            "(Billion)"
        } else if kc.amount >= 1_000_000 {
            "(Million)"
        } else if kc.amount >= 1_000 {
            "(Thousand)"
        } else {
            "(Hundred)"
        };
        details_table.add_row(row![
            "Starting Balance".bold(),
            format!(
                "STX {} {}",
                starting_balance,
                starting_balance_qualifier.gray()
            )
        ]);
        details_table.add_row(row!["Public Key".bold(), kc.public_key]);
        details_table.add_row(row!["Private Key".bold(), kc.private_key]);
        details_table.add_row(row!["Mnemonic".bold(), textwrap::fill(&kc.mnemonic, 60)]);
        let remark = kc.remark.clone().unwrap_or("<none>".gray().to_string());
        details_table.add_row(row![
            "Remark".bold(),
            if remark.is_empty() {
                "<none>".gray().to_string()
            } else {
                remark
            }
        ]);

        let wallet_text = format!(
            "{}\n{}\n\n{}\n{}",
            "Stacks Address".bold(),
            kc.stx_address,
            "Bitcoin Address".bold(),
            kc.btc_address
        );

        table.add_row(row![wallet_text, details_table]);
    }

    let mut lines = vec![];
    table.print(&mut lines)?;

    let table_str = String::from_utf8_lossy(&lines);
    for line in table_str.lines() {
        println!("{} {}", "│".bright_black(), line);
    }

    cliclack::outro(format!("{} keychains", env.keychains.len()))?;

    Ok(())
}

async fn exec_new(ctx: &CliContext, args: KeychainNewArgs) -> Result<()> {
    cliclack::intro("Generate new keychain".bold())?;
    let env_name = match args.env_name {
        Some(env_name) => EnvironmentName::new(&env_name)?,
        None => prompt_environment_name(ctx)?,
    };
    let env = ctx.db.load_environment(env_name.as_ref())?;

    let keychain = generate_stacks_keychain(ctx).await?;

    let balance: u64 = cliclack::input("Balance:")
        .required(false)
        .placeholder("10000000000000000")
        .default_input("10000000000000000")
        .interact()?;

    let remark: String = cliclack::input("Comment:")
        .placeholder("Write a short remark about this keychain")
        .required(false)
        .interact()?;

    ctx.db.add_environment_keychain(
        env.id,
        &keychain.key_info.address,
        &keychain.key_info.btc_address,
        &keychain.key_info.public_key,
        &keychain.key_info.private_key,
        &keychain.mnemonic,
        balance,
        &remark,
    )?;

    cliclack::outro("Keychain has been successfully added to the environment")?;

    Ok(())
}

async fn exec_remove(ctx: &CliContext, args: KeychainRemoveArgs) -> Result<()> {
    cliclack::intro(format!("Remove keychain '{}'", &args.stx_address.bold()))?;

    let keychain = match ctx
        .db
        .get_environment_keychain_by_stx_address(&args.stx_address)?
    {
        Some(kc) => kc,
        None => {
            bail!(CliError::Graceful {
                title: "Keychain not found".to_string(),
                message: "The specified keychain was not found in the environment".to_string()
            });
        }
    };

    let confirm = cliclack::confirm("Are you sure you want to remove this keychain?").interact()?;

    if confirm {
        ctx.db.delete_environment_keychain(&keychain.stx_address)?;
        cliclack::outro("Keychain has been successfully removed from the environment")?;
    } else {
        cliclack::outro("Keychain removal has been cancelled")?;
    }

    Ok(())
}

async fn generate_stacks_keychain(ctx: &CliContext) -> Result<MakeKeychainResult> {
    let generate_keychain_spinner = cliclack::spinner();
    generate_keychain_spinner.start("Generating new keychain...");
    let cli = ctx
        .docker()
        .api()
        .containers()
        .create(&ctx.docker().opts_for().generate_stacks_keychain())
        .await?;
    let mut cli_attach = cli.attach().await?;
    let mut cli_stdout = vec![];
    let mut cli_stderr = vec![];
    cli.start().await?;
    while let Some(result) = cli_attach.next().await {
        match result {
            Ok(chunk) => match chunk {
                TtyChunk::StdOut(data) => {
                    let str = String::from_utf8(data)?;
                    cli_stdout.push(str);
                }
                TtyChunk::StdErr(data) => {
                    let str = String::from_utf8(data)?;
                    cli_stderr.push(str);
                }
                TtyChunk::StdIn(_) => {
                    clilog!("received stdin tty chunk while executing stacks cli, this shouldn't happen");
                }
            },
            Err(e) => {
                cliclack::log::error(format!("Error: {}", e))?;
            }
        }
    }

    let cli_result = cli.wait().await?;

    if cli_result.status_code == 0 {
        let keychain = MakeKeychainResult::from_json(&cli_stdout.join(""))?;
        generate_keychain_spinner.stop(format!(
            "{} Generate keychain",
            THEME.read().unwrap().success_symbol()
        ));
        let mut msg_lines = vec![];
        let wrapped_mnemonic = textwrap::wrap(
            &keychain.mnemonic,
            Options::new(80).subsequent_indent("                "),
        );
        msg_lines.push(
            "A new keychain has been generated for this service. Here are the details:\n"
                .cyan()
                .to_string(),
        );
        msg_lines.push(format!("‣ Mnemonic:     {}", wrapped_mnemonic.join("\n")));
        msg_lines.push(format!("‣ Private Key:  {}", keychain.key_info.private_key));
        msg_lines.push(format!("‣ Public Key:   {}", keychain.key_info.public_key));
        msg_lines.push(format!("‣ STX Address:  {}", keychain.key_info.address));
        msg_lines.push(format!("‣ BTC Address:  {}", keychain.key_info.btc_address));
        msg_lines.push(format!("‣ WIF:          {}", keychain.key_info.wif));
        msg_lines.push(format!("‣ Index:        {}", keychain.key_info.index));
        cliclack::note("Keychain", msg_lines.join("\n"))?;

        Ok(keychain)
    } else {
        bail!("Failed to generate keychain: {:?}", cli_stderr);
    }
}
