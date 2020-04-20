# TCN Protocol

> This is a work-in-progress document. Changes are tracked through PRs
> and issues.

This document describes **Temporary Contact Numbers**, a decentralized,
privacy-first contact tracing protocol developed by the [TCN
Coalition][tcn-coalition].  This protocol is built to be extensible,
with the goal of providing interoperability between contact tracing
applications.  The TCN protocol and related efforts are designed with
the [Contact Tracing Bill of Rights](/ContactTracingBillOfRights.md) in
mind.

No personally-identifiable information is required by the
protocol, and although it is compatible with a trusted health authority, it
does not require one. Users' devices send short-range broadcasts over
Bluetooth to nearby devices. Later, a user who develops symptoms or tests
positive can report their status to their contacts with minimal loss of
privacy. Users who do not send reports reveal no information. Different
applications using the TCN protocol can interoperate, and the protocol can be
used with either verified test results or for self-reported symptoms via an
extensible report memo field.

PRs and Issues are welcome to be submitted directly to this repo. For questions
about the TCN Protocol or collaborating in more detail, contact
[Henry](mailto:hdevalence@hdevalence.ca?Subject=TCN%20Protocol) or
[Dana](mailto:dana+CoEpi@OpenAPS.org?Subject=TCN%20Protocol).

This repository also contains a reference implementation of the TCN protocol
written in Rust. View documentation by running `cargo doc --no-deps --open`,
and run tests by running `cargo test`.

To coordinate development, the protocol is versioned using [Semver].
Changes can be found in [`CHANGELOG.md`](./CHANGELOG.md).

**What's on this page:**

