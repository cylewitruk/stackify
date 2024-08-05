use color_eyre::{eyre::bail, Result};
use docker_api::conn::TtyChunk;
use futures_util::StreamExt;
use stackify_common::types::EnvironmentName;
use textwrap::Options;

use crate::{cli::{context::CliContext, env::prompt_environment_name, log::clilog, theme::{ThemedObject, THEME}}, db::cli_db::CliDatabase, util::stacks_cli::MakeKeychainResult};

use super::args::{KeychainArgs, KeychainListArgs, KeychainNewArgs, KeychainRemoveArgs, KeychainSubCommands};

pub async fn exec(ctx: &CliContext, args: KeychainArgs) -> Result<()> {
    match args.commands {
        KeychainSubCommands::List(inner_args) => exec_list(ctx, inner_args).await,
        KeychainSubCommands::New(inner_args) => exec_new(ctx, inner_args).await,
        KeychainSubCommands::Remove(inner_args) => exec_remove(ctx, inner_args).await,

    }
}

async fn exec_list(ctx: &CliContext, args: KeychainListArgs) -> Result<()> {
    todo!()
}

async fn exec_new(ctx: &CliContext, args: KeychainNewArgs) -> Result<()> {
    cliclack::intro("Generate new keychain".bold())?;
    let env_name = match args.env_name {
        Some(env_name) => EnvironmentName::new(&env_name)?,
        None => prompt_environment_name(ctx)?,
    };
    let env = ctx.db.load_environment(env_name.as_ref())?;
    
    cliclack::intro("Generate New Keychain")?;
    let keychain = generate_stacks_keychain(ctx).await?;
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
        &remark
    )?;

    cliclack::outro("Keychain has been successfully added to the environment")?;
    
    Ok(())
}

async fn exec_remove(ctx: &CliContext, args: KeychainRemoveArgs) -> Result<()> {
    todo!()
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
            "{} Generate new keychain",
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