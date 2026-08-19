# reedhold-host

Sync JSON HTTP process for [Reedhold](https://github.com/sergii-ziborov/reedhold).

This is the **API process** that [reedhold-swift](https://github.com/sergii-ziborov/reedhold-swift)
and [reedhold-site](https://github.com/sergii-ziborov/reedhold-site) talk to.
It wraps `reedhold-api`. The protocol kernel still has no Tokio.

Default listen: `127.0.0.1:4783`. Override with `REEDHOLD_HOST`.

```sh
cargo run
```

Each browser tab gets its own seat token (`X-Reedhold-Seat`). The host is
not one shared user: a stranger in incognito does not become whoever last
typed a password. Public aliases and topic rooms are shared; DMs and groups
belong to the seat. Overlays (durable grid, chain, reputation, ads, work)
are still in-process sandboxes. The host is not a source of truth.

```text
GET  /health
GET  /v1/invariants
GET  /v1/advertising/limits

POST /v1/account              { password, device_secret }
POST /v1/account/restore      { manifest_hex, password, device_secret }
GET  /v1/account
GET  /v1/account/manifest
GET  /v1/account/history
POST /v1/account/emit         { kind, payload }
POST /v1/account/verify       { event_hex }
POST /v1/account/password     { password }
POST /v1/account/split        { threshold, total }
POST /v1/account/combine      { threshold, password, device_secret, shares }
POST /v1/account/sealed       { conversation_key, plaintext }
POST /v1/account/open         { conversation_key, envelope_hex }
POST /v1/sync/plan            { epoch, candidates, prior_commit?, company?, relay_count? }

POST /v1/talk/open            { epoch, candidates, prior_commit?, company?, relay_count? }
POST /v1/talk/online          { peer }
POST /v1/talk/offline         { peer }
POST /v1/talk/block           { peer }
POST /v1/talk/dm              { to, to_msg_pub, plaintext }
POST /v1/talk/group           { name }
POST /v1/talk/invite          { group, member, member_msg_pub }
POST /v1/talk/send            { group, plaintext }
POST /v1/talk/remove          { group, member }
GET  /v1/talk/inbox
GET  /v1/talk/circles
POST /v1/alias                { nick }          not written into events
POST /v1/alias/lookup         { nick }
GET  /v1/contacts
POST /v1/contacts             { identity, messaging_public, petname? }
POST /v1/contacts/remove      { identity }
GET  /v1/chats
POST /v1/rooms/join           { topic }
POST /v1/rooms/leave          { topic }
POST /v1/rooms/post           { topic, text }
GET  /v1/rooms
POST /v1/interests            { topics }
GET  /v1/interests
GET  /v1/topics

POST /v1/durable/open         { holders, company? }
POST /v1/durable/put          { payload, tier? }
POST /v1/durable/get          { id }
POST /v1/durable/kill         { holder }
POST /v1/durable/repair       { id }

POST /v1/chain/open
POST /v1/chain/commit         { epoch, identity, groups, storage }
GET  /v1/chain/head
GET  /v1/chain/headers
POST /v1/chain/prove          { leaves, index }
POST /v1/chain/verify         { leaf, root, index, siblings }

POST /v1/rep/open
POST /v1/rep/seed             { identity, continuity, social, content, curation }
POST /v1/rep/react            { author, target, kind, cluster?, now }
POST /v1/rep/identity         { identity, now }
POST /v1/rep/content          { target, now }
POST /v1/rep/transfer         always fails

POST /v1/ads/open
POST /v1/ads/post             { advertiser, campaign, payload, topic, bucket_min, bucket_max, budget, expiry }
POST /v1/ads/register         { id, strength }
POST /v1/ads/bid              { advertiser, campaign, topic, bucket, epoch, price }
POST /v1/ads/clear            { topic, bucket, epoch }
POST /v1/ads/select           { topic, bucket, epoch }   no user id
POST /v1/ads/hide             { campaign }
POST /v1/ads/settle           { topic, bucket, epoch }
POST /v1/ads/bucket           { strength }

POST /v1/work/open
POST /v1/work/record          { node, kind, units, epoch, reliable }
POST /v1/work/view            { node, social }
POST /v1/work/transfer        { from, to, amount }
```

Bodies are JSON. Values that are secrets or ids are hex or UTF-8 strings.

> Prototype. Not independently audited.

## License

MIT.
