# Invites over home Wi-Fi (Mode B1): design and questions for Marc

> **Decided by Alex, 2026-11-25:** anonymous commands are fine (Newbound allows
> them by design, and Marc knows); B1 **uses the approval tap**; the contract now
> says answers come in Newbound's `{status, data}` envelope, with a `waiting` state and
> `invite_status` polling (newbound-mobile branch `contract/b1-envelope-approval`).
> Still Marc's: questions 2, 4 and 5 below.

**TL;DR.** B1 lets a phone join with one scan, **without its private key ever
leaving the phone**. The phone scans an invite QR, makes its own key, and
sends only the public key to Newbound over the home network. Newbound checks
the one-time token in the invite and answers with the certificate. Everything
needed exists in the platform except four decisions that are Marc's:
1. may the nebula app have **one command callable without a login**;
2. should Newbound **listen on the home network**;
3. is the **plain-HTTP token risk** acceptable (or should we add an approval tap);
4. can a command learn the **caller's IP**.

Nothing that depends on those is built. The dialog shows the option, disabled,
with the reason.

Contract: `newbound-mobile/docs/ENROLLMENT.md` §3.1 (kind `invite`). Today the
nebula app already ships Mode A (one scan, Newbound makes the key) and Mode B2
(paste the phone's public key), so B1 is a convenience, not a gap.

## 1. The flow
1. Nebula app: network → Members → **Add a phone** → **Invite over home Wi-Fi** → name.
   Newbound reserves the next IP and makes a one-time token. It shows an
   invite QR, blurred until "Show", like Mode A, because the token is secret.
2. The phone (on the same Wi-Fi) scans it, makes an X25519 keypair, and stores it as "pending".
3. The phone POSTs `{token, pubkey, device_name}` to the first `lan_urls` entry that answers.
4. Newbound checks the token, signs the public key (`nebula-cert sign -in-pub`,
   the same code as B2), records the phone (`members/<id>/`, `kind: phone`,
   `mode: b1`), and answers with `ca`, `cert`, `ip`, `lighthouses`,
   `static_hosts` and `newbound_url`.
5. The phone checks `ca` against the invite's `ca_fingerprint` and that
   `cert` holds its own public key. Then it connects.

## 2. What the platform does today (checked on scratch, 2026-11-25)
| Need | Today | Where |
|---|---|---|
| Call without a login | Possible already: `check_security` lets anyone through when the **library's** readers include `anonymous` **and** the **command's** readers include `anonymous`. The nebula library's readers are already `["anonymous"]`; every nebula command is admin-only (empty readers). | `newbound_core/src/security/security/init.rs` `check_security` |
| `POST /nebula/redeem_invite` with a JSON body | Works with no new routing: `/<app>/<cmd>` reaches the app's control command, and an `application/json` body becomes the params. | `app/service/init.rs` (body ~l.494, route ~l.1515) |
| Answer shape | Every command answer is wrapped: `{"status":"ok","data":{…},"nn_return_type":…,"pid":0}`. A refusal is **HTTP 200** with `{"status":"err","msg":"UNAUTHORIZED: …"}`. | measured with curl on scratch |
| Caller's IP | **Not available to ordinary commands.** The HTTP layer records it as the `nn-userlocation` header, but only the peer-relay command receives headers. | `app/service/init.rs` ~l.736 |
| Listening on the LAN | Set by `http_address` in `config.properties`. flowlang's default when the key is missing is `0.0.0.0`, but Eyevea's live instance listens on 127.0.0.1 (finding #9). | `flowlang appserver.rs` l.263 |
| Random token | **Rust's std has no secure random source.** The app has no crate for one (no third-party deps). | — |

## 3. Proposed design (to build once Marc decides)
**`make_invite(servicename, device_name, groups)`**, admin-only:
- Same checks as `add_phone`: a lighthouse with a public address, the next free IP, groups `member,phone` plus extras.
- It reserves the IP in memory and makes a 32-byte token (see question 5 for where the randomness comes from).
- It keeps only **SHA-256(token)** in the process globals, with the network, name, IP, groups, `expires` (now + 10 min) and an attempt counter. It never writes to disk, so a restart simply cancels open invites.
- It returns the `invite` payload (§1.2 fields + `token`, `lan_urls`, `device_name`, `ip`, `groups`, `expires`).
- `lan_urls`: `http://<each non-loopback IPv4>:<http_port>`, from the same interface list `peer/info` uses. If `http_address` is loopback, there are none, so the app refuses with "Newbound isn't reachable on your network — use 'paste the phone's public key' instead".

**`redeem_invite(token, pubkey, device_name)`**, the one **anonymous** command:
- It looks up SHA-256(token). Unknown, used and expired tokens all get the same answer: "This invite has expired or was already used. Make a new one on the computer." That way nobody can probe which tokens exist.
- It checks the public key as B2 does (PEM, 32 bytes), then signs it with the B2 code (shared with `add_phone`).
- The token is spent on the first successful signing. After 5 bad attempts on a valid token, the invite is cancelled.
- It records the member and answers with the §3.1 fields.
- It never creates a session and never touches anything but this one invite.
- A 256-bit token can't be guessed. The counter and the 10-minute life stop slow attempts.

**UI:** a third choice in Add a phone, next to "paste its public key". It shows the invite QR, a countdown, and "Waiting for the phone…", which turns into "Alex's iPhone joined" when the token is spent (the page polls a new admin-only `invite_status`).

**Approval tap (decided: Alex, 2026-11-25):**
- `redeem_invite` doesn't sign straight away. It checks the token and key, stores the public key with the invite, and answers `data.status: "waiting"` plus the `pubkey_fingerprint`.
- The owner's page (polling a new admin-only `invites(servicename)`) shows "Alex's iPhone sent a key ending **ab12 cd34** — the same as on the phone? [Approve] [Refuse]". The phone shows the same 8 hex characters.
- **Approve** (admin-only `approve_invite(servicename, invite_id, approve)`) signs and records the phone. **Refuse** cancels the invite.
- The phone polls the anonymous `invite_status(token)` every 2 s until `ok` or `err`.
- A second `redeem_invite` with another key while one is waiting is refused, so a swapped key can't slip in unnoticed.

## 4. Questions for Marc
1. ~~Anonymous command.~~ **Answered (Alex): by design.** `redeem_invite` and `invite_status` get readers `["anonymous"]`. Still welcome from Marc: any house rules for anonymous commands (logging, rate limit, naming).
2. **Listening on the home network.** Should Newbound listen beyond 127.0.0.1 by default (`http_address=0.0.0.0`), or offer a "home network only" choice? Until it does, B1 can't work on an instance like Eyevea's, and the app says so.
3. **Plain HTTP.** On the same Wi-Fi, someone who can intercept traffic could swap in their own public key and join **as that phone**. The phone would see an error, but the attacker would be in. The CA itself can't be faked, because the phone checks it against the fingerprint in the QR. Options:
   - **(a)** accept it for v1: single use, 10 minutes, home Wi-Fi only;
   - **(b)** add the approval tap above;
   - **(c)** wait for HTTPS.

   **Answered (Alex): (b), the approval tap.**
4. **Caller IP.** Can a command learn the caller's address (for example a `nn_remote` param)? Then `redeem_invite` could refuse anything that isn't a private/LAN address, so a port forward to Newbound's web port could never expose it.
5. **Random bytes.** std has no secure random source and the app takes no crates. Options:
   - read the OS source directly (`/dev/urandom` on Linux/macOS/Pi; Windows needs an OS call);
   - hash a fresh key from `nebula-cert keygen` (Go's `crypto/rand`, works on every OS);
   - flowlang exposes a CSPRNG helper.

   Which do you prefer?
6. ~~Answer envelope.~~ **Answered (Alex):** the contract changes. Phones read `data`; the outer `status` is transport, `data.status` is the verdict.

## 5. Contract changes
- **Applied (Alex approved):** §3.1 answers in Newbound's envelope (`data.status` = `waiting` / `ok` / `err`), the approval tap, and `POST /nebula/invite_status {token}` polling. They're on newbound-mobile branch `contract/b1-envelope-approval`.
- **Still proposed** (in the contract's "Proposed changes", for the nebula mobile builders): `pubkey_fingerprint` = SHA-256, lowercase hex, of the raw 32-byte X25519 public key.

## 6. What's built now
Only the disabled entry point. In **Add a phone**, "Invite over home Wi-Fi" is shown disabled with one line: it needs a platform decision, use "paste its public key" for now. When the instance listens only on loopback it also says "Newbound isn't reachable on your network". No token, command or listener exists.