- [Introduction](#tcn-protocol)
- [Ideal functionality and trust assumptions in contact tracing systems](#ideal-functionality-and-trust-assumptions-in-contact-tracing-systems)
- [A strawman protocol](#a-strawman-protocol)
- [The TCN Protocol](#the-tcn-protocol)
- [TCN sharing with Bluetooth Low Energy](#tcn-sharing-with-bluetooth-low-energy)
- [Contributors](#contributors)

As it is a work-in-progress, this page also contains [rough notes, yet to be
merged with the main document](#notes-to-be-merged-with-main-document).

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

Moreover, a protocol that places additional burdens on health
authorities (e.g., requiring them to deploy complex cryptography like
MPC or carefully manage cryptographic key material) faces severe
adoption barriers compared to one that does not, so reducing trust
requirements may allow accelerated deployment.

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

- **Broadcast**: users generate and broadcast Temporary Contact Numbers
  (TCNs) over Bluetooth to nearby devices.
- **Report**: a user uploads a packet of data to a server to send a report to
  all users they may have encountered in some time interval.
- **Scan**: users monitor data published by the server to learn whether they
  have received any reports.

Ideally, the protocol should have the following properties:

- **Server Privacy**: An honest-but-curious server should not learn information
  about any user's location or contacts.
- **Source Integrity**: Users cannot send reports to users they did not come
  in contact with or on behalf of other users.
- **Broadcast Integrity**: Users cannot broadcast TCNs they did not generate.
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
from rebroadcasting TCNs they observed from other users. However, the attack
it prevents is one where an adversary creates ghostly copies of legitimate
users, and this attack requires the adversary to go around with devices, so
it does not scale well. In what follows, we do not attempt to achieve
broadcast integrity.

## A strawman protocol

As a first attempt to formulate a protocol that satisfies these properties,
we consider a strawman protocol. All mobile devices running the app
periodically generate a random TCN, store the TCN, and broadcast it using
Bluetooth. At the same time, the app also listens for and records the TCNs
generated by other devices. To send a report, the user (or a health authority
acting on their behalf) uploads the TCNs she generated to a server, together
with a memo field containing application-specific report data. All users'
apps periodically download the list of reported TCNs, then compare it with
the list of TCNs they observed and recorded locally. The intersection of
these two lists is the set of positive contacts.

Intuitively, this provides server privacy, as the server only observes a list
of random numbers, and cannot correlate them with users or locations without
colluding with other users.  It prevents passive tracking, because all
identifiers are randomly generated and therefore unlinkable from each other.
It provides receiver privacy, because all users download the same list of
reported TCNs and process it locally.  And if the list of TCNs is batched
appropriately, users who send reports do not leak information beyond the time
of contact to users who observed the TCNs.

However, this proposal does not provide source integrity. Because TCNs have
no structure, nothing prevents a user from observing the TCNs broadcast by
another user and then including them in a report to the server. Notice that
this is still a problem even in the setting where a health authority verifies
reports, because although they can attest to test results, they have no way
to verify the TCNs. It also poses scalability issues, because the report
contains a list of every TCN the user broadcast over the reporting period,
and all users must download all reports.

## The TCN Protocol

To address the scalability issue, we change from purely random TCNs to TCNs
deterministically generated from some seed data. This reduces the size of the
report, because it can contain only the compact seed data rather than the
entire list of TCNs. This change trades scalability for reporter privacy,
because TCNs derived from the same report are linkable to each other.
However, this linkage is only possible by parties that have observed multiple
TCNs from the same report, not by all users. Distinct reports are not
linkable, so users can submit multiple partial reports rather than a single
report for their entire history. The report rotation frequency adjusts the
tradeoff between reporter privacy and scalability.

To address the source integrity issue, we additionally bind the derived TCNs
to a secret held by the user, and require that they prove knowledge of that
secret when submitting a report. This proof (in the form of a digital
signature) can be relayed to other users for public verifiability, or checked
only by the server.

### Key Derivation.

**Report Key Generation**. The user-agent creates the *report authorization
key* `rak` and the *report verification key* `rvk` as the signing and
verification keys of a signature scheme.  These keys will be periodically
rotated, as described below.

Then it computes the initial *temporary contact key (TCK)* `tck_1` as
```
tck_0 ← H_tck(rak)
tck_1 ← H_tck(rvk || tck_0)
```
Each report can contain at most `2**16` TCNs. `H_tck` is a domain-separated
hash function with 256 bits of output.

**TCK Ratchet**. Contact event keys support a *ratchet* operation:
```
tck_i ← H_tck(rvk || tck_{i-1}),
```
where `||` denotes concatenation.

**TCN Generation**. A temporary contact number is derived from a
temporary contact key by computing
```
tcn_i ← H_tcn(le_u16(i) || tck_i),
```
where `H_tcn` is a domain-separated hash function with 128 bits of output.

As noted below, it is important that changing of TCNs and therefore the TCK
ratchet is synchronized with MAC rotation at the Bluetooth layer as much as
possible to make local linkability attacks as hard as possible.


**Diagram**.  The key derivation process is illustrated in the following
diagram:
```
      ┌───┐
  ┌──▶│rvk│─────────┬──────────┬──────────┬──────────┬──────────┐
  │   └───┘         │          │          │          │          │
┌───┐       ┌─────┐ │  ┌─────┐ │  ┌─────┐ │          │  ┌─────┐ │
│rak│──────▶│tck_0│─┴─▶│tck_1│─┴─▶│tck_2│─┴─▶  ...  ─┴─▶│tck_n│─┴─▶...
└───┘       └─────┘    └─────┘    └─────┘               └─────┘
                          │          │                     │
                          ▼          ▼                     ▼
                       ┌─────┐    ┌─────┐               ┌─────┐
                       │tcn_1│    │tcn_2│      ...      │tcn_n│
                       └─────┘    └─────┘               └─────┘
```
Notice that knowledge of `rvk` and `tck_i` is sufficent to recover
all subsequent `tck_j`, and hence all subsequent `tcn_j`.

### Reporting.

A user wishing to notify contacts they encountered over the period `j1 >
0` to `j2` prepares a report as
```
report ← rvk || tck_{j1-1} || le_u16(j1) || le_u16(j2) || memo
```
where `memo` is a variable-length bytestring 2-257 bytes long whose structure
is described below. Then they use `rak` to produce `sig`, a signature over
`report`, and send `report || sig` to the server.

**Report Check**.
Anyone can verify the source integity of the report by checking `sig` over
`report` using the included `rvk`, recompute the TCNs as
```
tck_j1 ← H_tck(rvk || tck_{j1-1})              # Ratchet
tcn_j1 ← H_tcn(le_u16(j1) || tck_{j1})         # Generate
tck_{j1+1} ← H_tck(rvk || tck_{j1})            # Ratchet
tcn_{j1+1} ← H_tcn(le_u16(j1+1) || tck_{j1+1}) # Generate
...
```
and compare the recomputed TCNs with their observations.  Note that the
TCN derived from the provided `tck_{j1-1}` is *not* included in the
report, because the recipient cannot verify that it is bound to `rvk`.
The server can optionally strip the trailing 64 byte `sig` from each
report if client verification is not important.

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
- `0x2`: ito report v1;
- `0x3-0xfe`: reserved for allocations to applications on request;
- `0xff`: reserved (can be used to add more than 256 types later).

**Parameter Choices**. We implement 
* `H_tck` using SHA256 with domain separator `b"H_TCK"`;
* `H_tcn` using SHA256 with domain separator `b"H_TCN"`;
* `rak` and `rvk` as the signing and verification keys of Ed25519.

These parameter choices result in signed reports of 134-389 bytes or unsigned
reports of 70-325 bytes, depending on the length of the memo field.

**Test vectors** can be generated via
```
cargo test generate_test_vectors -- --nocapture
```

### Report Timespans and Key Rotation

Because a report allows other users to regenerate a set of TCNs, those TCNs
become linkable after a report is published, as they are all associated to the
same report.  This means that the report authorization key should be
periodically rotated, breaking a user's TCN history into chunks which are
unlinkable from each other.

If the report authorization key is never rotated, publication of a report could
allow users or passive adversaries to learn a user's location history (by
noting multiple observations of TCNs associated to the same report).  On the
other hand, if the report authorization key is rotated very frequently (e.g.,
once per TCN), then we are back to the strawman random TCN and the resulting
scalability problems.  The report rotation interval parameter thus must balance
privacy and scalability.

Two examples of re-identification attacks at different levels of sophistication
are, on the one hand, users of a tracing application comparing their
observations, and on the other, passive adversaries tracking Bluetooth
broadcasts at scale.

The first case is less serious, because it requires active coordination between
users, and the end result, being able to infer information about a reporter,
may be possible anyways, simply by comparing who they have talked to, etc or
looking at what time an encounter happened and remembering where they were and
who they were meeting.  Applications may record information like location
history to help users recall context and assess their risk.  Since identifying
such exposures is the whole point of the app, recipients can be expected to
receive and use contextual information for whatever purposes they deem
appropriate. Any inappropriate use of such information will need to be avoided
by social, not technological, means. 

However, the second case is a major concern.  Bluetooth tracking is [already
deployed][bluetooth-garbage] by advertising companies seeking to sell
information about people's daily routines.  These systems could be easily
repurposed to record TCNs in attempt to track reporters.  Substantially
shortening the report timespan reduces this risk, but cannot entirely eliminate
it.  In particular, if the report timespan is still long enough to cover a
user's routines, linking disjoint reports may be possible.  For instance, if
the report timespan is a single day, it may be possible for a passive adversary
to link reports by matching daily routines.

Both of these cases underscore the importance of informed consent by users who
submit reports.  **To reduce the risk of matching daily routines, we suggest a
report timespan of 6 hours or less.**

Note, however, that this discussion focuses on the case where the report
contents have little identifying content, for instance the fact of a positive
test result.  If users opt in to self-reporting symptom data, the symptom
bitflag may be sufficient to link disjoint reports, so users are already opting
to reveal the information this mechanism aims to protect.

[bluetooth-garbage]: https://gizmodo.com/brave-new-garbage-londons-trash-cans-track-you-using-1071610114

## TCN sharing with Bluetooth Low Energy

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

| Listener OS     | Sender OS         | TCN Communication |
| --------------- | ----------------- | --- | 
| Android         | Android           | The Sender broadcasts TCN over Bluetooth. The Listener observes this broadcast directly. |
| iOS             | Android           | Same as row above. |
| Android         | iOS               | The Listener signals availability as a Bluetooth peripheral. The Sender, acting as a Bluetooth central, connects to this peripheral and writes its TCN to a field exposed by the peripheral then disconnects. |
| iOS (background) | iOS (foreground) | The Listener acts as a Bluetooth central. It connects to the Sender, which acts as a peripheral, reads the Sender's exposed TCN field, then disconnects. |
| iOS (foreground) | iOS (background) | Same as the row above. |
| iOS (background) | iOS (background) | A nearby Android device acts as a bridge. It receives TCNs through “central” write operations (see 3rd row above) and adds them to a rotating list to broadcast alongside its own TCN. |


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
[tcn-coalition]: https://tcn-coalition.org/
[Semver]: https://semver.org/

# Notes (to be merged with main document)

## Further considerations

In the setting where TCNs are continuously broadcast, we must also choose the
rate at which we change from one TCN to another.  Again, the longer a TCN lasts
for, the greater the risk of tracking. In particular, in many settings it will
be easy to infer at the time that one TCN disappears and another appears that
they are the same device. This won’t be perfect, but if TCNs change
infrequently, it need not be perfect to recover a pretty good trace of a user's
location history.

Finally, Bluetooth itself exposes a number of tracking opportunities due to the
handling of MAC addresses and other identifiers. Unfortunately, the degree to
which these are properly randomized varies considerably across devices, with
many devices not implementing strong privacy protections. See 
[this paper](https://petsymposium.org/2019/files/papers/issue3/popets-2019-0036.pdf)
for an overview on privacy
issues.
To avoid making the situation worse, ideally every MAC address change should be
accompanied by a simultaneous change of the TCN. If this is not done, then
anyone observing (MAC A, TCN 1), then (MAC B, TCN 1), then (MAC B, TCN 2) can
conclude they are all the same device because all identifiers don’t change at
the same time. This makes devices running the TCN protocol more easily trackable
in a confined area for anyone who can continuously observe their Bluetooth
signals.
The extent to which such synchronization is possible is limited by the Bluetooth
APIs exposed by operating systems. On iOS we know of no way to be notified or
influence rotation of the Bluetooth MAC address. On Android, experiments show
that restarting Bluetooth advertising causes a new random MAC address to be
chosen by the operating system, so instead of reacting to MAC address changes we
can cause them to happen at the same time as TCN changes.
(Note that even if TCN changes happen simultaneously with MAC address changes,
unless rotation of MAC addresses and TCNs is globally synchronized among all
devices, an adversary who has Bluetooth observations with very fine time
resolution may still be able to link distinct MAC adresses simply because
appearance of a new MAC address for a device will closely follow disappearance
of the old one. This has very little to do with the TCN protocol and is simply a
consequence of having Bluetooth turned on.)

## Attacks

### Linkage Attack
A [linkage attack](https://www.cis.upenn.edu/~aaroth/Papers/privacybook.pdf)
is the matching of anonymized records with non-anonymized records in a different
dataset. An example for our usecase would be: A user is only close to one other
person in a given timeframe. If they get notified of a revealed contact, they
know who it was. Generally: If the timeframe of a contact is revealed, and users
do out of band correlation, like taking notes/pictures, they can narrow down the
possible real identies of their contacts, which revealed. As long as the users
know which TCNs are in the intersection, this can not be prevented.

### Replay Attack
An attacker collects TCNs of others and rebroadcasts them, to impersonate another
user during the gossip phase. If not mitigated, they can at most produce as many
false positives as they could with an illegitimate reveal (i.e. they are not
infected). Since in the above proposal, the validity period of a TCN will be known 
after a reveal, this attack can only be executed in a short timeframe.

### Address Carryover Attack
An [address-carryover](https://petsymposium.org/2019/files/papers/issue3/popets-2019-0036.pdf)
is possible when the rotation periods of Bluetooth MAC address and TCN are not
aligned, as in this figure:

```
|-------|-------|-------|-------|-------|  BT MAC rotation
|----|----|----|----|----|----|----|----|  TCN rotation
```

The attacker could then use the overlap to link multiple identifiers to the
same source. To mitigate this attack, TCN rotation needs to be aligned with the
platform specific rotation of lower level identifiers. TCN rotation frequency can
be higher than that of other identifiers, but any overlap has to be avoided.

### Shard Carryover Attack
If a space-time based sharding scheme is used, an attack similar to the
address carryover attack needs to be mitigated. When switching shards,
a new keypair should be generated. Otherwise, multiple shards could be linked
to a single source upon reveal. Simply rotating TCNs is not sufficient here.

## Counting CEN collisions
With 128 Bit CENs, at a world population of 8bn, expected total collision count for 
all legitimately generated CENs from a two week timeframe (revealed and non-revealed), 
without sharding, is ~1.7e-13 (see [`collisions.jl`](./scripts/collisions.jl)).
