use std::str::FromStr;
use kaspa_addresses::Address;
use kaspa_consensus_core::{
    constants::TX_VERSION_TOCCATA,
    mass::{ComputeBudget, ScriptUnits},
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{
        ComputeCommit, Transaction, TransactionInput, TransactionOutpoint,
        TransactionOutput, UtxoEntry,
    },
};
use kaspa_consensus_core::Hash;
use kaspa_grpc_client::GrpcClient;
use kaspa_notify::subscription::context::SubscriptionContext;
use kaspa_rpc_core::{api::rpc::RpcApi, notify::mode::NotificationMode};
use kaspa_txscript::{
    pay_to_address_script, pay_to_script_hash_script,
    pay_to_script_hash_signature_script_with_flags,
    script_builder::ScriptBuilder,
    opcodes::codes::OpZkPrecompile,
    EngineFlags,
};
use kaspa_txscript::zk_precompiles::tags::ZkTag;

const IMAGE_ID: &str   = "c7810a1f3d4c22ad0a5f3708fcdc048b98bcd7eb6d805465aa3f7991cdf5b4da";
const CONTROL_ID: &str = "c32b3627d2b3d60c64adf523a98bd16c0ff607471f3d6630d1f26d5e9406d841";
const HASHFN: &str     = "01";
const CLAIM: &str      = "16b2512ca8ec3bdac5b0a52fc69c977f07cd39f76b255e81c5d6f03740bd3a4a";
const CONTROL_INDEX: &str  = "06000000";
const CONTROL_DIGESTS: &str = "c9b08054994f542a6310b00d9b6fc6528ed7bb6f4ca5476a686847127cdfdc5bef137012c7d687610b46ea637164a17058e46826deebde361bb4394411d3630e5a3e7b624bdbeb31b396fb3c0563c2105ff4f545991b600ca40b1e76ee7f36079b98b73af20fba33f5ae2204d2fecb16bf1b3618572b652103ac9c722ef47e5b977f9e2868d664458077ac35fa9050290c7db016c2750620c362da3c275cab67f765ab6e0cf5dc55c11d65688af0fe1428afc359c08b1656bbc4ba6b54c9746cc6b87a237165c549ef7ac614d762ec1ce4b97441c9bfef6fd8ac90378170d8162be97040fd0b390959c33114712a436382b2cd419665ee2fe801c158a9bbb155";
const SEAL_HEX: &str    = include_str!("zkgate_seal.hex");
const JOURNAL_HEX: &str = include_str!("zkgate_journal.hex");

const FUND_TXID: &str = "3763098394616786ecdcd73bfaef46141930e305896af1f6194dbae87f078ba3";
const FUND_INDEX: u32 = 0;
const FUND_AMOUNT: u64 = 500_000_000;

const PAYOUT_ADDR: &str = "kaspatest:qrv56qnz2eya80zzs9v0k2jzuw6nwqlrlgv7vlrfjmnflauukc0hzg03xu0jl";

const RPC: &str = "localhost:16110";

fn hexd(s: &str) -> Vec<u8> { kaspa_txscript::hex::decode(s.trim()).expect("bad hex") }

#[tokio::main]
async fn main() {
    let flags = EngineFlags { covenants_enabled: true, ..Default::default() };

    let redeem = ScriptBuilder::with_flags(flags)
        .add_data(&hexd(IMAGE_ID)).unwrap()
        .add_data(&hexd(CONTROL_ID)).unwrap()
        .add_data(&hexd(HASHFN)).unwrap()
        .add_data(&[ZkTag::R0Succinct as u8]).unwrap()
        .add_op(OpZkPrecompile).unwrap()
        .drain();
    let covenant_spk = pay_to_script_hash_script(&redeem);

    let mut sig = ScriptBuilder::with_flags(flags);
    sig.add_data(&hexd(CLAIM)).unwrap()
       .add_data(&hexd(CONTROL_INDEX)).unwrap()
       .add_data(&hexd(CONTROL_DIGESTS)).unwrap()
       .add_data(&hexd(SEAL_HEX)).unwrap()
       .add_data(&hexd(JOURNAL_HEX)).unwrap();
    let signature_script =
        pay_to_script_hash_signature_script_with_flags(redeem.clone(), sig.drain(), flags).unwrap();

    let budget = ComputeBudget::checked_covering_script_units(ScriptUnits::from(ZkTag::R0Succinct.cost()))
        .expect("zk cost exceeds max compute budget");
    println!("compute budget units: {}", budget.value());

    let txid = Hash::from_str(FUND_TXID).expect("bad txid");
    let input = TransactionInput {
        previous_outpoint: TransactionOutpoint::new(txid, FUND_INDEX),
        signature_script,
        sequence: 0,
        compute_commit: ComputeCommit::ComputeBudget(budget),
    };

    let fee: u64 = 50_000_000;
    let payout = Address::try_from(PAYOUT_ADDR).expect("bad payout addr");
    let output = TransactionOutput {
        value: FUND_AMOUNT - fee,
        script_public_key: pay_to_address_script(&payout),
        covenant: None,
    };

    let tx = Transaction::new(
        TX_VERSION_TOCCATA,
        vec![input],
        vec![output],
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        vec![],
    );

    let _utxo = UtxoEntry::new(FUND_AMOUNT, covenant_spk, 0, false, None);

    let sub_ctx = SubscriptionContext::new();
    let client = GrpcClient::connect_with_args(
        NotificationMode::Direct,
        format!("grpc://{RPC}"),
        Some(sub_ctx),
        true, None, false, Default::default(), Default::default(),
    ).await.expect("connect failed");

    println!("submitting zk-unlock tx {} ...", tx.id());
    match client.submit_transaction((&tx).into(), false).await {
        Ok(resp) => println!("\n*** SUBMITTED — zk-unlock accepted: {resp:?} ***"),
        Err(e) => println!("\nsubmit error (informative, tune & retry): {e}"),
    }
}
