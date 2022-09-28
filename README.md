# QuickLock

QuickLock is a research project that enables secure and convenient control of smart locks with support for flexible auditing and granting access to multiple users. It leverages efficient cryptographic primitives to minimize detrimental effects on user experience and enable practical implementations directly in smart lock controller firmware. The protocol can operate over multiple transports, including HTTP and NFC.

A general-purpose auditing scheme for request-response protocols is also included in this work, based on QuickLock's auditing.

**This is research-quality work and may have security flaws. Do not use in production.**

Research work done for Stanford [Secure Computing Systems](https://www.scs.stanford.edu/).

## Protocol

See the [protocol overview](PROTOCOL.md) for details.

## Implementations

- `app`: Android app implementing the client role over HTTP or NFC. Also supports the relying party and server role over NFC host card emulation (HCE) with NDEF and IsoDep.
- `server-rs`: Rust server implementation over HTTP. Also used to implement server functionality for Android NFC HCE.
- `server`: Old Java/Kotlin server. No longer used; protocol design has diverged significantly.
- `web`: WebAuthn-compatible variant. Depends on Larch server and WebAssembly port of Larch client built with Emscripten.
- `zkproof`: Experiments with NIZK zero-knowledge proofs, specifically Microsoft's Spartan proving system. No longer used.
