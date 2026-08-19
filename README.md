# reedhold-host

Sync JSON HTTP process for [Reedhold](https://github.com/sergii-ziborov/reedhold).

This is the **API process** that [reedhold-swift](https://github.com/sergii-ziborov/reedhold-swift)
and [reedhold-site](https://github.com/sergii-ziborov/reedhold-site) talk to.
It wraps `reedhold-api`. The protocol kernel still has no Tokio.

Default listen: `127.0.0.1:4783`. Override with `REEDHOLD_HOST`.

```sh
cargo run
```

```text
GET  /health
GET  /v1/invariants
GET  /v1/advertising/limits
POST /v1/account              { password, device_secret }
POST /v1/account/restore      { manifest_hex, password, device_secret }
GET  /v1/account
POST /v1/account/emit         { kind, payload }
POST /v1/account/verify       { event_hex }
POST /v1/account/password     { password }
```

Bodies are JSON. Values that are secrets or ids are hex or UTF-8 strings.
There is no Kotlin / JNI binding here.

> Prototype. Not independently audited.

## License

MIT.
