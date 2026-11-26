# Phase 2: Add a phone, Mode A (nebula app)

Done 2026-11-25 on scratch (port 33182), branch `claude/nebula-phones`.
Screenshots: `docs/screenshots/phase2/`. Test vectors: `docs/enrollment-samples/`.

## What's new
- **`add_phone(servicename, device_name, groups)`**: a new admin-only command. It:
  - picks the next free overlay IP, skipping this host, the lighthouses and every member;
  - signs a cert named after the phone, with groups `member,phone` plus any extras;
  - writes `members/<phone-…>/host.crt` and `info.json` (`"kind":"phone"`, name, IP, groups, added, expiry, fingerprint);
  - returns the Mode A payload of ENROLLMENT.md §1.2 + §2, field for field.

  `nebula-cert` writes the key into a private temp folder inside the network folder (the same place `ca.key` lives). The command reads it and deletes the folder at once. The key is never stored under `members/`.
- **Lighthouses for phones**:
  - this host counts when it is marked as a lighthouse and has a **public address** (a new Service-tab field, saved as `public_address`);
  - every lighthouse entry with a public IP counts too;
  - with none, the command refuses: "Add a lighthouse first — phones need one". The dialog then shows a button to the right tab: Lighthouses, or Service when this host is the lighthouse but has no address yet.
- **Encoding** (ENROLLMENT §1.1): JSON → raw DEFLATE → base64url, no padding → `newbound://join?v=1&d=…`. The browser does it with the built-in `CompressionStream('deflate-raw')`: offline, no library.
- **QR**:
  - the app uses the vendored `vendor/qrcode.js` (MIT, davidshimjs + Kazuhiko Arase; licence and provenance in `vendor/qrcode.LICENSE.txt`) with ECC level L, drawn as black-on-white SVG;
  - it is blurred until **Show code**; a countdown runs, and at `expires` (10 minutes) the code, the link and the key are dropped from the page and can't be shown again;
  - closing the dialog drops them too;
  - if the link is over 1,800 characters, the file and link buttons come first and a note says why. The sample is 1,090.
- **Download .nbjoin**, **Copy link**, **Download for Mobile Nebula** (§2.1): the Mobile Nebula file is a plain nebula YAML with the PEMs inline, lighthouse and static hosts, `punchy` on, and an inbound firewall that allows only ICMP.
- **Members**: phones are listed with a PHONE badge and the date added. **Sync mesh** skips phones, because they aren't on the Newbound peer network. Remove works as before (it deletes the folder); revoking the cert is Phase 3.

## Checked
- The vendored encoder against node-qrcode 1.5.4 (MIT): 12 payload lengths (22 to 2,900 characters), every module matrix identical for the same version and mask.
- The test vectors with an independent Python verifier (`docs/enrollment-samples/verify.py`):
  - the URL decodes to the same JSON;
  - every contract field is present;
  - `nebula-cert verify` accepts the cert against the CA;
  - `ca_fingerprint` matches;
  - the cert's IP, name and groups are right;
  - the private key's X25519 public key equals the cert's.
- `nebula -test -config join.mobile-nebula.yml` passes with nebula v1.9.7, so the inline-PEM YAML is valid for nebula itself.
- The UI on scratch:
  - name step; code blurred, then shown (117 modules at ~446 px, about 3.8 px per module);
  - expiry (clock moved forward in the page); the no-lighthouse refusal and its button;
  - the public-address field; the Members list with phones; 2× zoom.
- Live (port 34241, `home`) untouched: network files and config hashes match this morning's baseline. Live is down only because Eyevea restarted at 20:22 (the Android session's BIOS change).

## Real-phone check (Alex, 2026-11-25, home Wi-Fi)
Test network `t-phone` on scratch: 192.168.254.0/24, UDP 4250. Eyevea was the
lighthouse at its LAN address 192.168.68.109:4250, so the router wasn't touched.
1. Alex logged in to scratch from Safari on the iPhone. Scratch has its own
   accounts: the `admin` from live doesn't work there.
2. Members → Add a phone → **Download for Mobile Nebula** → Defined Networking
   nebula **imported the YAML** (inline PEMs work). That was the §2.1 check, so
   the button stays.
3. The site connected: Eyevea logged the handshake with "Alex iPhone test", and
   Eyevea → 192.168.254.2 pings came back.
4. `http://192.168.254.1:33182` loaded on the phone, once ufw allowed TCP 33182
   on the `t-phone` device.

Setup it needed on Eyevea (all run by Alex):
- `sudo setcap cap_net_admin=+ep …/newbound-scratch/runtime/nebula/bin/nebula` (scratch's copy only);
- `sudo ufw allow from 192.168.68.0/24 to any port 4250 proto udp`;
- `sudo ufw allow from 192.168.68.0/24 to any port 33182 proto tcp`;
- `sudo ufw allow in on t-phone to any port 33182 proto tcp`.

**Finding (for Phase 5 / Marc):** on Linux with ufw (default inbound DROP),
a phone can join the tunnel and still not open Newbound, because nothing
allows Newbound's port on the tunnel device. The app should detect an active
ufw and show the exact `ufw allow in on <network> to any port <http_port>
proto tcp`, like it does for setcap.

## Not checked yet
- **QR scan** by a phone: there's no QR decoder on this machine, and nebula
  mobile isn't installed yet. The QR is proven only through the encoder cross-check.
- Over mobile data: that needs a UDP port forward on both routers (double NAT).
- I didn't click the download and copy buttons in the desktop browser (they
  would put a test key in Alex's Downloads and on his clipboard). Alex's
  download on the iPhone exercised the Mobile Nebula one.
- Copying the samples into `newbound-mobile/docs/enrollment-samples/`
  (ENROLLMENT §6) is waiting on Alex's OK.

## Notes
- The payload includes `static_hosts` from the network's mesh map (members' LAN and observed endpoints) when there are any, so a phone on the same Wi-Fi can connect directly.
- `owner` and `newbound_url` come from the running platform (`machineid`, the app's peer uuid, `http_port`). Under `newbound exec` they're read from `config.properties` and `runtime/app/botd.properties`.
