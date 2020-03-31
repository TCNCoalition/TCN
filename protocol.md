# CEN Protocol

This is a work-in-progress document.  So far, it just has copies of text from
various Google Docs, Slack threads, etc., to provide a single source of truth
to iterate the specification.

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
