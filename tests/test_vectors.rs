use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use std::io::Cursor;

use tcn::*;

// Run cargo test generate_test_vectors -- --nocapture
#[test]
fn generate_test_vectors() {
    // This is insecure!! It's only done for reproducibility.
    let rng = ChaChaRng::from_seed([0x19; 32]);

    let rak = ReportAuthorizationKey::new(rng);

    let mut buf = Vec::new();
    rak.write(Cursor::new(&mut buf))
        .expect("writing should succeed");
    println!("rak:\n{}", hex::encode(&buf));

    let mut tck = rak.initial_temporary_contact_key();

    let mut buf = Vec::new();
    tck.write(Cursor::new(&mut buf))
        .expect("writing should succeed");
    let rvk_bytes = &buf[2..2 + 32];
    println!("rvk:\n{}", hex::encode(rvk_bytes));

    for i in 1..10 {
        assert_eq!(i, tck.index());
        let mut buf = Vec::new();
        tck.write(Cursor::new(&mut buf))
            .expect("writing should succeed");
        // Only print the tck_bytes themselves
        let tck_bytes = &buf[2 + 32..2 + 32 + 32];
        println!("tck_{}:\n{}", i, hex::encode(tck_bytes));

        let tcn = tck.temporary_contact_number();
        println!("tcn_{}:\n{}", i, hex::encode(&tcn.0));

        tck = tck.ratchet().unwrap();
    }

    let signed_report = rak
        .create_report(
            MemoType::CoEpiV1,        // The memo type
            b"symptom data".to_vec(), // The memo data
            2,                        // Index of the first TCN to disclose
            10,                       // Index of the last TCN to check
        )
        .expect("Report creation can only fail if the memo data is too long");

    let mut buf = Vec::new();
    signed_report
        .write(Cursor::new(&mut buf))
        .expect("writing should succeed");
    println!("signed_report:\n{}", hex::encode(&buf));
}
