use tcn::*;

#[test]
fn generate_temporary_contact_numbers_and_report_them() {
    // Generate a report authorization key.  This key represents the capability
    // to publish a report about a collection of derived temporary contact numbers.
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    // Compute a list of temporary contact numbers by ratcheting the report's initial
    // temporary contact key.
    let mut tck = rak.initial_temporary_contact_key();
    let mut tcns = Vec::new();
    for _ in 0..100 {
        tcns.push(tck.temporary_contact_number());
        tck = tck.ratchet().unwrap();
    }

    // Prepare a report about a subset of the temporary contact numbers.
    let signed_report = rak
        .create_report(
            MemoType::CoEpiV1,        // The memo type
            b"symptom data".to_vec(), // The memo data
            20,                       // Index of the first TCN to disclose
            100,                      // Index of the last TCN to check
        )
        .expect("Report creation can only fail if the memo data is too long");

    // Verify the source integrity of the report...
    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    // ...allowing the disclosed TCNs to be recomputed.
    let recomputed_tcns = report.temporary_contact_numbers().collect::<Vec<_>>();

    // Check that the recomputed TCNs match the originals.
    assert_eq!(&recomputed_tcns[..], &tcns[20..100]);
}

#[test]
fn basic_read_write_round_trip() {
    use std::io::Cursor;

    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    rak.write(Cursor::new(&mut buf1))
        .expect("writing should succeed");
    ReportAuthorizationKey::read(Cursor::new(&buf1))
        .expect("reading should succeed")
        .write(Cursor::new(&mut buf2))
        .expect("writing should succeed");
    assert_eq!(buf1, buf2);

    let tck = rak.initial_temporary_contact_key();

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    tck.write(Cursor::new(&mut buf1))
        .expect("writing should succeed");
    TemporaryContactKey::read(Cursor::new(&buf1))
        .expect("reading should succeed")
        .write(Cursor::new(&mut buf2))
        .expect("writing should succeed");
    assert_eq!(buf1, buf2);

    let signed_report = rak
        .create_report(
            MemoType::CoEpiV1,        // The memo type
            b"symptom data".to_vec(), // The memo data
            20,                       // Index of the first TCN to disclose
            100,                      // Index of the last TCN to check
        )
        .expect("Report creation can only fail if the memo data is too long");

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    signed_report
        .write(Cursor::new(&mut buf1))
        .expect("writing should succeed");
    SignedReport::read(Cursor::new(&buf1))
        .expect("reading should succeed")
        .write(Cursor::new(&mut buf2))
        .expect("writing should succeed");
    assert_eq!(buf1, buf2);

    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    report
        .write(Cursor::new(&mut buf1))
        .expect("writing should succeed");
    Report::read(Cursor::new(&buf1))
        .expect("reading should succeed")
        .write(Cursor::new(&mut buf2))
        .expect("writing should succeed");
    assert_eq!(buf1, buf2);
}
