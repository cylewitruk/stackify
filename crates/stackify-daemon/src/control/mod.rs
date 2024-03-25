use std::{cell::RefCell, collections::HashMap, path::Path, process::{Child, Command}, rc::Rc, sync::Arc, thread::{self, JoinHandle}, time::Duration};
use diesel::{Connection, SqliteConnection};
use stackify_common::ServiceType;
use tokio::sync::mpsc::{channel, Sender, Receiver};

use color_eyre::{owo_colors::OwoColorize, Result};

use crate::db::DaemonDb;
use crate::db::model;

pub mod bitcoin;
pub mod stacks_node;
pub mod stacks_signer;
pub mod stacks_stacker;

pub struct Monitor {
    db: DaemonDb,
    children: HashMap<ServiceType, Child>
}

impl Monitor {
    pub fn new<P: AsRef<str> + ?Sized>(db_path: &P) -> Result<Self> {
        let db_conn = SqliteConnection::establish(db_path.as_ref())?;
        let db = DaemonDb::new(db_conn);
        Ok(Self { db, children: HashMap::new() })
    }

    /// Consumes the Monitor and starts the monitoring thread, returning its
    /// JoinHandle and a Sender to signal the thread to stop.
    pub fn start(self) -> Result<(JoinHandle<()>, Sender<()>)> {
        let (sender, mut receiver) = channel(10);

        let handle = thread::spawn(move || {
            loop {
                println!("Monitoring...");
                self.monitor_task()
                    .unwrap_or_else(|e| eprintln!("{}: {}", "Error".red(), e));
                thread::sleep(Duration::from_secs(1));

                match receiver.try_recv() {
                    Ok(_) => {
                        println!("Received stop signal.");
                        break;
                    }
                    Err(_) => {}
                }
            }
        });

        Ok((handle, sender))
    }

    fn monitor_task(&self) -> Result<()> {
        let services = self.db.list_services()?;

        for service in services {
            match ServiceType::from_i32(service.service_type_id)? {
                ServiceType::BitcoinMiner => {
                    
                    self.local_bitcoin_miner(service)?
                },
                ServiceType::BitcoinFollower => self.local_bitcoin_follower(service)?,
                ServiceType::StacksMiner => self.local_stacks_miner(service)?,
                ServiceType::StacksFollower => self.local_stacks_follower(service)?,
                ServiceType::StacksSigner => self.local_stacks_signer(service)?,
                ServiceType::StacksStackerSelf => self.local_stacks_stacker(service)?,
                ServiceType::StacksStackerPool => self.local_stacks_stacker(service)?,
                ServiceType::StackifyDaemon | ServiceType::StackifyEnvironment => {},
            }
        }

        let tmp = Command::new("bitcoind")
            .arg("/home/stacks/start-bitcoind.sh")
            .spawn()?;

        todo!()
    }

    fn local_bitcoin_miner(&self, service: model::Service) -> Result<()> {


        todo!()
    }

    fn local_bitcoin_follower(&self, service: model::Service) -> Result<()> {
        todo!()
    }

    fn local_stacks_miner(&self, service: model::Service) -> Result<()> {
        todo!()
    }

    fn local_stacks_follower(&self, service: model::Service) -> Result<()> {
        todo!()
    }

    fn local_stacks_signer(&self, service: model::Service) -> Result<()> {
        todo!()
    }

    fn local_stacks_stacker(&self, service: model::Service) -> Result<()> {
        todo!()
    }

    fn remote_bitcoin_node(&self, service: model::Service) -> Result<()> {
        todo!()
    }

    fn remote_stacks_node(&self, service: model::Service) -> Result<()> {
        todo!()
    }

    fn remote_stacks_signer(&self, service: model::Service) -> Result<()> {
        todo!()
    }
}