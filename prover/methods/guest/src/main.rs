// zkgate guest — the statement being proven.
//
// Stage 1 statement (hello-world, but NOT throwaway):
//   "I know a secret preimage `s` and blinding `r` such that
//    commitment = sha256(s || r), and here is `commitment` publicly."
//
// Why this shape: it is the atomic unit of a shielded note. A shielded-pool
// note is exactly a commitment hash(value, owner, salt); unlocking it means
// proving knowledge of the preimage without revealing it. Stage 1 proves the
// smallest honest version of that — knowledge of a preimage behind a public
// commitment — so the circuit grows into the real thing instead of being
// discarded.
//
// PUBLIC (committed to the journal, visible to the on-chain verifier):
//   - commitment: [u8; 32]
// PRIVATE (witness, never leaves the proof):
//   - secret:   Vec<u8>
//   - blinding: [u8; 32]

#![no_main]

use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

risc0_zkvm::guest::entry!(main);

fn main() {
    // Read the private witness from the host.
    let secret: Vec<u8> = env::read();
    let blinding: [u8; 32] = env::read();

    // Recompute the commitment inside the zkVM.
    let mut hasher = Sha256::new();
    hasher.update(&secret);
    hasher.update(blinding);
    let commitment: [u8; 32] = hasher.finalize().into();

    // Commit ONLY the public output to the journal.
    // The secret and blinding stay inside the proof and are never revealed.
    env::commit(&commitment);
}
