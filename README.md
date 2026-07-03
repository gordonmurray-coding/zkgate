# zkgate — signatureless ZK-proof covenant unlock on Kaspa L1

A worked, end-to-end demonstration that a **RISC Zero zero-knowledge proof** can
authorize spending a Kaspa UTXO — no signature, no key — using the KIP-16
`OP_ZKPRECOMPILE` verifier shipped in the **Toccata** hard fork.

**Live proof (Testnet-10):**
[`ce84138050b214440354c89ab8a74437591ad853c7ab0471f5a7fe75bf35778c`](https://tn10.kaspa.stream/transactions/ce84138050b214440354c89ab8a74437591ad853c7ab0471f5a7fe75bf35778c?script=3763098394616786ecdcd73bfaef46141930e305896af1f6194dbae87f078ba3.0)
— confirmed on-chain, recognized by the explorer as a ZK precompile spend
(verification tag `0x21`, R0Succinct), witness contains proof data and **no
signature**.

> Status: research demonstration on **testnet**. The proven statement is a
> deliberately trivial hello-world (knowledge of a preimage behind a public
> commitment). It is the atomic unit of a shielded note, chosen so the circuit
> grows into a real one — but this is **not** a private-payments system yet, and
> nothing here should touch mainnet value.

## What it proves works

Post-Toccata, Kaspa L1 can gate a covenant spend on a ZK proof. Concretely,
this repo shows the full pipeline:

1. A RISC Zero guest program proves: *"I know `s`, `r` such that
   `commitment = sha256(s || r)`"* — revealing only the commitment.
2. A **succinct** receipt is produced and verified locally.
3. The receipt's eight fields are extracted in the exact order Kaspa's KIP-16
   `R0Succinct` (tag `0x21`) verifier expects.
4. An off-chain test runs the real receipt through Kaspa's **own** precompile
   (`TxScriptEngine`) and it verifies — proving correctness with zero network risk.
5. A P2SH covenant address is derived whose spending condition **is** proof
   validity.
6. That address is funded, then spent by a transaction whose witness is the
   five proof fields and **no signature**.

## Measured facts (the useful output)

These answer the "is it feasible / what does it cost" questions for anyone
building ZK apps on Kaspa:

- A succinct-receipt (`0x21`) verify costs **250,000 script units** →
  **2,500 compute-budget units** → compute mass **473,634**.
- Required fee floor at that mass: **~0.47 KAS** (we used 0.5).
- The succinct seal is **222,668 bytes** and **fits within mempool policy** —
  a full succinct receipt is submittable in one transaction.
- Groth16 (`0x20`) is cheaper (140k units) but requires re-encoding a RISC Zero
  proof into ark-bn254 form + a Docker stark2snark wrap; `0x21` takes the native
  succinct receipt directly. For a RISC Zero proof, `0x21` is the natural path.

## Layout

```
prover/                 RISC Zero (risc0 3.0.5)
  methods/guest/src/    the guest circuit (the statement)
  host/src/             prove + extract the 8 KIP-16 fields -> zk_fields.txt
kaspa/
  tests/                off-chain verification through Kaspa's own precompile
  bin/                  derive P2SH address; build+submit the ZK-unlock tx
docs/                   field-mapping notes, the KIP-16 0x21 stack layout
```

## Reproducing

Requires: risc0 3.0.5 (`rzup`), a Kaspa TN10 node with covenants active and
gRPC enabled, and a checkout of `rusty-kaspa` (the `kaspa/` files build against
its crates — see each file's header for placement).

1. `cd prover && cargo run --release -p host` → writes `zk_fields.txt`.
2. Drop `kaspa/tests/zkgate_proof_verifies.rs` + the seal/journal hex into the
   `rusty-kaspa` txscript crate; `cargo test` → proof verifies off-chain.
3. Derive the address (`kaspa/bin/zkgate_address.rs`), fund it from a wallet.
4. Fill the funding outpoint + payout address in `kaspa/bin/zkgate_unlock.rs`,
   build it inside `rusty-kaspa/rothschild` (has the RPC deps), run → submits.

## Security note

The seal / journal / secret in this repo correspond to the throwaway demo secret
`"correct horse battery staple"`. They are public *by design* — this is a
hello-world. A real shielded note's secret and blinding must **never** be
committed anywhere. The whole point of the eventual pool design is that the
secret stays in the proof and never leaves the prover.

## Where this goes

This is the validated primitive under two larger designs:
a covenant-native stablecoin settlement layer, and a shielded-pool covenant.
Both needed exactly this proven before they could be designed seriously.

MIT licensed. Built in one session; see commit history.
