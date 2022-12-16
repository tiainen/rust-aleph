use aleph_bft::{Index, NodeIndex, SignatureSet};
use codec::{Decode, Encode};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Decode, Encode)]
pub struct Signature {
    msg: Vec<u8>,
    index: NodeIndex,
}

impl Index for Signature {
    fn index(&self) -> NodeIndex {
        self.index
    }
}

impl Signature {
    pub fn new(msg: Vec<u8>, index: NodeIndex) -> Self {
        Self {
            msg,
            index,
        }
    }

    pub fn msg(&self) -> &Vec<u8> {
        &self.msg
    }
}

pub type PartialMultisignature = SignatureSet<Signature>;
