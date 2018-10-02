
# Konvoy

A toy decentralised publishing network.

Authors create files in `/corpus`, then run `cargo run --bin corpus` to
generate keys if necessary, and a signed tar archive of the corpus files.
Running `cargo run --bin server` will start hosting the archive.

Consumers list their favourite authors' public keys (base64 encoded) in
`/following.txt`, and run `cargo run --bin client` to connect to servers and
retrieve the latest archives. If an archive signature is valid, the archive has
not been tampered with, and is downloaded into `/archives` to be opened and
enjoyed. Archives with invalid signatures are rejected.

Signatures are stored in the archive filename, and are generated against a
concatenation of the author's public key, the Konvoy version, the current
timestamp, and the archive's data. Any modification of these properties will
result in an invalid signature.

The server will not send archives of a particular author that are older than
those a client already has.

Currently the client only checks localhost for a running server - some kind of
peer-to-peer server discovery is needed for a proper network.

