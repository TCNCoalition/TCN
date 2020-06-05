use tcn::*;

#[test]
fn generate_temporary_contact_numbers_and_report_them() {
    // Generate a report authorization key.  This key represents the capability
    // to publish a report about a collection of derived temporary contact numbers.
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    // Use the temporary contact key ratchet mechanism to compute a list
    // of temporary contact numbers.
    let mut tck = rak.initial_temporary_contact_key(); // tck <- tck_1
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
            90,                       // Index of the last TCN to check
        )
        .expect("Report creation can only fail if the memo data is too long");

    // Verify the source integrity of the report...
    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    // ...allowing the disclosed TCNs to be recomputed.
    let recomputed_tcns = report.temporary_contact_numbers().collect::<Vec<_>>();

    // Check that the recomputed TCNs match the originals.
    // The slice is offset by 1 because tcn_0 is not included.
    assert_eq!(&recomputed_tcns[..], &tcns[20 - 1..90]);
}

#[test]
fn match_btreeset() {
    // Simulate many users generating TCNs, some of them being observed,
    // and comparison of observed TCNs against report data.

    // Use a BTreeSet to efficiently match TCNs.
    use rand::{
        distributions::{Bernoulli, Distribution},
        thread_rng,
    };
    use std::collections::BTreeSet;

    // Parameters.
    let num_reports = 10_000;
    let tcns_per_report: u16 = 24 * 60 / 15;
    let tcn_observation = Bernoulli::new(0.001).unwrap();

    // Store observed tcns.
    let mut observed_tcns = BTreeSet::new();

    // Generate some tcns that will be reported.
    let reports = (0..num_reports)
        .map(|_| {
            let rak = ReportAuthorizationKey::new(thread_rng());
            let mut tck = rak.initial_temporary_contact_key();
            for _ in 1..tcns_per_report {
                if tcn_observation.sample(&mut thread_rng()) {
                    observed_tcns.insert(tck.temporary_contact_number());
                }
                tck = tck.ratchet().expect("tcns_per_report < u16::MAX");
            }

            rak.create_report(MemoType::CoEpiV1, Vec::new(), 1, tcns_per_report)
                .expect("empty memo is not too long, so report creation cannot fail")
        })
        .collect::<Vec<_>>();

    // The current observed_tcns are exactly the ones that we expect will be reported.
    let expected_reported_tcns = observed_tcns.clone();

    // Generate some extra tcns that will not be reported.
    {
        let rak = ReportAuthorizationKey::new(thread_rng());
        let mut tck = rak.initial_temporary_contact_key();
        for _ in 1..60_000 {
            observed_tcns.insert(tck.temporary_contact_number());
            tck = tck.ratchet().expect("60_000 < u16::MAX");
        }
    }

    use std::time::Instant;

    println!("Expanding candidates");

    let expansion_start = Instant::now();
    // Now expand the reports into a second BTreeSet of candidates.
    let mut candidate_tcns = BTreeSet::new();
    for report in reports.into_iter() {
        let report = report.verify().expect("test reports should be valid");
        candidate_tcns.extend(report.temporary_contact_numbers());
    }
    let expansion_time = expansion_start.elapsed();

    println!(
        "Comparing {} candidates against {} observations",
        candidate_tcns.len(),
        observed_tcns.len()
    );

    let comparison_start = Instant::now();
    // Compute the intersection of the two BTreeSets.
    let reported_tcns = candidate_tcns
        .intersection(&observed_tcns)
        .cloned()
        .collect::<BTreeSet<_>>();
    let comparison_time = comparison_start.elapsed();

    assert_eq!(reported_tcns, expected_reported_tcns);

    println!(
        "Took {:?} (expansion) + {:?} (comparison) = {:?} (total)",
        expansion_time,
        comparison_time,
        (expansion_time + comparison_time),
    );
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

#[test]
fn report_with_j1_0_and_j2_1_generates_one_tcn() {
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    let signed_report = rak
        .create_report(MemoType::CoEpiV1, b"symptom data".to_vec(), 0, 1)
        .expect("Report creation can only fail if the memo data is too long");

    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    let recomputed_tcns = report.temporary_contact_numbers().collect::<Vec<_>>();
    assert_eq!(recomputed_tcns.len(), 1);
}

#[test]
fn report_with_j1_1_and_j2_1_generates_one_tcn() {
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    let signed_report = rak
        .create_report(MemoType::CoEpiV1, b"symptom data".to_vec(), 1, 1)
        .expect("Report creation can only fail if the memo data is too long");

    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    let recomputed_tcns = report.temporary_contact_numbers().collect::<Vec<_>>();
    assert_eq!(recomputed_tcns.len(), 1);
}

#[test]
fn report_with_j1_1_and_j2_2_generates_2_tcns() {
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    let signed_report = rak
        .create_report(MemoType::CoEpiV1, b"symptom data".to_vec(), 1, 2)
        .expect("Report creation can only fail if the memo data is too long");

    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    let recomputed_tcns = report.temporary_contact_numbers().collect::<Vec<_>>();
    assert_eq!(recomputed_tcns.len(), 2);
}

#[test]
fn report_with_j2_max_and_j1_max_minus_1_generates_2_tcns() {
    let rak = ReportAuthorizationKey::new(rand::thread_rng());

    let signed_report = rak
        .create_report(
            MemoType::CoEpiV1,
            b"symptom data".to_vec(),
            u16::MAX - 1,
            u16::MAX,
        )
        .expect("Report creation can only fail if the memo data is too long");

    let report = signed_report
        .verify()
        .expect("Valid reports should verify correctly");

    let recomputed_tcns = report.temporary_contact_numbers().collect::<Vec<_>>();
    assert_eq!(recomputed_tcns.len(), 2);
}
