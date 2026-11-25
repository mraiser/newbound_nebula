# Phase 1 — quick fixes (nebula app)

Done 2026-11-25 on scratch (port 33182), branch `claude/nebula-phones`.
Screenshots: `docs/screenshots/phase1/` (`before-*` / `after-*`, 1× and 2×).

| # | What changed | Where |
|---|---|---|
| 2 | Run now: a failed start now gives one sentence in the status strip; the Service tab shows what to do for this OS (Linux: the exact `sudo setcap cap_net_admin=+ep <engine>` with a Copy button, plus "redo after every engine update"; macOS: root-only `utun`, run `sudo nebula -config …` in Terminal; Windows: Run as administrator + Wintun). The raw log sits under **Details**. On Linux the box shows up front when the engine lacks the permission (checked with `getcap`), and after an engine install the app re-checks and says so. The "no systemd, no sudo" hint is now per OS and true. | `start` (classifies failures: tun permission / port in use / other), `info` (`tun` block, shared helpers), UI |
| 5 | New CAs last 10 years (`nebula-cert ca -duration 87600h`; leap days put the end 3 days before the calendar date). Host/member certs keep nebula-cert's default: they end 1 second before the CA. The network page shows **Expires <date>**, and a banner warns 30 days before the CA or this host's cert runs out (red once expired). | `create_network`, `info` (`ca_not_after`, `host_not_after`), UI |
| 8 | New network proposes the lowest port (from 4242) and subnet that no network on this instance uses, across all names. It warns if another network uses the port, or if anything on the machine has it bound (new admin-only command `port_status`, a bind test that works on every OS). | `port_status` (new), `network` dialog, UI |
| 9 | When `http_address` is loopback, the Service tab says phones and members can't open Newbound's web page over the network, and names the setting (`http_address=0.0.0.0` in `config.properties` + restart). No config writes. | `info` (`http_address`), UI |
| 11 | Lighthouses copy: optional for Newbound peers, **required for phones**; the empty state says the same. | UI |
| 12, 14 | Primary buttons and checked toggles rested at `--accent-dim`, which read as disabled. The cause was in this app's own css, not the kit: they now rest at `--accent` like the Applications shelf (`sh-primary`) and Dev's Save. Hover brightens slightly. Disabled buttons look like Dev's disabled Save (panel + faint text) instead of a faded accent. | nebula css |
| 13 | Engine row: Version/Platform are dark kit-style selects (own `<select>`, not `app:select`, which renders a white native control), labels above, and the selects and Install share one 32 px height and bottom edge. One control height (`--nb-ctl-h`) now applies to every button, select and text input in the app. | nebula html/css/js |
| 15 | Back is a bordered "←" top-left, before `Nebula / local / <network>` (title "Back (Esc)"; Esc goes back; asks first if there are unsaved changes). The old "← All networks" is gone. **Save** follows Dev's workbench (`wb-chipbar`: facet tabs left, state + Save right): it sits at the right end of the network's tab bar with "no changes / unsaved changes", is disabled until something changes, shows `Ctrl+S`/`⌘S`, and Ctrl/⌘+S saves. | nebula html/css/js |
| 16 | The Platform list preselects this machine's build (from the installed tarball name, else the instance's OS/arch), not the first asset (darwin). Choosing another build shows a warning, and Install (or Install from URL) asks before installing a build for a different OS. | nebula js |

## Checked
- Every changed row measured in the browser: the run row, the systemd row, the tab bar (tab / state / Save), the header (← / title / crumb), the engine row (selects / Install, version text / buttons) and the command / Copy row all share one vertical centre. Heights are 32 px for controls.
- States: engine releases open; wrong platform; Run now failure (Details open); unsaved changes then Ctrl+S; Esc back; busy port (4243); 10-year CA on a network made in the dialog; banner at 10 days; loopback note (scratch briefly set to `127.0.0.1`, then put back); toggles on.
- Live (port 34241, `home`) untouched: same PIDs and start times, `home` still 192.168.100.1, file list and config hashes identical.

## Not done / for Marc
- **Windows engine path:** the app calls `runtime/nebula/bin/nebula` everywhere; on Windows that is `nebula.exe`, so Run now and certificate signing can't work there yet. It was like this before Phase 1 and isn't fixed here.
- macOS / Windows / Pi text is reasoned from how each OS works, not tested on those machines.
- The nebula crate's `lib.rs` has no `nb_ffi_contract_nebula` export, so hotswap loads it "unchecked" (it was like this before Phase 1).
- #1 (logged-out page), #9's actual setting, and #10 (HTTP) stay with Marc as planned.
