# CEN Protocol

> This is a work-in-progress document. Changes are tracked through PRs
> and issues.

This document describes **Contact Event Numbers**, a decentralized,
privacy-first contact tracing protocol developed by [CoEpi] and [CovidWatch].
However, this protocol is not limited to use solely by CoEpi or CovidWatch, and
is built to be extensible, with the goal of being used more broadly.
The CEN protocol and related efforts are designed with the [Contact Tracing
Bill of Rights](/ContactTracingBillOfRights.md) in mind.

No personally-identifiable information is required by the
protocol, and although it is compatible with a trusted health authority, it
does not require one. Users' devices send short-range broadcasts over
Bluetooth to nearby devices. Later, a user who develops symptoms or tests
positive can report their status to their contacts with minimal loss of
privacy. Users who do not send reports reveal no information. Different
applications using the CEN protocol can interoperate, and the protocol can be
used with either verified test results or for self-reported symptoms via an
extensible report memo field.

PRs and Issues are welcome to be submitted directly to this repo. For questions
about the CEN Protocol or collaborating in more detail, contact
[Henry](mailto:hdevalence@hdevalence.ca?Subject=CEN%20Protocol) or
[Dana](mailto:dana+CoEpi@OpenAPS.org?Subject=CEN%20Protocol).

This repository also contains a reference implementation of the CEN protocol
written in Rust. View documentation by running `cargo doc --no-deps --open`,
and run tests by running `cargo test`.

**What's on this page:**

