use std::fmt::Debug;

use cliclack::{log::warning, outro_note};
use color_eyre::Result;

use crate::{cli::theme::ThemedObject, db::errors::LoadEnvironmentError};

pub trait ReportResultExt {
    fn handle(self) -> Result<()>;
}

impl<T: Debug> ReportResultExt for Result<T> {
    fn handle(self) -> Result<()> {
        match self {
            Ok(_) => Ok(()),
            Err(ref e) => {
                if let Some(err) = e.downcast_ref::<LoadEnvironmentError>() {
                    match err {
                        LoadEnvironmentError::NotFound { env_name } => {
                            warning(format!(
                                "The {} environment does not exist.\n",
                                env_name.magenta()
                            ))?;
                            outro_note(
                                "Environment Not Found".bold().red(),
                                format!(
                                    "{} {} {}",
                                    "To create an environment, use the",
                                    "stackify env create".bold().white(),
                                    "command.".dimmed()
                                ),
                            )?;
                            Ok(())
                        }
                        LoadEnvironmentError::MissingParam {
                            service_name,
                            param_name,
                        } => {
                            warning(format!(
                                "The {} service is missing the {} parameter.\n",
                                service_name.magenta(),
                                param_name.cyan()
                            ))?;
                            outro_note(
                                "Configuration Error".bold().red(),
                                format!(
                                    "{} {} {}",
                                    "To add a parameter to the service, use the",
                                    "stackify env service config".bold().white(),
                                    "command.".dimmed()
                                ),
                            )?;
                            Ok(())
                        }
                        _ => Err(self.unwrap_err()),
                    }
                } else {
                    println!("{}", crate::cli::log::get_log().join("\n"));
                    Err(self.unwrap_err())
                }
            }
        }
    }
}
