pub mod aleph;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path;

use crate::aleph::data::SimpleDataProvider;
use crate::aleph::finalizer::SimpleFinalizationHandler;
use crate::aleph::keychain::SimpleKeychain;
use crate::aleph::network::SimpleNetwork;
use crate::aleph::spawner::Spawner;

use aleph_bft::{run_session, Terminator, NodeIndex, NodeCount, LocalIO};
use clap::Parser;
use futures::StreamExt;
use futures::channel::oneshot;
use log::{debug, error, info};

#[derive(Debug, Parser)]
#[clap(name = "Aleph Simple")]
struct Args {
    /// Index of the node
    #[clap(long, short = 'i')]
    pub index: usize,
    /// Total number of nodes that will be running
    #[clap(long, short = 'c', default_value = "5")]
    pub node_count: usize,
    /// Total number of items to count
    #[clap(long, short = 'd')]
    pub data: String,
}

fn create_backup(node_id: NodeIndex) -> anyhow::Result<(fs::File, io::Cursor<Vec<u8>>)> {
    let stash_path = path::Path::new("./aleph-bft-examples-ordering-backup");
    fs::create_dir_all(&stash_path)?;
    let file_path = stash_path.join(format!("{}.units", node_id.0));
    let loader = if file_path.exists() {
        io::Cursor::new(fs::read(&file_path)?)
    } else {
        io::Cursor::new(Vec::new())
    };
    let saver = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    Ok((saver, loader))
}

fn finalized_counts(cf: &HashMap<NodeIndex, u32>) -> Vec<u32> {
    let mut v = cf
        .iter()
        .map(|(id, n)| (id.0, n))
        .collect::<Vec<(usize, &u32)>>();
    v.sort();
    v.iter().map(|(_, n)| **n).collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init_timed();

    let args = Args::parse();

    let index: NodeIndex = args.index.into();
    let node_count: NodeCount = args.node_count.into();

    let mut network = SimpleNetwork::new(index).await;
    for n in 0..node_count.0 {
        let port = 9900 + n;
        network.add_address(n, format!("127.0.0.1:{port}"));
    };

    let chars_as_u32 = args.data.chars().into_iter().map(|c| c as u32).collect();

    let data_length = args.data.len();
    let data_provider = SimpleDataProvider::new(index, chars_as_u32);
    let (finalization_handler, mut finalization_receiver) = SimpleFinalizationHandler::new();

    let (backup_saver, backup_loader) = create_backup(index)?;
    let local_io = LocalIO::new(data_provider, finalization_handler, backup_saver, backup_loader);

    let (exit_tx, exit_rx) = oneshot::channel();
    let member_terminator = Terminator::create_root(exit_rx, "AlephBFT-member");
    let member_handle = tokio::spawn(async move {
        let keychain = SimpleKeychain::new(node_count, index);
        let config = aleph_bft::default_config(node_count, index, 0);
        run_session(
            config,
            local_io,
            network,
            keychain,
            Spawner {},
            member_terminator,
        )
        .await
    });

    let mut count_finalized: HashMap<NodeIndex, u32> =
        (0..node_count.0).map(|c| (c.into(), 0)).collect();

    loop {
        match finalization_receiver.next().await {
            Some((id, char)) => {
                *count_finalized.get_mut(&id).unwrap() += 1;
                debug!(
                    "Finalized new item: node {:?}, char {:?}; total: {:?}",
                    id.0,
                    char::from_u32(char).unwrap_or('�'),
                    finalized_counts(&count_finalized)
                );
            }
            None => {
                error!(
                    "Finalization stream finished too soon. Got {:?} items, wanted {:?} items",
                    finalized_counts(&count_finalized),
                    data_length
                );
                panic!("Finalization stream finished too soon.");
            }
        }
        if count_finalized.values().all(|c| c >= &(data_length as u32)) {
            info!("Finalized required number of items.");
            info!("Waiting 10 seconds for other nodes...");
            tokio::time::sleep(core::time::Duration::from_secs(10)).await;
            info!("Shutdown.");
            break;
        }
    }

    exit_tx.send(()).expect("should send");
    member_handle.await.map_err(|e| e.into())
}
