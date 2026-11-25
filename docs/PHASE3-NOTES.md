# Phase 3: members, removal, Mode B2 (nebula app)

Done 2026-11-25 on scratch (port 33182), branch `claude/nebula-phones`.
Screenshots: `docs/screenshots/phase3/`.

## What changed
| Spec item | Change |
|---|---|
| 1. Members list | Phones and Newbound peers in one table: name, kind (a PHONE badge, and whether Newbound or the phone made the key), IP, groups, **Added**, **Expires** (orange within 30 days). `members` now fills `not_after` and `fingerprint` from the cert for members added before Phase 3. |
| 2. Remove | `remove_member` (now returns an object) puts the member's cert fingerprint on the network's blocklist (`connection.json` `blocklist`: fingerprint → {name, removed}), deletes its folder, and applies the list here: `save_config` renders `pki.blocklist`, and a running supervised tunnel restarts. The page then sends the list to every Newbound member with the new `update_blocklist`, and **Sync mesh** sends it again (with the firewall mode) so members that were offline catch up. A "Removed — certificates blocked" table lists them. Raw-YAML networks get the list recorded and a message to add it by hand. |
| 3. Mode B2 | In **Add a phone**, "The phone made its own key? Paste its public key" switches the dialog. `add_phone` takes a new `pubkey` param: it checks the PEM (32-byte X25519), signs with `-in-pub`, and makes no private key. It returns a `cert` code: the Mode A fields minus `key`, plus `pubkey_fingerprint`. The QR isn't blurred (nothing secret). **Copy certificate** and **Copy CA** cover Defined Networking nebula's paste-by-hand flow. |
| 4. Firewall (§5) | **New** networks: this host's cert gets group `member`, `connection.json` gets `firewall: member`, and the default inbound rules render as `groups: [member]`. `add_member` adds `member` to new Newbound members on such networks. The new `set_firewall` lets the owner set the same mode on members (sent on Sync mesh, which runs after every join). Existing networks render byte-for-byte the same config as before. The Service tab shows the network's inbound rule. |

New admin-only commands: `update_blocklist(servicename, blocklist)`, `set_firewall(servicename, mode)`. Changed: `remove_member` (returns an object), `add_phone` (+`pubkey`), `members`, `save_config`, `create_network`, `add_member`.

## Checked
- **Real iPhone (Alex, home Wi-Fi, `t-phone`):**
  - connected and loaded `http://192.168.254.1:33182`;
  - **Remove** on "Alex iPhone test" → "removed and its certificate blocked (the tunnel restarted to apply it)";
  - Eyevea → phone pings went from 2/2 to 0/2, and the page stopped loading on the phone;
  - Eyevea's log showed `certificate validation failed: certificate is in the block list` for every retry from 192.168.68.104.
- **B2 on scratch:**
  - a keypair made with `nebula-cert keygen` was signed;
  - the code has no `key`; the cert verifies against the CA and holds exactly that public key;
  - `pubkey_fingerprint` equals Python's `hashlib.sha256` of the raw key;
  - a bad key gets a plain refusal, and no temp folder is left.
- **Remove on scratch:**
  - three test phones (B2 and UI) were removed and blocked;
  - `config.yml` renders `pki.blocklist`, and `nebula -test` passes.
- **Firewall:**
  - a new `t-fw` network: host cert groups `[member]`, inbound `groups: [member]`, and a test member asking for `family` got `member,family`;
  - `nebula -test` passes;
  - `set_firewall` switches an old network both ways and back;
  - `t-before` (made before Phase 3) re-renders to the identical file.
- Live (port 34241, `home`): network files and config hashes match this morning's baseline (live is still off since the 20:22 reboot).

## Not checked
- **Sending the blocklist and firewall mode to a second Newbound instance**: no other Newbound peer is connected to scratch. The code path is `pushPolicy` → `send_update_blocklist` / `send_set_firewall` on each member, and failures name the peer (an older peer without these commands shows up there).
- Phones can't receive the blocklist (they aren't on the peer network). They reach the network through its lighthouses, which refuse a removed phone; but a phone that already knew another phone's direct address could still reach it. Rotating the CA is the complete answer.

## For Alex / the contract
- **`pubkey_fingerprint` isn't defined in ENROLLMENT.md §3.2.** The nebula app uses **SHA-256 (lowercase hex) of the raw 32-byte X25519 public key** (the base64 body of the PEM, decoded). Proposed for the contract's "Proposed changes"; it needs Alex's OK before nebula mobile relies on it.
- `remove_self` (a phone removing itself, proposed in the contract) isn't built: it's waiting for Alex's confirmation.
