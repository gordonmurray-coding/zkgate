use methods::{ZKGATE_GUEST_ELF, ZKGATE_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt, ProverOpts};
use risc0_zkvm::sha::Digestible;
use sha2::{Digest as ShaDigest, Sha256};

fn hexs(b: &[u8]) -> String { hex::encode(b) }

fn main() {
    let secret: Vec<u8> = b"correct horse battery staple".to_vec();
    let blinding: [u8; 32] = [7u8; 32];
    let env = ExecutorEnv::builder()
        .write(&secret).unwrap()
        .write(&blinding).unwrap()
        .build().unwrap();
    let prover = default_prover();
    println!("proving succinct...");
    let receipt = prover
        .prove_with_opts(env, ZKGATE_GUEST_ELF, &ProverOpts::succinct())
        .expect("succinct prove failed")
        .receipt;
    receipt.verify(ZKGATE_GUEST_ID).expect("local verify failed");
    println!("local verify OK\n");

    let hashfn: [u8; 1] = [0x01];
    let image_id: Vec<u8> =
        ZKGATE_GUEST_ID.iter().flat_map(|w| w.to_le_bytes()).collect();
    let journal_raw: Vec<u8> = receipt.journal.bytes.clone();
    let journal_digest: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(&receipt.journal.bytes);
        h.finalize().into()
    };
    let sr = match &receipt.inner {
        InnerReceipt::Succinct(sr) => sr,
        other => panic!("expected succinct receipt, got {other:?}"),
    };
    let control_id: Vec<u8> = sr.control_id.as_bytes().to_vec();
    let seal_bytes: Vec<u8> =
        sr.seal.iter().flat_map(|w| w.to_le_bytes()).collect();
    let control_index_u32: u32 = sr.control_inclusion_proof.index;
    let control_index_le: [u8; 4] = control_index_u32.to_le_bytes();
    let control_digests: Vec<u8> = sr
        .control_inclusion_proof.digests.iter()
        .flat_map(|d| d.as_bytes().to_vec()).collect();
    let claim_digest: Vec<u8> = sr.claim.digest().as_bytes().to_vec();

    println!("=== KIP-16 tag 0x21 stack fields ===");
    println!("hashfn            : {}", hexs(&hashfn));
    println!("control_id        : {}", hexs(&control_id));
    println!("image_id          : {}", hexs(&image_id));
    println!("journal (raw {}B) : {}", journal_raw.len(), hexs(&journal_raw));
    println!("journal_digest(ref): {}", hexs(&journal_digest));
    println!("seal_len_bytes    : {}", seal_bytes.len());
    println!("control_index u32 : {}", control_index_u32);
    println!("control_index le  : {}", hexs(&control_index_le));
    println!("control_digests   : {}", hexs(&control_digests));
    println!("claim_digest      : {}", hexs(&claim_digest));

    use std::io::Write;
    let mut f = std::fs::File::create("zk_fields.txt").unwrap();
    writeln!(f, "hashfn={}", hexs(&hashfn)).unwrap();
    writeln!(f, "control_id={}", hexs(&control_id)).unwrap();
    writeln!(f, "image_id={}", hexs(&image_id)).unwrap();
    writeln!(f, "journal={}", hexs(&journal_raw)).unwrap();
    writeln!(f, "seal={}", hexs(&seal_bytes)).unwrap();
    writeln!(f, "control_index={}", control_index_u32).unwrap();
    writeln!(f, "control_index_le={}", hexs(&control_index_le)).unwrap();
    writeln!(f, "control_digests={}", hexs(&control_digests)).unwrap();
    writeln!(f, "claim_digest={}", hexs(&claim_digest)).unwrap();
    println!("\nfields -> zk_fields.txt (seal {} bytes)", seal_bytes.len());
}
