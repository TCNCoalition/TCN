use cen::*;

#[test]
fn generate_contact_event_numbers_and_report_them() {
    // Generate a report authorization key.  This key represents the capability
    // to publish a report about a collection of derived contact event numbers.
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    // Use the contact event key ratchet mechanism to compute a list of contact
    // event numbers.
    let mut cek = rak.initial_cek();
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
            100,                      // Index of the last CEN to check
        )
        .expect("Report creation can only fail if the memo data is too long");

    // Verify the source integrity of the report...
    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    // ...allowing the disclosed CENs to be recomputed.
    let recomputed_cens = report.contact_event_numbers().collect::<Vec<_>>();

    // Check that the recomputed CENs match the originals.
    assert_eq!(&recomputed_cens[..], &cens[20..100]);
}
