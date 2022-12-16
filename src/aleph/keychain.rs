use aleph_bft::{Keychain, NodeCount, Index, NodeIndex, MultiKeychain, PartialMultisignature, SignatureSet};
use async_trait::async_trait;

use super::signature::Signature;

#[derive(Clone)]
pub struct SimpleKeychain {
    count: NodeCount,
    index: NodeIndex,
}

impl Index for SimpleKeychain {

    fn index(&self) -> aleph_bft::NodeIndex {
        self.index
    }
}

#[async_trait]
impl Keychain for SimpleKeychain {
    type Signature = Signature;

    fn node_count(&self) -> aleph_bft::NodeCount {
        self.count
    }

    async fn sign(&self, msg: &[u8]) -> Self::Signature {
        Signature::new(msg.to_vec(), self.index)
    }

    fn verify(&self, msg: &[u8], sgn: &Self::Signature, index: aleph_bft::NodeIndex) -> bool {
        index == sgn.index() && msg == sgn.msg()
    }
}

impl MultiKeychain for SimpleKeychain {
    type PartialMultisignature = super::signature::PartialMultisignature;

    fn bootstrap_multi(
        &self,
        signature: &Self::Signature,
        index: NodeIndex,
    ) -> Self::PartialMultisignature {
        SignatureSet::add_signature(SignatureSet::with_size(self.node_count()), signature, index)
    }

    fn is_complete(&self, msg: &[u8], partial: &Self::PartialMultisignature) -> bool {
        let signature_count = partial.iter().count();
        if signature_count < self.quorum() {
            return false;
        }
        partial.iter().all(|(i, sgn)| self.verify(msg, sgn, i))
    }
}

impl SimpleKeychain {

    pub fn new(count: NodeCount, index: NodeIndex) -> Self {
        Self {
            count, index
        }
    }

    fn quorum(&self) -> usize {
        2 * self.count.0 / 3 + 1
    }
}
