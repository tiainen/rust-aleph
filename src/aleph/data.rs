use aleph_bft::{DataProvider, NodeIndex};
use async_trait::async_trait;
use log::debug;

pub type Data = (NodeIndex, u32);

#[derive(Clone, Debug)]
pub struct SimpleDataProvider {
    index: NodeIndex,
    position: usize,
    data: Vec<u32>,
}

#[async_trait]
impl DataProvider<Data> for SimpleDataProvider {
    async fn get_data(&mut self) -> Option<Data> {
        debug!("SimpleDataProvider::get_data");

        if self.position >= self.data.len() {
            None
        } else {
            let current_data = self.data[self.position];
            self.position += 1;
            Some((self.index, current_data))
        }
    }
}

impl SimpleDataProvider {
    pub fn new(index: NodeIndex, data: Vec<u32>) -> Self {
        debug!("SimpleDataProvider::new() index={:?}, data={:?}", index, data);
        Self {
            index,
            position: 0,
            data,
        }
    }
}
