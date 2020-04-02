# CEN Protocol

> This is a work-in-progress document.  So far, it just has copies of text from
> various Google Docs, Slack threads, etc., to provide a single source of truth
> to iterate the specification.

**XXX** Add a paragraph to the beginning with a BLUF summary.

Contact tracing is used to identify people who may have been exposed to
infection and notify them of their exposure, allowing isolation, testing, or
treatment as may be appropriate.  However, contact tracing poses risks of its
own, such as fear of stigma or discrimination based on health status, or the
risk that contact tracing systems could be repurposed for surveillance by
governments or individuals.

This document describes a protocol for mobile devices that aims to support
contact tracing with minimal risk and without requiring trust in a centralized
party.  No personally-identifiable information is required by the protocol.
Users' devices send short-range broadcasts over Bluetooth to nearby devices.
Later, a user who develops symptoms or tests positive can report their status
to their contacts with minimal loss of privacy.  Users who do not send reports
reveal no information.

**XXX** Fill in the rest of this introduction with an overview of the document's contents

## Trust assumptions in contact tracing systems

**XXX** This should exist in some form but I'm not sure where.

1.  **Location Privacy**.  Is any party trusted with user's location data, and
    if so, under what circumstances?

2.  **Functional Capacity**.  Does the system trust that health authorities
    will be able to carry out their functions, or is it resilient in case they
    become overwhelmed and are unable to?

3.  **Report Integrity**.  What measures does the system use, if any, to
    determine the integrity of a report of symptoms or test status?

The CEN protocol does not require trust related to location data, and it also
does not require participation by a health authority.  Any contact tracing
protocol that does not require participation by a health authority is
effectively a particular kind of anonymous messaging protocol, which allows
users to send reports to all users whom they may have come in contact with
without revealing their identity.  Leaving the question of report integrity as
an application-level concern means that different applications can make
different choices, while still remaining interoperable.  For instance, CoEpi
allows users to self-report symptoms, while CovidWatch trusts a health
authority to attest to the integrity of a positive test status.

**XXX** Insert comparisons with other protocols.

## Ideal functionality for contact tracing protocols

The protocol's interactions should fit into the following phases:

- **Broadcast**: users generate and broadcast Contact Event Numbers (CENs) over
  Bluetooth to nearby devices.
- **Report**: a user uploads a packet of data to a server to send a report to
  all users they may have encountered in some time interval.
- **Scan**: users monitor data published by the server to learn whether they
  have received any reports.
- **Fetch**: users who learn of a report addressed to them can download it.
  - **XXX** Adding this as a separate step would make the protocol much more
    extensible.  The alternative would be to include the message in the data
    published by the server, but this maybe requires all clients to be able to
    parse messages and if the message contents are longer than a message ID,
    it's less bandwidth efficient.  For the current CoEpi design, this is
    probably not important, but it will probably not be possible to change the
    protocol if it is widely deployed, so this may be the only chance to add
    extensibility.

The protocol should have the following properties:

- **Server Privacy**: An honest-but-curious server should not learn information
  about any user's location or contacts.
- **Source Integrity**: Users cannot send reports to users they did not come
  in contact with or on behalf of other users.
- **No Passive Tracking**: A passive adversary monitoring Bluetooth connections
  should not be able to learn any information about the location of users who
  do not send reports.
- **Receiver Privacy**: Users who receive reports do not reveal information to
  anyone.
- **Weak Reporter Privacy**: Users who send reports do not reveal information
  to users they did not come in contact with, and reveal only the time of
  contact to users they did come in contact with.  Note that in practice, the
  timing alone may still be sufficient for their contact to learn their
  identity (e.g., if their contact was only around one other person at the
  time).

**XXX** It may be possible / better to merge this with the section above.

## A strawman protocol

As a first attempt to formulate a protocol that satisfies these properties, we
consider a strawman protocol.  All mobile devices running the app periodically
generate a random CEN, store the CEN, and broadcast it using Bluetooth. At the
same time, the app also listens for and records the CENs generated by other
devices.  To send a report, the user (or a health authority acting on their
behalf) uploads the CENs she generated to a server.  All users' apps
periodically download the list of reported CENs, then compare it with the list
of CENs they observed and recorded locally. The intersection of these two lists
is the set of positive contacts.

