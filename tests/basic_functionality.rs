use cen::*;

#[test]
fn generate_contact_event_numbers_and_report_them() {
    // Generate a report authorization key.  This key represents the capability
    // to publish a report about a collection of derived contact event numbers.
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    // Use the contact event key ratchet mechanism to compute a list of contact
    // event numbers.
    let mut cek = rak.initial_contact_event_key(); // cek <- cek_1
    let mut cens = Vec::new();
    for _ in 0..100 {
        cens.push(cek.contact_event_number());
        cek = cek.ratchet().unwrap();
    }

    // Prepare a report about a subset of the contact event numbers.
    let signed_report = rak
        .create_report(
            MemoType::CoEpiV1,        // The memo type
            b"symptom data".to_vec(), // The memo data
            20,                       // Index of the first CEN to disclose
            90,                       // Index of the last CEN to check
        )
        .expect("Report creation can only fail if the memo data is too long");

    // Verify the source integrity of the report...
    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    // ...allowing the disclosed CENs to be recomputed.
    let recomputed_cens = report.contact_event_numbers().collect::<Vec<_>>();

    // Check that the recomputed CENs match the originals.
    // The slice is offset by 1 because cen_0 is not included.
    assert_eq!(&recomputed_cens[..], &cens[20 - 1..90 - 1]);
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

    let cek = rak.initial_contact_event_key();

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    cek.write(Cursor::new(&mut buf1))
        .expect("writing should succeed");
    ContactEventKey::read(Cursor::new(&buf1))
        .expect("reading should succeed")
        .write(Cursor::new(&mut buf2))
        .expect("writing should succeed");
    assert_eq!(buf1, buf2);

    let signed_report = rak
        .create_report(
            MemoType::CoEpiV1,        // The memo type
            b"symptom data".to_vec(), // The memo data
            20,                       // Index of the first CEN to disclose
            100,                      // Index of the last CEN to check
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
