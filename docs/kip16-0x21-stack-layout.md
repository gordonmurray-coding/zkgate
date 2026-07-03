# KIP-16 tag 0x21 (R0Succinct) — stack layout

Verified against rusty-kaspa `crypto/txscript/src/zk_precompiles/risc0/mod.rs`
and confirmed on-chain (tx ce84138050b214440354c89ab8a74437591ad853c7ab0471f5a7fe75bf35778c).

## Redeem script (locking; committed to the P2SH address)
```
PUSH image_id (32)      // RISC Zero guest/image ID
PUSH control_id (32)    // recursion control ID
PUSH hashfn (1)         // 0x01 = poseidon2 (only supported hashfn)
PUSH 0x21               // ZkTag::R0Succinct
OP_ZKPRECOMPILE
```

## Signature script (witness; provided at spend — NO signature)
```
PUSH claim         (32)  // receipt claim DIGEST (not expanded claim)
PUSH control_index (4)   // u32 LE, the Merkle inclusion index
PUSH control_digests     // concatenated 32-byte inclusion-proof digests
PUSH seal                // succinct STARK seal, u32-LE bytes (222,668 for this guest)
PUSH journal       (32)  // journal DIGEST = sha256(journal bytes)
```

## The two gotchas (each cost a wrong-encoding round)
- `journal` on the stack is the 32-byte DIGEST (sha256 of raw journal), NOT the
  raw journal. Verifier says "Invalid digest length: 128" if you pass raw.
- `claim` on the stack is the claim DIGEST, not the expanded ReceiptClaim.

## Cost (measured on TN10)
250,000 script units -> 2,500 compute-budget units -> compute mass 473,634
-> fee floor ~0.47 KAS. Seal 222,668 bytes fits mempool policy.
