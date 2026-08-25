---
status: "proposed"
date: 2026-08-19
decision-makers: ["Orogenys"]
consulted: []
informed: []
---

# Hybrid Mesh/SFU Voice Topology with Coordinator-Decided Switching

## Context and Problem Statement

ADR 0003 chose a P2P mesh over iroh for game-party voice chat, scoped to
small parties (~5 users), and explicitly deferred the question of larger
rooms. `faultline-voice` now needs to support both small parties *and* large
rooms with a single client API.

The core constraint is client uplink: in a mesh of N members, each client
uploads N-1 copies of its audio. Past ~8 members this exceeds typical
residential uplink and battery budgets, regardless of whether packets flow
directly or through a relay (the iroh relay forwards per-connection encrypted
packets; it does not reduce the number of copies a client sends).

## Decision Drivers

* One client API for any room size; games should not care about topology
* Small parties keep near-zero server media cost (mesh, direct paths)
* Large rooms must bound client uplink to a single stream
* The host stays in control of identity and admission
* Reuse the faultline stack (iroh, QUIC datagrams, bitcode)

## Considered Options

* Mesh only, cap room size at ~8
* SFU only, all media through the server
* Hybrid: mesh for small rooms, SFU for large ones, decided by the room
  coordinator and switched live

## Decision Outcome

Chosen option: **hybrid mesh/SFU with coordinator-decided switching**.

* The **room coordinator** (in `VoiceServer`) owns the topology decision:
  rooms up to `mesh_max` (default 8) are mesh; larger rooms use the SFU.
  Downgrades apply hysteresis (`downgrade_slack`) to avoid flapping.
* Topology changes are announced with a monotonically increasing **epoch**;
  clients apply only newer epochs and rebuild their media plane in place.
  The `VoiceSession` API is identical under both topologies.
* The **SFU is an iroh node** speaking the same voice ALPN, so it inherits
  NAT traversal and relay fallback. It forwards opaque datagrams without
  decoding audio, using sender-computed energy bits in the frame header to
  select the top-N active speakers in large rooms.
* **Admission** is host-driven: the game server issues ed25519-signed
  `VoiceTicket{user, room, expiry}`; the voice server verifies the ticket
  plus a proof-of-possession of the client's iroh endpoint key (a signed
  challenge), binding `user_id ↔ EndpointId`. SFU membership is gated by
  per-member session tokens minted by the coordinator.

### Consequences

* Good, because small parties keep the ADR-0003 cost profile (no server media)
* Good, because large rooms bound client uplink to one stream
* Good, because games integrate once and get both behaviors
* Good, because the SFU reuses iroh connectivity instead of new infra
* Bad, because a topology flip has a brief media gap (~1 RTT + handshake)
* Bad, because the SFU is a new server role to operate for large rooms
* Bad, because top-N speaker selection trusts sender-reported energy (a
  malicious client can shout its way in; acceptable for game parties,
  revisit if rooms become adversarial)

## More Information

Implemented in `faultline-voice` (crates: session, media plane, SFU,
coordinator, tickets, jitter buffer, optional `audio` pipeline). ADR 0003
remains valid for the small-party case; this ADR supersedes its "revisit for
larger rooms" clause.