Intuitively, this provides server privacy, as the server only observes a list
of random numbers, and cannot correlate them with users or locations without
colluding with other users.  It prevents passive tracking, because all
identifiers are randomly generated and therefore unlinkable from each other.
It provides receiver privacy, because all users download the same list of
reported CENs and process it locally.  And if the list of CENs is batched
appropriately, users who send reports do not leak information beyond the time
of contact to users who observed the CENs.

However, this proposal does not provide source integrity.  Because CENs have no
structure, nothing prevents a user from observing the CENs broadcast by another
user and then including them in a report to the server.  Notice that this is
still a problem even in the setting where a health authority verifies reports,
because although they can attest to test results, they have no way to verify
the CENs.  It also poses scalability issues, because all users must download
the entire list of reported CENs.

Both of these issues can be addressed by deriving CENs differently, as
described below.

## Compressing CEN uploads for scalability:

To ensure scalability, we upload a compressed representation of all the CENs a
user generates. To do this, we use cryptography to generate the entire sequence
of CENs for a user from a short key. As a result, a single 128 bit key can be
used to represent all CENs a user will ever generate. The exact method of
deriving keys is currently under discussion, but there are multiple feasible
approaches and it is simply a matter of picking one. Provided the CEN are
cryptographically pseudorandom, the particular properties of the generation
process should not affect security, but it may affect scalability.The existing
designs are detailed here and here.

## Key rotation and compression factor:

One important question is how frequently do we change the key. If it does not
change, then uploading the key on a positive test reveals all contacts a user
has ever had, even several months ago. On the other extreme, we could change
the key every time we generate a CEN, then we are back to the strawman random
CEN and the resulting scalability problems. What is an appropriate middle
ground?

## Rotation considerations:

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

## Further considerations.

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

## George Danezis proposal of 3/27/20

Builds on prior work from CoEpi (?)

The following CEN derivation function employs only a secure cryptographic hash function `H`:

### CEN Key Derivation

#### CEN Key Generation Initialization.  

The app would initialize the CEN generator with:
- `S`, a secret nonce, of at least 128 bits (optionally a public verification key `vk` from a signature scheme, with the corresponding signing key `sk` remaining always secret);
- `L = H(S)`, a short label;
- `K_0`, a fresh secret key for the session. (A session ends when a report of an infection is filled).

#### CEN Key Generation.  

For each period `i`, the key would be updated:
```
K_i ← H(K_{i-1}, L)
```

#### CEN Broadcast.  

For each period `i`, CENs would be derived as
```
CEN_i = H(K_i, period_i)
```
where `period_i` is an integer denoting the period for the `CEN_i`.

### Operations

#### Reveal

Upon a positive diagnosis, the app broadcasts to a health authority database:
the short label `L`, a key `K_{j-1}`, and the initial period `j` and number of
periods `j_max`. Other application users download `K_{j-1}`, `j`, and `j_max`,
and can compute all subsequent `K_j` and `CEN_j`s and compare them with the
ones seen on the phone.  The comparison here is purely string comparison, and
therefore efficient string search algorithms can be used on the phone. The
period the CEN was active can also be compared with the time a CEN was observed
to protect against replayed CENs from the past.  

#### Delete

After some days are past (with no symptoms), older
versions of the key before Ki can be deleted, making the link of the phone with
older CENs unrecoverable.

### Discussion

The advantage of the above over the proposals below is:

Given a new Ki the operation on the phone to match its associated CENs with
local observed CENs is efficient. There are 2x periods applications of a hash
function, and then only string matching – which is efficient through indexing. 

**Mitigating impersonation**: The key leverage you have is the trusted health
worker that supervises the positive test, and enables the contact tracing alert
(that broadcasts key `K_i` to derive CENs).  How can the protocol assist the
health worker (and its app) to ensure that reported CENs / `K_i`s are not
impersonations?

**Central admission control setting**: Since `L = H(S)`, where `S` is a secret
nonce, the app can prove to the healthcare worker that it generated the keys `K_i`
by revealing `S`.  However, if someone merely observes `K_i` and `L` (as part of the
contract tracing protocol), they cannot extract `S` and therefore cannot convince
any health workers to include the `K_i`/CENs they are merely relaying. This relies
on the honesty of health workers to check `L = H(S)` and not reveal the `S`
provided.

