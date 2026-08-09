# Nebula App — User Journeys and UI Assessment

**Status: assessment, awaiting the owner's direction** (2026-08-09, after the
library overhaul merged at b15f7fc). Method: every journey below was traced
**live** — Playwright driving the real app on a disposable instance with a
real nebula v1.9.5 binary, an owned network (`nbtest1`, one member) and a
joined network (`nbtest1-joined`) — plus a full read of the four controls'
facets. "Live-traced" means observed happening, not inferred from code.
Code references are `nebula.nebula js:<line>` against the merged facet.

## 1. The expected journeys

The journeys the app is *designed* for are sound. The execution is what
fails.

- **J1 — Install the engine.** Open the app → "Installed Version: Not
  Installed" → Check for update → pick version and platform binary →
  Install. (Server-side download via `install_release`; listing via
  api.github.com from the browser.)
- **J2 — Create a network (become an owner).** ⊕ → name / subnet pattern /
  port → OK → network page → set interface/port/IP, mark lighthouses, Save
  configuration.
- **J3 — Grow the network.** Members tab → ⊕ → pick a connected peer,
  accept the proposed overlay IP, optional groups → OK → cert signed
  locally (`add_member`), credentials delivered over the peer channel,
  remote instance joins (`join_network` on the target).
- **J4 — Operate.** Service and Running toggles (systemd install/start);
  edit config; Advanced tab for raw YAML override.
- **J5 — Remote operation.** Peer selector at the top re-points the whole
  app at another admin peer; "Install App" bootstraps a peer that lacks
  the app.
- **J6 — Leave / clean up.** Delete a member; uninstall the service;
  remove the library (`dev.github.remove`).

## 2. What is actually wrong, by severity

### A — blockers and dangerous behavior

