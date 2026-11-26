# QA report: nebula app, Phases 1–3 (+ Phase 4's disabled entry)

**TL;DR.**
- On scratch (Linux, Eyevea), **42/42 command checks pass**, among them **every one of the 25 nebula commands refuses when logged out**.
- **21 UI states** were measured at 1× and 2×: every row's controls share a vertical centre within 1 px (all measured 0 px), and every pane starts its blocks at one left edge.
- Real iPhone: Defined Networking nebula imported the file, connected and loaded Newbound; after **Remove** it was refused ("certificate is in the block list").
- QA found one gap, **ufw silently blocking phones**, and fixed it.
- Still open: the logged-out broken page (#1, Marc's) and a few per-OS items, listed at the end.

Run on 2026-11-25, scratch instance (`/home/gm/Desktop/newbound-scratch`, port 33182), branch `claude/nebula-phones`.
Screenshots: `docs/screenshots/phase1..5/`.

## 1. Commands (automated)
`python3 -I docs/qa/qa_commands.py <instance> <port>` logs in as admin (the password is never printed), builds a throw-away network `t-qa`, exercises everything over HTTP like the page does, then deletes `t-qa`. Output: `docs/qa/qa_commands.out.txt`.

| Area | Checks |
|---|---|
| Logged out | All 25 nebula commands answer `UNAUTHORIZED`, including the new `add_phone`, `port_status`, `update_blocklist`, `set_firewall` and `remove_member`. |
| #2 / #9 info | `tun` block (os, arch, how, ready), `http_address`, `http_port`, `ufw`. |
| #8 port_status | A port held open reads bound; once released it reads free; a non-number gets a sentence. |
| #5 / §5 create_network | CA lasts 10 years; this host's cert is in group `member`; `firewall: member`; inbound `groups: [member]`; `nebula -test` passes. |
| Add a phone refusals | No lighthouse, empty name and bad group are each refused with a reason. |
| Mode A | Kind `join` with a key; the lighthouse gets the network port added; groups `member,phone,family`; expires = created + 600; owner and `newbound_url` filled; the member folder holds only `host.crt` + `info.json`; no temp folder left; the cert verifies; the CA fingerprint matches. |
| Mode B2 | Kind `cert` with no key; `pubkey_fingerprint` equals SHA-256 of the raw key; the cert holds the phone's public key; a damaged key is refused. |
| Members | Both phones listed with added, expiry and fingerprint; modes `a` and `b2` recorded. |
| §5 add_member | Group `member` is added on a member-firewall network. |
| Remove | Fingerprint returned; folder deleted; `config.yml` blocklists it; `nebula -test` passes; an unknown member is refused; a bad fingerprint and a bad firewall mode are refused. |
| Older networks | `t-before` and `t-after` re-render byte-identical. |
| Run now / Stop | Starts supervised (engine has `cap_net_admin`), and Stop stops it. The **failure** path (no `cap_net_admin`) was tested in Phase 1: one sentence, the `setcap` fix with Copy, the log under Details (`docs/screenshots/phase1/after-runnow-error-1x.jpg`). |

## 2. UI, every changed screen, 1× and 2×
An in-page checker measured each visible row (header, tab bar, engine rows, run row, dialog footers, action rows, command rows): the vertical centres, or the bottom edges for the engine-install row, of all controls in a line.

| State | 1× | 2× |
|---|---|---|
| Home: engine releases open (linux-amd64 preselected) | 0 px | 0 px |
| Home: wrong platform chosen (warning) | 0 px | — |
| New network: proposed values; port used by another network (warning) | 0 px | 0 px |
| Network → Service (expiry, inbound line, ready-state run hint, public address) | 0 px | — |
| Service with unsaved changes (Save enabled, "unsaved changes") | 0 px | 0 px |
| Lighthouses (phones-need-one copy) | 0 px | — |
| Members (phones, added/expires, blocked list) | 0 px | 0 px |
| Add a phone: Mode A form / B2 form / disabled invite | 0 px | — |
| Join code: blurred / shown (386 px QR at this pane size) / expired | 0 px | 0 px (shown) |
| B2 result (cert QR, copy cert/CA) | 0 px | 0 px |
| No-lighthouse refusal with a fix button | 0 px | — |

Also checked:
- **left edges**: Service, Lighthouses, Members and the Add-a-phone dialog each start every block at one x;
- **Esc** with unsaved changes asks first, and Cancel stays put;
- **Ctrl+S** saves and clears the state (Phase 1);
- **Back ←** returns to the list.

The browser pane was 800×600, so the 2× screenshots show part of the page; the 2× checks are the measurements plus those crops.

## 3. Logged out, loopback, firewall
- **Logged out:** the nebula page renders unstyled and broken (`docs/screenshots/phase5/logged-out-broken-page-finding1.jpg`). This is **finding #1**: the platform's page loader doesn't send a logged-out visitor to the login page when a nested kit control is refused. It's `data/app`, Marc's, so it's not changed here. Every nebula command still refuses without a login (section 1).
- **Loopback** (scratch briefly on `http_address=127.0.0.1`, then put back):
  - the Service tab says phones can't open Newbound's page and names the setting;
  - the firewall section drops the web-port rule (no use while loopback-only);
  - the disabled invite option adds "Newbound also isn't reachable on your network yet".
- **ufw (fixed in this phase):** on Linux with an active ufw, the Service tab has a collapsed "Firewall: ufw is on" section. It holds the two exact rules (the tunnel's UDP port, and Newbound's port **only on the network's interface**), each with Copy. It's collapsed because the rules can't be read without sudo and may already exist. This is the gap the Phase 2 phone test hit.

## 4. Real iPhone (Alex, home Wi-Fi, test network `t-phone`)
| Step | Result |
|---|---|
| Add a phone → **Download for Mobile Nebula** → import in Defined Networking nebula | Imported (inline PEMs accepted) |
| Connect | Handshake on Eyevea; Eyevea ↔ phone pings both ways |
| Open `http://192.168.254.1:33182` | Loaded once ufw allowed the port on `t-phone` (now shown by the app) |
| **Remove** the phone | Eyevea pings 2/2 → 0/2; the page stopped loading; the log shows `certificate is in the block list` for every retry |

Not covered: **scanning the QR** (nebula mobile isn't installed on a phone yet, and this machine has no QR decoder; the encoder matches node-qrcode bit for bit), and **mobile data** (it needs router port forwards).

## 5. Per OS
✅ tested on Eyevea (Ubuntu, x86_64) · 🔶 reasoned, not run · ⚠️ known gap

| Feature | Linux | NixOS | macOS (Apple Silicon) | Windows | Raspberry Pi |
|---|---|---|---|---|---|
| Run now: permission check | ✅ `getcap` + uid; both states seen | 🔶 same code; `setcap` needs libcap in the profile (the message says so) | 🔶 root check (`id -u`); message: run `sudo nebula -config …` in Terminal (no launchd service) | 🔶 admin check (`net session`); message: Run as administrator + Wintun. ⚠️ the engine path is `bin/nebula`, not `nebula.exe` (pre-existing), so Run now and signing can't work yet | 🔶 same as Linux (`libcap2-bin` on Raspberry Pi OS) |
| 10-year CA, expiry line + banner | ✅ | 🔶 | 🔶 | 🔶 (⚠️ the `nebula-cert` path, as above) | 🔶 |
| Port proposal + busy-port check (bind test) | ✅ | 🔶 | 🔶 | 🔶 | 🔶 |
| Loopback note | ✅ | 🔶 | 🔶 | 🔶 | 🔶 |
| Platform preselect + warning | ✅ (from the tarball name) | 🔶 | 🔶 `darwin` (universal) | 🔶 `windows-amd64` / `-arm64` | 🔶 `linux-arm64`; ⚠️ 32-bit Pi maps to `arm-7`, wrong for Pi Zero/1 (armv6) unless the installed tarball names it |
| Add a phone: temp folder for the key | ✅ 0700, deleted at once | 🔶 | 🔶 0700 | 🔶 inherits the network folder's ACL; deleted at once | 🔶 |
| Join code encoding (`CompressionStream`) | ✅ Chrome | depends on the **browser**, not the OS: Chrome/Edge 103+, Firefox 113+, Safari 16.4+ | | | |
| Copy buttons on plain HTTP | ✅ `execCommand` fallback | same as left | | | |
| ufw hint | ✅ | 🔶 (NixOS usually uses its own firewall module, not ufw, so no hint) | — pf/app firewall not detected | — Defender Firewall not detected (Windows asks on first listen) | 🔶 if ufw is installed |
| Ctrl/⌘+S label | ✅ Ctrl+S | 🔶 | 🔶 ⌘S | 🔶 | 🔶 |

## 6. Open items
1. **#1, logged-out page broken**: Marc (`data/app` page loader).
2. **Windows engine path** (`nebula` vs `nebula.exe`) across the app: pre-existing, not fixed here.
3. **32-bit Pi Zero/1** platform guess (`arm-7` vs `arm-6`): only matters before the first engine install.
4. **Sending the blocklist and firewall mode to a second Newbound peer** is untested (no peer connected to scratch).
5. **QR scan by a phone**: waits for nebula mobile.
6. Scratch has `t-before` and `t-phone` on the same port 4250, because `t-phone` was made from the shell, bypassing the dialog's warning. It's harmless while only one runs.
7. The nebula crate has no `nb_ffi_contract_nebula` export, so it loads "unchecked" (pre-existing).