**Public verifiability**: If we set `S = vk` for `vk` for a `(vk, sk)` key pair, with
`sk` only known to the user reporting, then we can include a signature in the
report to prevent others impersonating the CENs and including them in their
report. In that scheme we provide a signature on `(S = vk, L, Kj-1, j-1, j_max)`
with the report. Others accept if the signature is valid under the included `vk`.
In this scheme the health authorities or anyone else can check the signed
record, and do not have to be trusted to mitigate impersonation.

## Modified proposal

Roughly, we replace `S` and `L` by the `sk` and `vk` of a signature scheme.

The app generates signing and verification keys `sk` (the *report authorization
key*) and `vk` of a signature scheme, and computes the initial CEN key as
```
k_0 ← H_k(sk).
```
CEN keys are updated by computing
```
k_i ← H_k(vk || k_{i-1}).
```
CENs are derived from the CEN key by computing
```
CEN_i ← H_CEN(le_u16(i) || k_i).
```
Note that each report authorization key can create at most `2**16` CENs.

A user wishing to notify contacts they encountered over the period `j1` to `j2`
prepares a report as
```
report ← vk || k_{j1} || le_u16(j1) || le_u16(j2) || memo
```
where `memo` is a variable-length bytestring with the following structure:
```
len: u8 || type: u8 || data: [u8; len]
```
Then they use `sk` to produce `sig`, a signature over `report`, and send
`report || sig` to the server.

Anyone can verify the source integity of the report by checking `sig` over
`report` using the included `vk`, and recompute the CENs as
```
CEN_j1 ← H_CEN(le_u16(j1) || k_{j1})
k_{j1+1} ← H_k(vk || k_{j1})
CEN_{j1+1} ← H_CEN(le_u16(j1+1) || k_{j1+1})
...
```
Using SHA256 and Ed25519, reports are `134-390` bytes each, depending on the
length of the memo field.  Servers can optionally strip the trailing 64 bytes
of each report if client verification is not important.

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


## CEN sharing with Bluetooth Low Energy

Applications following this protocol are based on iOS and Android applications
capability to broadcast a 128-bit Contact Event Number (CEN) using Bluetooth
Low Energy.  Current prototype implementations are using the following service
UUID and characteristic UUID:
```
service UUID = "BC908F39-52DB-416F-A97E-6EAC29F59CA8"
characteristic: UUID = "2ac35b0b-00b5-4af2-a50e-8412bcb94285"
```
Each device (Central) scans its environment and finds another device
(Peripheral) with the same service UUID, finding CEN in either the
Characteristic or ServiceData field.  Energy considerations may guide protocol
design so that scan frequencies are kept reasonable.  There are 4 key flows
between a Central device `C` that finds a peripheral device `P`:

| Central (C) | Peripheral (P) | How the Contact Event Number (CEN) Is Found |
|-------------|----------------|---------------------------------------------|
| iOS | iOS | C connects to P and retrieves the value of the characteristic |
| Android | Android | C reads P’s CEN from the ServiceData field of the Service Advertisement broadcast.  Frequency of changing the CEN: as often as the BLE stack and the OS will allow without getting mad at us (every 15mins, every min, every second?).  |
| iOS | Android | C connects to P and retrieves the value of the characteristic.  |
| Android | iOS | This combination does not appear!  For an Android device to receive a CEN from an iOS device, the iOS device connects as central and writes the value of a characteristic (not sure if it can be the same characteristic UUID or should be different).  |

Current open source implementations (MIT License) from CoEpi + COVID-Watch
generating CENs locally and covering the communication for each of the above 4
key flows are being developed in the following repositories:

* https://github.com/Co-Epi/app-android
* https://github.com/Co-Epi/app-ios
* https://github.com/covid19risk/covidwatch-ios
* https://github.com/covid19risk/covidwatch-android
* [if you have a repository, please file a PR to add it here]

It is expected that the process to generate the 128-bit CEN will not vary
between different platforms, but the process of communicating CENs between
platforms has been reduced to working implementations in the above
repositories.

## Contributors

- Sourabh Niyogi <sourabh@wolk.com>,
- James Petrie,
- Scott Leibrand,
- Jack Gallagher,
- Hamish,
- Manu Eder <manulari@posteo.eu>,
- Zsombor Szabo,
- George Danezis (UCL),
- Ian Miers,
- Henry de Valence <hdevalence@hdevalence.ca>,
