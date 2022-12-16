use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use aleph_bft::{Network, NodeIndex, Recipient};
use async_trait::async_trait;
use codec::{Decode, Encode};
use log::{debug, error, info, warn};
use tokio::net::UdpSocket;

use super::data::Data;
use super::hasher::Hasher64;
use super::signature::{PartialMultisignature, Signature};

pub type NetworkData = aleph_bft::NetworkData<Hasher64, Data, Signature, PartialMultisignature>;

pub struct SimpleNetwork {
    index: NodeIndex,
    addresses: HashMap<NodeIndex, SocketAddr>,
    socket: UdpSocket,
    buffer: Box<[u8; 65536]>,
}

#[async_trait]
impl Network<NetworkData> for SimpleNetwork {

    fn send(&self, data: NetworkData, recipient: aleph_bft::Recipient) {
        debug!("SimpleNetwork::send");
        match recipient {
            Recipient::Everyone => {
                debug!("Need to send data {data:?} to all nodes.");
                for address in &self.addresses {
                    if *address.0 != self.index {
                        self.send_data(*address.1, data.clone());
                    }
                }
            },
            Recipient::Node(node) => {
                debug!("Need to send data {data:?} to single node: {recipient:?}.");
                if let Some(address) = self.addresses.get(&node) {
                    self.send_data(*address, data.clone());
                } else {
                    info!("Could not find address for recipient with index {node:?}.");
                }
            },
        }
    }

    async fn next_event(&mut self) -> Option<NetworkData> {
        debug!("SimpleNetwork::next_event");
        match self.socket.recv_from(self.buffer.as_mut()).await {
            Ok((_len, _address)) => NetworkData::decode(&mut &self.buffer[..]).ok(),
            Err(e) => {
                error!("Couldn't receive datagram: {:?}", e);
                None
            },
        }
    }
}

impl SimpleNetwork {

    pub async fn new(index: NodeIndex) -> Self {
        let port = 9900 + index.0;
        let socket = Self::bind_socket(&format!("127.0.0.1:{port}")).await;
        Self {
            index,
            addresses: HashMap::new(),
            socket,
            buffer: Box::new([0; 65536]),
        }
    }

    async fn bind_socket(address: &str) -> UdpSocket {
        loop {
            match UdpSocket::bind(address).await {
                Ok(socket) => {
                    return socket;
                }
                Err(e) => {
                    error!("{}", e);
                    error!("Waiting 10 seconds before the next attempt...");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            };
        }
    }

    pub fn add_address(&mut self, idx: usize, address: String) {
        self.addresses.insert(NodeIndex(idx), address.parse::<SocketAddr>().unwrap());
    }

    fn send_data(&self, address: SocketAddr, data: NetworkData) {
        info!("Sending data {:?} to {:?}", data, address);

        let encoded_data = data.encode();
        if let Err(e) = self.socket.try_send_to(&encoded_data, address) {
            warn!("Failed to write data to recipient at address {:?}: {:?}", address, e);
        }
    }
}
