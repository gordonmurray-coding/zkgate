use kaspa_addresses::Prefix;
use kaspa_txscript::{
    pay_to_script_hash_script,
    script_builder::ScriptBuilder,
    opcodes::codes::OpZkPrecompile,
    extract_script_pub_key_address,
    EngineFlags,
};
use kaspa_txscript::zk_precompiles::tags::ZkTag;

const IMAGE_ID: &str   = "c7810a1f3d4c22ad0a5f3708fcdc048b98bcd7eb6d805465aa3f7991cdf5b4da";
const CONTROL_ID: &str = "c32b3627d2b3d60c64adf523a98bd16c0ff607471f3d6630d1f26d5e9406d841";
const HASHFN: &str     = "01";

fn hexd(s: &str) -> Vec<u8> { kaspa_txscript::hex::decode(s.trim()).expect("bad hex") }

fn main() {
    let flags = EngineFlags { covenants_enabled: true, ..Default::default() };
    let redeem = ScriptBuilder::with_flags(flags)
        .add_data(&hexd(IMAGE_ID)).unwrap()
        .add_data(&hexd(CONTROL_ID)).unwrap()
        .add_data(&hexd(HASHFN)).unwrap()
        .add_data(&[ZkTag::R0Succinct as u8]).unwrap()
        .add_op(OpZkPrecompile).unwrap()
        .drain();
    let spk = pay_to_script_hash_script(&redeem);
    let addr = extract_script_pub_key_address(&spk, Prefix::Testnet)
        .expect("derive address failed");
    println!("\n=== FUND THIS ADDRESS from your GUI wallet ===");
    println!("{addr}");
    println!("\n(send a small amount, e.g. 5 TN10 KAS)");
}