- **A1. The layout is broken on today's platform.** The home page and the
  Members/Lighthouses tabs render the ⊕ icon as a giant unstyled glyph
  filling the viewport (the 2020-era shared styles that sized
  `.roundbutton` images are gone; the app's own css is 19 lines). The
  network list is pushed off-screen below it. **Live-traced (screenshots).**
- **A2. Error detection is structurally wrong for every object-returning
  command.** The UI checks `result.status`, but for JSONObject returns the
  envelope is always `ok` and the real verdict rides `result.data.status`.
  Live consequence: on a joined network, Add Member with no peer selected
  ran `add_member` (which failed — no CA key), *did not notice*, and
  proceeded to fire `join_network` with undefined credentials, surfacing a
  raw `missing required parameter: ca_crt` JSON alert. `js:139-152`.
  **Live-traced.**
- **A3. Every outcome is an `alert()` of raw JSON.** Save configuration
  dumps the entire config object *including the full rendered YAML*;
  service toggles dump systemd stderr even when the operation half-worked
  ("Created symlink … Failed to connect to bus …"); join success is
  `alert(JSON.stringify(result))`. `js:149, 273, 302-322`. **Live-traced.**
- **A4. J3 dead-ends silently on a fresh instance.** The member popup's
  peer selector (`local:false, connectedonly:true`) renders an *empty*
  dropdown with no message, no link to the peer app, no explanation that
  you need a connected admin peer first — and OK proceeds anyway into the
  A2 cascade. **Live-traced.**
- **A5. Joined networks present owner-only affordances.** A network this
  instance merely joined (no `ca.key`) still shows Add Member, Add
  Lighthouse, and an empty members table; every one of those actions can
  only fail (and the failed probe left a stray empty `members/` dir).
  `info` already reports `owner`, so the UI has what it needs to gate
  these. **Live-traced.**

### B — misleading behavior and dead ends

- **B1. "Check for update" hangs forever when api.github.com is
  unreachable** — `$.getJSON` with no error handler leaves "*checking for
  update…*" up permanently (offline LAN, proxy, CORS). `js:397`.
  **Live-traced.**
- **B2. Dead and unlabeled chrome.** "Reboot Device" has no click handler
  at all (nothing in the facet references `rebootpeerbutton`). Escape does
  not close popups (**live-traced**); dialogs close only via a bare
  unlabeled ✕ image. The add-member dialog is titled "Send Credentials".
- **B3. Stale state after actions.** The Service/Running switches don't
  re-read `info` after a toggle — the UI keeps the pre-action state until
  you leave and re-enter the network page. The two switches also reuse
  hardcoded duplicate DOM ids (`appfilter-switch-1/2`) and their MDL
  markup renders with the label text overlapping the switch chrome
  (**live-traced, screenshot**).
- **B4. The remote-install flow no longer matches the packaging.** "Install
  App" bootstraps peers via `dev/install_lib`, which copies the store but
  knows nothing about FFI crates — after the overhaul, a peer installed
  that way has the UI but no working commands (no crate build, no
  initializer regen, no restart). Remote bootstrap needs to become
  `dev.github.import` on the target peer (or `install_lib` must learn FFI
  the way `import` did). This is a backend/UX decision, not a facet fix.
- **B5. Debris in the save path.** A live `debugger;` statement sits in the
  member-save flow (`js:146`) — the app freezes with devtools open. Line
  95 is a dead label statement (`closeselector: ".closelhbutton",` — cf.
  the *property* form at `js:176`), so the lighthouse popup's close
  wiring silently never attached.
- **B6. Overlay IPs are proposed by counting members + 2** (`js:103-109`).
  Remove a member and the next add re-proposes the same IP while the old
  certificate is still valid; no collision check against lighthouses or
  the host's own IP.
- **B7. The lighthouse table's DOM is the data store.** Save walks the
  table cells back out (`extractLightHouses`, `js:277-298`); deleting a
  row just removes the `<tr>`. Rendering *is* persistence — silent config
  loss is one mis-render away.

### C — polish

- **C1.** MDL-era markup running against a shimmed `componentHandler`;
  three styling dialects in one app; none of it uses the platform's
  tokens.css kit (kit v1 is a stable contract now).
- **C2.** No busy/progress affordances: release install, cert signing, and
  remote calls give no feedback until the alert lands; double-submit is
  possible everywhere.
- **C3.** Release names are parsed with fixed offsets
  (`o.name.substring(7, o.name.length - 7)`, `js:423`) — correct for
  `.tar.gz` assets, mangles the `.zip` (Windows) asset labels.

## 3. Verdict: toss the main control's facets, keep everything around them

Rebuild `nebula.nebula`'s html/css/js from scratch against the tokens.css
kit; redo the three small popup forms in the same pass; keep everything
else. Reasoning:

- **The command layer is sound and verified** — the UI is a thin shell
  over 14 commands. Nothing in the 464-line facet is worth preserving
  except the send-call graph, and §1 of this document *is* that graph.
- **The A-class defects are architecture, not warts.** Fixing the error
  contract (A2), the ownership split (A5), and DOM-as-data (B7) rewrites
  the file's structure anyway; patching around them would cost more than
  a clean rebuild and keep the debris.
- **The platform has moved.** tokens.css (kit v1, frozen names) and
  `app.ui`'s chrome are the current dialect; the MDL residue has no
  future and already renders broken (A1, B3).
- **Keep:** the four-control structure, the app shell mount, the
  `installControl` / `peer.peer_select` / `app.select` composition, and
  the journeys as designed — the *design* was never the problem.
- The popup forms (lighthouse/network/member ≈ 50 lines each) are
  salvageable but carry the same MDL markup; redoing them alongside costs
  little and buys one consistent dialect.

### Proposed shape (for review, not built)

Same four controls, rewritten facets:

- **Owner/joined mode split** from `info.owner`: joined networks show
  status + service controls only; owner networks get the full grow/manage
  surface.
- **A status strip instead of alerts**: every command outcome rendered
  from `data.status`/`data.msg` honestly — success is quiet, errors are
  readable, long operations show progress and disable their trigger.
- **Guided J3**: when no peers are connected, the Members tab says so and
  points at the peer app; the IP proposal checks existing members,
  lighthouses, and the host IP.
- **J1 resilience**: update check with a timeout and an offline message;
  release labels from the asset name split on the last `.tar.gz`/`.zip`.
- Dead chrome deleted (Reboot Device, or wired to `peer.reboot` if it is
  wanted — decision needed); Escape and ✕ both close every popup.
- Net size: roughly today's line count, most of it new; the three popup
  controls shrink.

### What a facet rewrite does not fix (schedule separately)

- **B4** — the remote bootstrap path (`install_lib` vs FFI) needs a
  platform-side decision.
- `install_service` returning `systemctl enable`'s stderr as noise even on
  success wants a command-side cleanup (return OK + a `detail` field).
- Cert lifecycle honesty (B6 fallout): removing a member should surface
  "certificate remains valid until expiry/CA rotation" in the UI — the
  command desc already says it; the UI must too.
