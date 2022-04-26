///

use libp2p::{
    noise::{
        X25519Spec,
        AuthenticKeypair,
        Keypair as NoiseKeypair
    },
    identity::{
        self,
        Keypair
    },
    PeerId,
};
use getrandom::getrandom;

pub struct Keys {
    pub key: Keypair,
    pub peer_id: PeerId,
    pub noise_key: AuthenticKeypair<X25519Spec>,
}

impl Keys {
    pub fn new() -> Self {
        let mut secret_key_seed = [0u8; 32];
        GenKeyPair::generate_seed(&mut secret_key_seed[..]);
        let local_key = GenKeyPair::generate_ed25519(&mut secret_key_seed[..]);
        let local_public_key = local_key.public();
        let local_peer_id = PeerId::from(local_public_key.clone());

        let local_noise_key = NoiseKeypair::<X25519Spec>::new()
            .into_authentic(&local_key)
            .expect("Signing libp2p-noise static DH keypair failed.");
        
        Self {
            key: local_key,
            peer_id: local_peer_id,
            noise_key: local_noise_key,
        }
    }
}


pub trait GenKeyPair {
    fn generate_seed(&mut self);
    fn generate_ed25519(&mut self) -> Keypair;
}

impl GenKeyPair for u8 {
    fn generate_seed(&mut self) {
        let mut seed = [0u8; 1];
        if let Err(err) = getrandom(&mut seed) {
            panic!("getrandom failed: {}", err);
        }
        *self = seed[0];
    }
    fn generate_ed25519(&mut self) -> Keypair {
        let mut bytes = [0u8; 32];
        bytes[0] = *self;
    
        let secret_key = identity::ed25519::SecretKey::from_bytes(&mut bytes)
            .expect("this returns `Err` only if the length is wrong; the length is correct; qed");
        Keypair::Ed25519(secret_key.into())
    }
}

impl GenKeyPair for [u8] {
    fn generate_seed(&mut self) {
        if let Err(err) = getrandom(self) {
            panic!("getrandom failed: {}", err);
        }
    }
    fn generate_ed25519(&mut self) -> Keypair {
        let secret_key = identity::ed25519::SecretKey::from_bytes(self)
            .expect("this returns `Err` only if the length is wrong; the length is correct; qed");
        Keypair::Ed25519(secret_key.into())
    }
}
