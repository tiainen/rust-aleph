use aleph_bft::{DataProvider, NodeIndex};
use async_trait::async_trait;
use log::debug;

pub type Data = (NodeIndex, Vec<u8>);

#[derive(Clone, Debug)]
pub struct SimpleDataProvider {
    index: NodeIndex,
    position: usize,
    data: Vec<String>,
}

#[async_trait]
impl DataProvider<Data> for SimpleDataProvider {
    async fn get_data(&mut self) -> Option<Data> {
        debug!("SimpleDataProvider::get_data");

        if self.position >= self.data.len() {
            None
        } else {
            let current_data = String::from(&self.data[self.position]);
            self.position += 1;
            Some((self.index, current_data.as_bytes().to_owned()))
        }
    }
}

impl SimpleDataProvider {
    pub fn new(index: NodeIndex, data: Vec<String>) -> Self {
        debug!("SimpleDataProvider::new() index={:?}, data={:?}", index, data);
        Self {
            index,
            position: 0,
            data,
        }
    }
}
