use std::{thread::{self, JoinHandle}, time::Duration};
use diesel::{Connection, SqliteConnection};
use stackify_common::{ServiceState, ServiceType};
use tokio::sync::mpsc::{channel, Sender};
use tokio::process::Child;

use color_eyre::{owo_colors::OwoColorize, Result};

use crate::db::DaemonDb;
use crate::db::model;

pub mod bitcoin;
pub mod stacks_node;
pub mod stacks_signer;
pub mod stacks_stacker;

/// This struct is responsible for monitoring the state of the services running
/// on the local node, as well as reporting status for remote services (i.e. services
/// running in other containers). The [`Monitor`] is capable of monitoring at
/// most one service of each type at a time -- if multiple local services need to
/// be monitored than multiple [`Monitor`] instances should be created.
/// 
/// Note that the Diesel [`SqliteConnection`] is not thread-safe, so each [`Monitor`]
/// will create its own connection to the database and use it for all operations.
/// TODO: This could be alleviated by running the SqliteConnection in its own
/// thread and using a channel to communicate with it.
pub struct Monitor {
    db: DaemonDb,
    data: Option<MonitorData>
}

/// Data struct which holds the state of the monitoring process for a single
/// service. This struct is used to store the state of the service, as well as
/// any child processes spawned by the monitor.
pub struct MonitorData {
    child: Option<Child>,
    service_type: ServiceType,
    expected_state: ServiceState,
    version: String
}

pub struct MonitorMsg {
    service_type: ServiceType,
    expected_state: ServiceState
}

impl Monitor {
    pub fn new<P: AsRef<str> + ?Sized>(db_path: &P) -> Result<Self> {
        let db_conn = SqliteConnection::establish(db_path.as_ref())?;
        let db = DaemonDb::new(db_conn);
        Ok(Self { 
            db,
            data: None
        })
    }

    /// Consumes the Monitor and starts the monitoring thread, returning its
    /// JoinHandle and a Sender to signal the thread to stop.
    pub fn start(mut self: Self) -> Result<(JoinHandle<()>, Sender<MonitorMsg>)> {
        let (sender, mut receiver) = channel::<MonitorMsg>(10);

        let handle = thread::spawn(move || {
            loop {
                println!("Monitoring...");

                match receiver.try_recv() {
                    Ok(_) => {
                        println!("Received stop signal.");
                        break;
                    }
                    Err(_) => {}
                }

                self.monitor_task()
                    .unwrap_or_else(|e| eprintln!("{}: {}", "Error".red(), e));

                thread::sleep(Duration::from_secs(1));
            }
        });

        Ok((handle, sender))
    }

    /// This function is called in a loop by the monitoring thread and is used
    /// as a router to call the appropriate monitoring function for this node's
    /// service type.
    fn monitor_task(&mut self) -> Result<()> {
        let services = self.db.list_services()?;
        if self.data.is_none() {
            return Ok(());
        }

        let mut data = self.data.take();

        if let Some(ref mut data) = data {
            for service in services {
                match ServiceType::from_i32(service.service_type_id)? {
                    ServiceType::BitcoinMiner => {
                        if service.is_local {
                            self.local_bitcoin_miner(&service, data)?;
                        } else {
                            self.remote_bitcoin_node(&service, data)?;
                        }
                    },
                    ServiceType::BitcoinFollower => {
                        if service.is_local {
                            self.local_bitcoin_follower(&service, data)?;
                        } else {
                            self.remote_bitcoin_node(&service, data)?;
                        }
                    },
                    ServiceType::StacksMiner => {
                        if service.is_local {
                            self.local_stacks_miner(&service, data)?;
                        } else {
                            self.remote_stacks_node(&service, data)?;
                        }
                    },
                    ServiceType::StacksFollower => {
                        if service.is_local {
                            self.local_stacks_follower(&service, data)?;
                        } else {
                            self.remote_stacks_node(&service, data)?;
                        }
                    },
                    ServiceType::StacksSigner => {
                        if service.is_local {
                            self.local_stacks_signer(&service, data)?;
                        } else {
                            self.remote_stacks_signer(&service, data)?;
                        }
                    },
                    ServiceType::StacksStackerSelf | ServiceType::StacksStackerPool => {
                        self.local_stacks_stacker(&service, data)?;
                    },
                    ServiceType::StackifyDaemon | ServiceType::StackifyEnvironment => {},
                }
            }
        }

        self.data = data;

        Ok(())
    }
    

    
}