# Enrollment test vectors (ENROLLMENT.md §6)

**Throw-away data.** Made on 2026-11-25 from the test network `t-sample` on
the scratch instance. Its CA key was deleted afterwards, and the network was
never reachable: the lighthouse address `203.0.113.10` is a documentation-only
IP (TEST-NET-3). The private key in these files is not a real key.

| File | What it is |
|---|---|
| `join.url` | Mode A code as a URL: `newbound://join?v=1&d=<payload>` (1090 chars) |
| `join.nbjoin` | Same line, as the app's **Download .nbjoin** writes it |
| `join.payload` | The bare `<payload>` (readers must accept it without the URL) |
| `join.decoded.json` | The JSON inside (§1.2 + §2 fields) |
| `join.qr.svg` | The QR the app draws for `join.url` (ECC level L) |
| `join.mobile-nebula.yml` | **Download for Mobile Nebula** output for the same phone (§2.1) |
| `verify.py` | Independent check: decode, compare, `nebula-cert verify`, fingerprint, and the key's X25519 public key equals the cert's |

The files were made by the nebula app's own browser code (raw DEFLATE via
`CompressionStream('deflate-raw')`, base64url without padding), taken verbatim
from the `nebula` control's js.

Check them (needs `nebula-cert` v1.9.7 or newer):

```
python3 -I verify.py . /path/to/nebula-cert
```

The device in the sample: `Sample iPhone`, `192.168.253.4/24`, groups
`member,phone,family`, on `t-sample`, owner `Eyevea`.
`nebula -test -config join.mobile-nebula.yml` passes with nebula v1.9.7.
