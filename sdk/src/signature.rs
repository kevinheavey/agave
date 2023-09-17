//! Functionality for public and private keys.
#![cfg(feature = "full")]

// legacy module paths
pub use solana_signature_core::Signature;
pub use solana_signer::{keypair::*, null_signer::*, presigner::*, *};
use {
    solana_pubkey::Pubkey,
    std::borrow::{Borrow, Cow},
};

pub trait Signable {
    fn sign(&mut self, keypair: &solana_signer::keypair::Keypair) {
        let signature = keypair.sign_message(self.signable_data().borrow());
        self.set_signature(signature);
    }
    fn verify(&self) -> bool {
        self.get_signature()
            .verify(self.pubkey().as_ref(), self.signable_data().borrow())
    }

    fn pubkey(&self) -> Pubkey;
    fn signable_data(&self) -> Cow<[u8]>;
    fn get_signature(&self) -> Signature;
    fn set_signature(&mut self, signature: Signature);
}