- [Introduction](#cen-protocol)
- [Ideal functionality and trust assumptions in contact tracing systems](#ideal-functionality-and-trust-assumptions-in-contact-tracing-systems)
- [A strawman protocol](#a-strawman-protocol)
- [The CEN Protocol](#the-cen-protocol)
- [CEN sharing with Bluetooth Low Energy](#cen-sharing-with-bluetooth-low-energy)
- [Contributors](#contributors)

As it is a work-in-progress, this page also contains [rough notes, yet to be
merged with the main document](#notes-to-be-merged-with-main-document)

## Ideal functionality and trust assumptions in contact tracing systems

Cryptography builds systems that mediate and rearrange trust, so before
beginning discussion of cryptographic approaches to contact tracing, it's
worthwhile to delineate categories of trust involved in the problem.

1.  **Location Privacy**.  Is any party trusted with access to location data,
    and if so, under what circumstances?  Because a contact tracing system
    allows users to report potential exposure to other users, this category can
    be usefully subdivided into *reporter privacy* and *receiver privacy*.

2.  **Functional Capacity**.  Does the system trust that health authorities
    will be able to carry out their functions, or is it resilient in case they
    become overwhelmed and are unable to?

3.  **Report Integrity**.  What measures does the system use, if any, to
    determine the integrity of a report of symptoms or test status?

Contact tracing is used to identify people who may have been exposed to
infection and notify them of their exposure, allowing isolation, testing, or
treatment as may be appropriate.  However, contact tracing poses risks of its
own, such as fear of stigma or discrimination based on health status, or the
risk that contact tracing systems could be repurposed for surveillance by
governments or individuals.  This makes location privacy paramount.

However, trust in functional capacity is also problematic. In an ideal world,
health authorities would have unlimited resources and perfect effectiveness
in deploying them. But in the real world, health authorities have limited
resources, are strained under the burden of dealing with the epidemic, or may
fail to respond adequately or effectively. Indeed, each of these
possibilities has already happened during the current epidemic. While no
technological system can properly compensate for institutional failure, a
system that is resilient to failure can potentially absorb slack and give
people agency to help themselves.

Moreover, a protocol that places additional burdens on health authorities
(e.g., requiring them to deploy complex cryptography like MPC or carefully
manage cryptographic key material) faces severe adoption barriers to one that
does not, so reducing trust requirements may allow accelerated deployment.

For these reasons, it seems preferable to design a protocol that does not
require participation by any health authority, but is optionally compatible
with health authorities that verify report integrity (e.g., by sending
reports to a portal that signs them on behalf of the health authority or
allowing the authorities to generate URLs that pass an authenticated
positive diagnosis result to an app).
Leaving the question of report integrity as an application-level concern
means that different applications can make different choices, while still
remaining interoperable. For instance, [CoEpi] allows users to self-report
symptoms, while [CovidWatch] trusts a health authority to attest to the
integrity of a positive test status.

This analysis lets us describe the structure and ideal functionality of a
contact tracing protocol. The protocol's interactions should fit into the
following phases:

- **Broadcast**: users generate and broadcast Contact Event Numbers (CENs) over
  Bluetooth to nearby devices.
- **Report**: a user uploads a packet of data to a server to send a report to
  all users they may have encountered in some time interval.
- **Scan**: users monitor data published by the server to learn whether they
  have received any reports.

Ideally, the protocol should have the following properties:

- **Server Privacy**: An honest-but-curious server should not learn information
  about any user's location or contacts.
- **Source Integrity**: Users cannot send reports to users they did not come
  in contact with or on behalf of other users.
- **Broadcast Integrity**: Users cannot broadcast CENs they did not generate.
- **No Passive Tracking**: A passive adversary monitoring Bluetooth connections
  should not be able to learn any information about the location of users who
  do not send reports.
- **Receiver Privacy**: Users who receive reports do not reveal information to
  anyone.
- **Reporter Privacy**: Users who send reports do not reveal information
  to users they did not come in contact with, and reveal only the time of
  contact to users they did come in contact with.  Note that in practice, the
  timing alone may still be sufficient for their contact to learn their
  identity (e.g., if their contact was only around one other person at the
  time).

Of these properties, broadcast integrity is very difficult to achieve,
because it requires authentication at the physical layer to prevent a user
from rebroadcasting CENs they observed from other users. However, the attack
it prevents is one where an adversary creates ghostly copies of legitimate
users, and this attack requires the adversary to go around with devices, so
it does not scale well. In what follows, we do not attempt to achieve
broadcast integrity.

## A strawman protocol

As a first attempt to formulate a protocol that satisfies these properties,
we consider a strawman protocol. All mobile devices running the app
periodically generate a random CEN, store the CEN, and broadcast it using
Bluetooth. At the same time, the app also listens for and records the CENs
generated by other devices. To send a report, the user (or a health authority
acting on their behalf) uploads the CENs she generated to a server, together
with a memo field containing application-specific report data. All users'
apps periodically download the list of reported CENs, then compare it with
the list of CENs they observed and recorded locally. The intersection of
these two lists is the set of positive contacts.

Intuitively, this provides server privacy, as the server only observes a list
of random numbers, and cannot correlate them with users or locations without
colluding with other users.  It prevents passive tracking, because all
identifiers are randomly generated and therefore unlinkable from each other.
It provides receiver privacy, because all users download the same list of
reported CENs and process it locally.  And if the list of CENs is batched
appropriately, users who send reports do not leak information beyond the time
of contact to users who observed the CENs.

However, this proposal does not provide source integrity. Because CENs have
no structure, nothing prevents a user from observing the CENs broadcast by
another user and then including them in a report to the server. Notice that
this is still a problem even in the setting where a health authority verifies
reports, because although they can attest to test results, they have no way
to verify the CENs. It also poses scalability issues, because the report
contains a list of every CEN the user broadcast over the reporting period,
and all users must download all reports.

## The CEN Protocol

To address the scalability issue, we change from purely random CENs to CENs
deterministically generated from some seed data. This reduces the size of the
report, because it can contain only the compact seed data rather than the
entire list of CENs. This change trades scalability for reporter privacy,
because CENs derived from the same report are linkable to each other.
However, this linkage is only possible by parties that have observed multiple
CENs from the same report, not by all users. Distinct reports are not
linkable, so users can submit multiple partial reports rather than a single
report for their entire history. The report rotation frequency adjusts the
tradeoff between reporter privacy and scalability.

To address the source integrity issue, we additionally bind the derived CENs
to a secret held by the user, and require that they prove knowledge of that
secret when submitting a report. This proof (in the form of a digital
signature) can be relayed to other users for public verifiability, or checked
only by the server.

### Key Derivation.

**Report Key Generation**. The user-agent creates the *report authorization
key* `rak` and the *report verification key* `rvk` as the signing and
verification keys of a signature scheme.
Then it computes the initial *contact event key (CEK)* as
```
cek_0 ← H_cek(rak).
```
Each report can contain at most `2**16` CENs. `H_cek` is a domain-separated
hash function with 256 bits of output.

**CEK Ratchet**. Contact event keys support a *ratchet* operation:
```
cek_i ← H_cek(rvk || cek_{i-1}).
```
where `||` denotes concatenation. As noted below, it is crucial that CEK
ratchet is  synchronized with MAC rotation at the Bluetooth layer to prevent
linkability attacks.

**CEN Generation**. A contact event number is derived from a contact event 
key by computing
```
cen_i ← H_cen(le_u16(i) || cek_i),
```
where `H_cen` is a domain-separated hash function with 128 bits of output.

**Diagram**.  The key derivation process is illustrated in the following
diagram:
```
             ┌───┐
  ┌─────────▶│rvk│────────┬──────────┬──────────┬──────┬──────────┐
  │          └───┘        │          │          │      │          │
  │                       │          │          │      │          │
┌───┐             ┌─────┐ │  ┌─────┐ │  ┌─────┐ │      │  ┌─────┐ │
│rak│────────────▶│cek_0│─┴─▶│cek_1│─┴─▶│cek_2│─┴─▶...─┴─▶│cek_n│─┴─▶...
└───┘             └─────┘    └─────┘    └─────┘           └─────┘
                     │          │          │                 │
                     ▼          ▼          ▼                 ▼
                  ┌─────┐    ┌─────┐    ┌─────┐           ┌─────┐
                  │cen_0│    │cen_1│    │cen_2│           │cen_n│
                  └─────┘    └─────┘    └─────┘           └─────┘
```
Notice that knowledge of `rvk` and `cek_i` is sufficent to recover
all subsequent `cek_j`, and hence all subsequent `cen_j`.

### Reporting.

A user wishing to notify contacts they encountered over the period `j1` to `j2`
prepares a report as
```
report ← rvk || cek_{j1} || le_u16(j1) || le_u16(j2) || memo
```
where `memo` is a variable-length bytestring 2-257 bytes long whose structure
is described below. Then they use `rak` to produce `sig`, a signature over
`report`, and send `report || sig` to the server.

**Report Check**.
Anyone can verify the source integity of the report by checking `sig` over
`report` using the included `rvk`, recompute the CENs as
```
cen_j1 ← H_cen(le_u16(j1) || cek_{j1})
cek_{j1+1} ← H_cek(rvk || cek_{j1})
cen_{j1+1} ← H_cen(le_u16(j1+1) || cek_{j1+1})
...
```
and compare the recomputed CENs with their observations. The server can
optionally strip the trailing 64 byte `sig` from each report if client
verification is not important.

**Memo Structure**.
The memo field provides a compact space for freeform messages. This ensures
that the protocol is application-agnostic and extensible. For instance, the
memo field could contain a bitflag describing self-reported symptoms, in the
case of [CoEpi], or a signature verifying test results, in the case of
[CovidWatch].

The memo field is between 2 and 257 bytes and has the following
tag-length-value structure:
```
type: u8 || len: u8 || data: [u8; len]
```
The `data` field contains 0-255 bytes of data whose type is
encoded by the `type` field, which has the following meaning:
- `0x0`: CoEpi symptom report v1;
- `0x1`: CovidWatch test result v1;
- `0x2-0xfe`: reserved for allocations to applications on request;
- `0xff`: reserved (can be used to add more than 256 types later).

**Parameter Choices**. We implement 
* `H_cek` using SHA256 with domain separator `b"H_CEK"`;
* `H_cen` using SHA256 with domain separator `b"H_CEN"`;
* `rak` and `rvk` as the signing and verification keys of Ed25519.

These parameter choices result in signed reports of 134-389 bytes or unsigned
reports of 70-325 bytes, depending on the length of the memo field.

## CEN sharing with Bluetooth Low Energy

Applications following this protocol use iOS and Android apps' capability to share a 128-bit Temporary Contact Number (TCN) with nearby apps using Bluetooth Low Energy (BLE).

Sharing TCNs using BLE should work:
- cross-platform between iOS and Android apps.
- cross-app.
- without asking the user to access their location.
- power-efficiently, with the least amount of BLE traffic.
- between apps while they both are in the background with the devices' screen locked.

With the above requirements, we encountered the following BLE platform limitations:
- iOS 13.4 (and older) does not support the discoverability between third-party iOS apps in the suspended or background-running state, and with the devices' screens locked. Note: If the user unlocks the screen or launches an app (e.g., Settings.app), which does active Bluetooth scanning, then yes.
- iOS 13.4 (and older) does not support the broadcasting of small advertisement data of third-party apps, while Android supports up to 31 bytes.

To overcome the above limitations, the protocol uses both broadcast-oriented and connection-oriented BLE modes to share TCNs. The terminology used for BLE devices in these modes are:
- Broadcaster and observer in broadcast-oriented mode.
- Peripheral and central in connection-oriented mode.

In both modes, the protocol uses the `0xC019` 16-bit UUID for the service identifier.

In broadcast-oriented mode, a broadcaster advertises a 16-byte TCN using the service data field (`0x16` GAP) of the advertisement data. The observer reads the TCN from this field.

In connection-oriented mode, the peripheral adds a primary service whose UUID is `0xC019` to the GATT database and advertises it. The service exposes a readable and writeable characteristic whose UUID is `D61F4F27-3D6B-4B04-9E46-C9D2EA617F62` for sharing TCNs. After sharing a TCN, the centrals disconnect from the peripherals.

### How the Temporary Contact Number (TCN) is Found

Android-Android, iOS-Android: 1 observes 2’s broadcast and reads the value from the advertisement data.

Android-iOS: 2 connects to 1 and writes the value of the characteristic and disconnects.

iOS(F)-iOS(B), iOS(B)-iOS(F): 1 connects to 2 and reads the value of the characteristic and disconnects.

iOS(B)-iOS(B): Nearby Android device acts as a bridge: It adds the TCNs received through the characteristic write requests and will advertise those also, while in range.

F = App in Foreground

B = App in Background

Current open-source implementations from CoEpi + Covid Watch generating TCNs locally and covering the communication for each of the above key flows are being developed in the following repositories:

* https://github.com/Co-Epi/app-android
* https://github.com/Co-Epi/app-ios
* https://github.com/covid19risk/covidwatch-ios
* https://github.com/covid19risk/covidwatch-android
* [if you have a repository, please file a PR to add it here]

It is expected that the process to generate the 128-bit TCN will not vary
between different platforms, but the process of communicating TCNs between
platforms has been reduced to working implementations in the above
repositories.

## References and Further Reading

- [Shared research document tracking privacy-preserving contact tracing mechanisms](https://docs.google.com/document/d/16Kh4_Q_tmyRh0-v452wiul9oQAiTRj8AdZ5vcOJum9Y/edit#).

## Contributors

- Sourabh Niyogi <sourabh@wolk.com>,
- James Petrie,
- Scott Leibrand,
- Jack Gallagher,
- Hamish,
- Manu Eder <manulari@posteo.eu>,
- Zsombor Szabo, <zsombor@gmail.com>
- George Danezis (UCL),
- Ian Miers,
- Henry de Valence <hdevalence@hdevalence.ca>,
- Daniel Reusche,

[CoEpi]: https://www.coepi.org/
[CovidWatch]: https://www.covid-watch.org/

# Notes (to be merged with main document)

## Key rotation and compression factor

One important question is how frequently do we change the key. If it does not
change, then uploading the key on a positive test reveals all contacts a user
has ever had, even several months ago. On the other extreme, we could change
the key every time we generate a CEN, then we are back to the strawman random
CEN and the resulting scalability problems. What is an appropriate middle
ground?

## Rotation considerations

The key rotation interval must balance a trade off between security and
scalability. Consider, for example, rotating keys every day. This should result
in reasonable amounts of data being uploaded and downloaded. However, it means
any user who tested positive would associate all the CENs they broadcast in a
given day together by revealing the key that generated them. This has two
consequences: 1) other users could “compare notes” and see if they saw CENs
generated by the same key and therefore encountered the same person. 2) The
user could easily be tracked during that day by anyone who passively listens
for CENs and notes their locations. 

Discussions around the first concern concluded it was the less problematic of
the two attacks. To actively mount the attack would require users to actively
find each other, collude, and compare data. And the end result is being able to
infer they had contact with the same person. It is worth noting that this may
be possible for humans to do on their own in many cases, simply by comparing
who they have talked to, etc or looking at what time an encounter happened and
remembering where they were and who they were meeting. It is also likely that
the CoEpi app will allow users to locally store location history data to assist
with identifying where a contact occurred, and therefore how likely it was to
have represented a possible exposure. Since identifying such exposures is the
whole point of the app, recipients can be expected (by the users reporting
symptoms or test results) to receive and use such information for whatever
purposes they deem appropriate. Any inappropriate use of such information will
need to be avoided by social, not technological, means. 

However, the second issue is a major concern. BLE beacon tracking is already
used in some settings. Moreover, if contact tracing apps become ubiquitous,
enterprise solutions for tracking contacts for businesses and public spaces
will emerge rapidly. This will result in CENs being recorded in bulk and likely
aggregated in cloud managed services. In this setting then, linking 24 hours of
CENs together will be easy and equivalent to simply revealing a user’s location
history for that day (if they later report symptoms or a positive test). Worse,
due to the relative simplicity of re-identification attacks, it should be
fairly simple to link each 24 hour snippet of a user’s location history
together to compute a history over a week or more. Substantially shortening the
interval reduces this risk. It does not completely eliminate it, but as a
primary defense against BLE beacon tracking, re-keying intervals should be as
short as feasible. 

Note, however, re-keying is much less of a concern if contacts are made based
on symptom reports. If users are notified of the contact and their symptoms,
and the symptom descriptors are reasonably unique, then with high probability
all CENs which have a report containing the same symptoms are from the same
user. This is the same information that would be leaked by using a long
rekeying interval.

## Further considerations

In the setting where CENs are continuously broadcast, we must also choose the
rate at which we change from one CEN to another.  Again, the longer a CEN lasts
for, the greater the risk of tracking. In particular, in many settings it will
be easy to infer at the time that one CEN disappears and another appears that
they are the same device. This won’t be perfect, but if CENs change
infrequently, it need not be perfect to recover a pretty good trace of a user's
location history.

Finally, Bluetooth itself exposes a number of tracking opportunities due to the
handling of MAC addresses and other identifiers. Unfortunately, the degree to
which these are properly randomized varies considerably across devices, with
many devices not implementing strong privacy protections. See 
[this paper](https://arxiv.org/pdf/2003.11511.pdf) for an
overview on privacy issues. In all cases, the duration for which a CEN lasts
should be a multiple of the frequency with which MAC address and other
identifiers in the BLE protocol get randomized. For example, if the MAC address
changes every minute, then the CEN can change every minute, every 10 seconds,
or every second. But it cannot change every two minutes or change e.g. every 7
seconds. In the latter two cases, then when the MAC address changed, the CEN
would not. Anyone observing (MAC A, CEN 1), then (MAC B, CEN 1), then (MAC B,
CEN 2) can conclude they are all the same device because all identifiers don’t
change at the same time. This would entirely compromise Bluetooth privacy. 

## Attacks

### Linkage Attack
A [linkage attack](https://www.cis.upenn.edu/~aaroth/Papers/privacybook.pdf)
is the matching of anonymized records with non-anonymized records in a different
dataset. An example for our usecase would be: A user is only close to one other
person in a given timeframe. If they get notified of a revealed contact, they
know who it was. Generally: If the timeframe of a contact is revealed, and users
do out of band correlation, like taking notes/pictures, they can narrow down the
possible real identies of their contacts, which revealed. As long as the users
know which CENs are in the intersection, this can not be prevented.

### Replay Attack
An attacker collects CENs of others and rebroadcasts them, to impersonate another
user during the gossip phase. If not mitigated, they can at most produce as many
false positives as they could with an illegitimate reveal (i.e. they are not
infected). Since in the above proposal, the validity period of a CEN will be known 
after a reveal, this attack can only be executed in a short timeframe.

## Counting CEN collisions
With 128 Bit CENs, at a world population of 8bn, expected total collision count for 
all legitimately generated CENs from a two week timeframe (revealed and non-revealed), 
without sharding, is ~1.7e-13 (see [`collisions.jl`](./scripts/collisions.jl)).
