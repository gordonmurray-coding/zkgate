use kaspa_txscript::{
    caches::Cache,
    pay_to_script_hash_script, pay_to_script_hash_signature_script_with_flags,
    script_builder::ScriptBuilder,
    opcodes::codes::OpZkPrecompile,
    EngineCtx, EngineFlags, TxScriptEngine,
};
use kaspa_txscript::zk_precompiles::tags::ZkTag;
use kaspa_consensus_core::{
    hashing::sighash::SigHashReusedValuesUnsync,
    tx::{PopulatedTransaction, Transaction, TransactionId, TransactionInput,
         TransactionOutpoint, UtxoEntry},
};

const IMAGE_ID: &str       = "c7810a1f3d4c22ad0a5f3708fcdc048b98bcd7eb6d805465aa3f7991cdf5b4da";
const CONTROL_ID: &str     = "c32b3627d2b3d60c64adf523a98bd16c0ff607471f3d6630d1f26d5e9406d841";
const HASHFN: &str         = "01";
const CLAIM: &str          = "16b2512ca8ec3bdac5b0a52fc69c977f07cd39f76b255e81c5d6f03740bd3a4a";
const CONTROL_INDEX: &str  = "06000000";
const CONTROL_DIGESTS: &str = "c9b08054994f542a6310b00d9b6fc6528ed7bb6f4ca5476a686847127cdfdc5bef137012c7d687610b46ea637164a17058e46826deebde361bb4394411d3630e5a3e7b624bdbeb31b396fb3c0563c2105ff4f545991b600ca40b1e76ee7f36079b98b73af20fba33f5ae2204d2fecb16bf1b3618572b652103ac9c722ef47e5b977f9e2868d664458077ac35fa9050290c7db016c2750620c362da3c275cab67f765ab6e0cf5dc55c11d65688af0fe1428afc359c08b1656bbc4ba6b54c9746cc6b87a237165c549ef7ac614d762ec1ce4b97441c9bfef6fd8ac90378170d8162be97040fd0b390959c33114712a436382b2cd419665ee2fe801c158a9bbb155";
const SEAL_HEX: &str    = include_str!("zkgate_seal.hex");
const JOURNAL_HEX: &str = include_str!("zkgate_journal.hex");

fn hexd(s: &str) -> Vec<u8> { kaspa_txscript::hex::decode(s.trim()).expect("bad hex") }
fn zk_flags() -> EngineFlags { EngineFlags { covenants_enabled: true, ..Default::default() } }

fn redeem_script(image_id: &[u8], control_id: &[u8], hashfn: &[u8]) -> Vec<u8> {
    ScriptBuilder::with_flags(zk_flags())
        .add_data(image_id).unwrap()
        .add_data(control_id).unwrap()
        .add_data(hashfn).unwrap()
        .add_data(&[ZkTag::R0Succinct as u8]).unwrap()
        .add_op(OpZkPrecompile).unwrap()
        .drain()
}

fn signature_script(redeem: Vec<u8>, claim: &[u8], control_index: &[u8],
                    control_digests: &[u8], seal: &[u8], journal: &[u8]) -> Vec<u8> {
    let mut sig = ScriptBuilder::with_flags(zk_flags());
    sig.add_data(claim).unwrap()
       .add_data(control_index).unwrap()
       .add_data(control_digests).unwrap()
       .add_data(seal).unwrap()
       .add_data(journal).unwrap();
    pay_to_script_hash_signature_script_with_flags(redeem, sig.drain(), zk_flags()).unwrap()
}

#[test]
fn zkgate_real_receipt_verifies_through_kip16() {
    let image_id = hexd(IMAGE_ID);
    let control_id = hexd(CONTROL_ID);
    let hashfn = hexd(HASHFN);
    let claim = hexd(CLAIM);
    let control_index = hexd(CONTROL_INDEX);
    let control_digests = hexd(CONTROL_DIGESTS);
    let seal = hexd(SEAL_HEX);
    let journal = hexd(JOURNAL_HEX);

    let redeem = redeem_script(&image_id, &control_id, &hashfn);
    let sig = signature_script(redeem.clone(), &claim, &control_index,
                               &control_digests, &seal, &journal);

    let outpoint = TransactionOutpoint::new(TransactionId::default(), 0);
    let input = TransactionInput::new_with_compute_budget(outpoint, sig, 0, 0);
    let utxo = UtxoEntry::new(20_000, pay_to_script_hash_script(&redeem), 0, false, None);
    let tx = Transaction::new(1, vec![input], vec![], 0, Default::default(), 0, vec![]);
    let populated = PopulatedTransaction::new(&tx, vec![utxo.clone()]);
    let reused = SigHashReusedValuesUnsync::new();
    let cache = Cache::new(0);

    let mut vm = TxScriptEngine::from_transaction_input(
        &populated, &populated.tx.inputs[0], 0, &utxo,
        EngineCtx::new(&cache).with_reused(&reused),
        zk_flags(),
    );
    match vm.execute() {
        Ok(()) => println!("\n*** zkgate proof VERIFIED through Kaspa KIP-16 R0Succinct ***\n"),
        Err(e) => panic!("verification failed: {e:?}"),
    }
}
