## Proposals

[Apple/Google](https://www.apple.com/covid19/contacttracing/), 
[DP-3T](https://github.com/DP-3T),
[PACT (MIT)](https://pact.mit.edu/), 
[PACT (UW)](https://covidsafe.cs.washington.edu/),
[TCN](https://github.com/TCNCoalition/TCN)

Neither the list of proposals, nor of the  building blocks claims to be complete. Contributions are very welcome, to enable convergence of protocols.

## Building Blocks

### Broadcast Pseudorandom IDs

### Generate Pseudorandom IDs with Ratchet/KDF
#### Variable Key Duration

### Distribute Bloom/Cuckoo Filters to Decorrelate IDs
Server aggregates IDs in a bloom or cuckoo filter. If IDs are derived from a secret, the server performs the derivation and adds the results to the filter. Then only the server know which IDs are correlated to a specific key, but the recepients of the filters do not.

### Randomize Order of Pseudorandom IDs

### Spread Secret Shared Pseudorandom IDs
Spread IDs to require multiple broadcasts for reconstruction of an ID. This should be compatible with [rotation synchronization](#synchronize-ble-mac-and-id-rotation), since a MAC and reconstructed ID could only be correlated if enough packets are received.
	
### Synchronize BLE MAC and ID Rotation
Sync rotation to mitigate [address-carryover attacks](https://petsymposium.org/2019/files/papers/issue3/popets-2019-0036.pdf).
	
### Sharding
Users push/pull reports to/from a bucket correlated to e.g. coarse space-time location, to reduce bandwidth requirements.

### Permissionless Self Reporting

### Submission Signing by Health Authorities

### Freeform Payload
Permits Self Reporting, but can also contain e.g. a signature by health authorities. Permits flexibility and makes few assumptions about the rest of the infrastructure.
