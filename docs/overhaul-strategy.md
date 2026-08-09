# Nebula Library Overhaul — Strategy

**Status: proposal, awaiting the owner's review** (2026-08-09). Nothing in
Phase 3 executes until he signs off. Facts marked **[verified]** were read
from source in this session (newbound master @ current, flowlang 0.3.31,
newbound-agent @ d05ea4a, this repo @ d0bcb0a).

## 1. What this library is today

A 2020–2023-era Newbound library (`data/nebula`, store version 42) that
manages [Slack's Nebula](https://github.com/slackhq/nebula) overlay network
across the peers you admin. Published as the "Nebula" app
(`runtime/nebula` + `data/nebula/_APPS/nebula`, byte-identical copies
**[verified]**).

Four controls:

| Control | Facets | Role |
|---|---|---|
| `nebula` | html/css/js + **all 14 commands** | The app UI: peer selector, release install, network list, network detail page |
| `lighthouse` | html/css/js | Popup form: add/edit a lighthouse (peer, private IP, public IP, port) |
| `network` | html/css/js | Popup form: create a network (name, subnet, port) |
| `member` | html/css/js | Popup form: add a member (peer, IP, groups) |

The 14 commands, end to end **[verified]**:

- **Release management** — `info` (installed version + networks list, read
  from `runtime/nebula/bin/version.txt` and `runtime/nebula/networks/*`);
  `install_release` (downloads a GitHub release tarball with `attohttpc`,
  `tar -xzf` into `runtime/nebula/bin`). The browser side hits
  `api.github.com/repos/slackhq/nebula/releases` directly to offer versions.
- **CA / certs** — `create_network` (runs `nebula-cert ca` + `nebula-cert
  sign` via `system_call`, then *renames the key/cert files out of the
  process working directory* into `runtime/nebula/networks/<name>/`);
  `add_member` (*copies `ca.key`/`ca.crt` into the working directory*,
  signs the member's cert, deletes the copies, returns `ca_crt` +
  `host_crt` + `host_key` as strings in the JSON reply).
- **Rollout** — `join_network` (writes the received certs/keys and a
  `connection.json`/`config.yml` on the target instance; invoked on the
  *remote* peer through the peer-proxy `send_*(…, target)` path);
  `members` (reads `networks/<name>/members/*/info.json`).
- **Config** — `save_config` and `build_config` (each contains its own
  copy of the full config.yml YAML template, ~40 duplicated lines);
  optional raw-YAML mode.
- **systemd** — `install_service` (renders `_ASSETS/service.txt`, `sudo mv`
  to `/etc/systemd/system`, daemon-reload, enable), `start_service`,
  `stop_service`, `uninstall_service` (all `sudo systemctl`),
  `restart_service` (odd one out: un-sudo'd legacy `service <name>
  restart`).
- **Relic** — `convert_legacy` is a stub returning `"NO"`.

The UI also carries a Metabot-era remote-app-install flow (`../peer/remote/
<peer>/dev/install_lib` then `app/settings` to activate) and two dead
buttons (`updateMember`/`deleteMember` → `alert("NO")`).

### What still works against today's platform

- The **store format** is current: records + sibling attachment files load
  fine; `load_library` reads this `meta.json` **[verified]**.
- The **UI contract** is current: `installControl`, generated `send_*`
  wrappers with a `target` peer argument, `peer.peer_select` and
  `app.select` all still exist **[verified]**.
- The **app contract** is current: `runtime/<app>/app.properties` is how
  apps are discovered; activation is the `apps` list in
  `config.properties`, written by `app/settings` **[verified]**.
- The **repo layout is exactly the `dev.github.import` contract**:
  `data/<lib>` + `runtime/<lib>` at the repo root **[verified]** (§3).
- The **security default is sane**: command records carry empty `readers`,
  so with security on, only `admin` (or explicitly granted groups) can
  execute them **[verified]** — right for commands that run `sudo`.

### What is broken or superseded

- **`meta.json` has no `root`** — today's builder routes the generated
  Rust into the shared, gitignored `cmd` crate: static, host-rebuild +
  restart on every change, no hot reload.
- **`src/nebula/**` is the pre-split generated tree** — nothing consumes
  that location anymore; it is dead weight in the repo.
- **Java-era relics throughout**: the control's `cmd` list still carries
  `"java"` source pointers next to the live `"rust"` ones; old Java source
  records (one with a `.java` attachment) sit beside the `.rs` ones;
  `app.properties` says `botclass=com.newbound.robot.published.Nebula`,
  `generate=html,java`, with a 2020-format signature.
- **Key hygiene warts**: `create_network` and `add_member` stage
  `ca.key`/certs through the *newbound working directory* — a crash
  mid-command leaves a CA key sitting in the checkout root.
- **Blank `desc` on everything** — the control and all 14 commands are
  invisible to MCP `tools/list` and to any agent browsing the store.
- **`attohttpc 0.24`** — the library's only third-party dependency, used
  once, for a file download.
- The **README install** ("hand-move folders, activate in Metabot,
  restart") predates `dev.github.import` and names an app that no longer
  exists.
- No `_patches` journals (pre-journal store) — they will accrete naturally
  once modern `dev.code` writes begin.

## 2. Target install story

**Install** (non-expert path, from the Dev app's shelf → git-import panel,
or the same command over MCP):

1. `dev.github.import` with `https://github.com/mraiser/newbound_nebula`.
   The platform clones into `repositories/nebula`, symlinks `data/nebula`
   and `runtime/nebula` into the instance, `load_library`s it, and kicks
   `rebuild_lib` **[verified]**.
2. One restart. A brand-new FFI root needs the regenerated initializer
   (`newbound rebuild` + host build) before the watcher knows the crate
   **[verified: watcher list and init blocks are baked at rebuild]** — so
   the honest story is *import → rebuild → restart once*. After that,
   nebula command changes hot-reload like agent/kb.
3. Activate the app (the standard apps toggle → `app/settings` →
   `config.properties`), select it, done.

**Update**: `git pull` in `repositories/nebula` + `rebuild_lib`. There is
no `dev.github.update` command today — only `import` and `list`
**[verified]**. Proposed (owner's call, §6): add `update` (and `remove`)
to `dev.github` as a small platform-side branch — it benefits every
imported library, which is the tighter-integration direction the doctrine
wants. Fallback if declined: document the two manual steps in the README.

**Uninstall**: stop/uninstall any services from the UI, deactivate the
app, then remove the two symlinks and `repositories/nebula`
(`dev.github.remove` if approved; documented manual steps otherwise).
`runtime/nebula/networks/` — the user's keys — lives inside
`repositories/nebula`, so removal must warn before deleting a directory
that still contains networks.

## 3. Packaging pattern: the newbound-agent pattern

`meta.json` gains `"root": "nebula"` and
`"cargo": {"crate_types": ["dylib"], "ffi": true, "dependencies": {}}`,
and a real `desc`. Consequences:

- The builder generates a self-contained **`nebula/` FFI dylib crate**;
  the hot-reload watcher picks it up after one initializer regen. Same
  lifecycle as `agent`/`kb` **[verified against rebuild_lib's FFI path]**.
- The generated crate is **tracked in this repo** (like `agent/` in
  newbound-agent) so the Rust is reviewable as ordinary source. On an
  installed instance the crate is *regenerated from the store* by
  `rebuild` — the tracked copy is provenance/review, not the install
  artifact, and the two cannot drift for users.
- The old `src/` tree is deleted; this repo stops pretending to be a
  pre-split checkout.
- `data/nebula/_APPS/nebula` stays (it is the peer-to-peer
  `dev/install_lib` install contract **[verified]**); `runtime/nebula`
  stays (github-import contract + app discovery). Both are regenerated by
  a modern `publishapp` run, which also replaces the Java-era
  `app.properties` fields with current ones.

Why not the rootless/`cmd` route (today's default for this meta.json): it
compiles nebula into the host — every code change is a host rebuild +
restart, dependencies land in the shared crate, and nothing hot-reloads.
The FFI pattern is what "drop-in library" means on today's platform, and
it is the pattern the task's reference (newbound-agent) established.

## 4. Migrate / rewrite / delete (the codebase shrinks)

**Migrate as-is** (regenerated wrappers, modern param-guard builder; body
logic unchanged): `info`, `members`, `join_network`, `install_service`,
`start_service`, `stop_service`, `uninstall_service`.

**Rewrite, small and targeted:**

- `create_network` / `add_member`: use `nebula-cert`'s `-out-crt`,
  `-out-key`, `-ca-crt`, `-ca-key` flags so keys are written where they
  belong and the CA key never transits the working directory. Kills the
  worst hygiene wart with a net *reduction* in code (no copy/rename/delete
  dance).
- `save_config` / `build_config`: one shared YAML-template function;
  `build_config` becomes a thin read-and-render. Removes ~40 duplicated
  lines.
- `install_release`: replace `attohttpc` with a `system_call` download
  (`curl -fsSL -o`), the same pattern the platform already uses for `git`,
  `zip`, `unzip`, and `tar` **[verified]**. Empties the dependency list —
  doctrine: no third-party code where a narrow local alternative exists.
- `restart_service`: `sudo systemctl restart`, consistent with its
  siblings.
- `service.txt` unit template: `Type=simple` with a direct `ExecStart`
  (drop the `bash -c '… &'` + `Type=forking` + `RemainAfterExit` hack);
  add `Restart=on-failure`, `After=network-online.target`.
- UI: keep the existing jquery/`installControl` architecture (it is still
  the platform's own pattern — restyling is out of scope and would grow
  the diff, not shrink it). Fix or remove the two dead buttons
  (recommend: implement `deleteMember` — cert removal on the admin side is
  just deleting `members/<peer>/` — and drop `updateMember`), update the
  Metabot-era wording, and re-test the remote-install flow.
- Fill `desc` on the library, all four controls, and all 14 commands —
  desc is discovery.

**Delete:**

- `src/**` (pre-split generated tree; superseded by the tracked `nebula/`
  crate).
- `convert_legacy` (stub, returns `"NO"`).
- All Java-era source records and the `.java` attachment; the stale
  `"java"` pointers drop out of the control record as each command is
  re-saved through `dev.code`.
- The `attohttpc` dependency.
- README's hand-install instructions (replaced by the import story).

Net: one command fewer, one dependency fewer, ~hundreds of lines of dead
tree and duplicated template gone, against a small amount of new code
(desc strings, the shared template fn, optionally `dev.github.update`/
`remove` platform-side).

## 5. Key & cert security posture

- **Nothing secret is ever committed.** The repo's `.gitignore` already
  excludes `runtime/nebula/{bin,html,networks,botd.properties}`
  **[verified]** — `networks/` is where every private key lives. That
  stays, and the overhaul adds nothing secret-shaped anywhere tracked.
  (The `key=` field in `app.properties` is the platform's app-publish
  signing construct, present in the platform's own committed apps — not an
  instance secret. It gets regenerated in the modern format by
  `publishapp`.)
- **`runtime/` is the home for local values** — network configs,
  certs, keys, the nebula binary — none of it store data, none of it
  canon. On an imported instance all of it lives under
  `repositories/nebula/runtime/nebula/`, covered by this repo's own
  `.gitignore`, so even a user who pokes git in that clone can't
  accidentally stage a key.
- **The CA key never leaves the owning instance.** Only member certs and
  member keys travel, and only through the already-encrypted Newbound
  peer channel as `join_network` parameters (unchanged design, now
  documented). The cert-flag rewrite (§4) removes the one place the CA
  key touched a directory outside `networks/`.
- **Command exposure stays admin-only** (empty `readers` on every
  command record; these commands run `sudo`). The overhaul keeps that and
  states it in each `desc`.

## 6. What gets verified here vs. on your hardware

Per `kb.workflow`: a capability that can only fail at a service the
sandbox lacks gets a **wire-shape check**, not a fake end-to-end claim.

**Verifiable in this sandbox (disposable instance):**

- The full import story from a local clone URL: import → symlinks →
  `rebuild` → host build → dylib build → commands present in MCP
  `tools/list` *with descs* → UI serves.
- `install_release` against a real GitHub release (network permitting;
  otherwise against a local tarball URL — same wire shape).
- The whole cert lifecycle, which needs no tun device: `create_network`,
  `add_member`, `members`, `info` — real `nebula-cert`, real files,
  asserted against `networks/<name>/` layout.
- `config.yml` generation as golden-text comparison (both the structured
  and raw-YAML paths).
- The rendered systemd unit as golden text, and the exact
  `systemctl`/`sudo` argv each service command issues (asserted as data,
  not executed — the sandbox has no systemd and no sudo).

**Yours only (I will not claim these):** actual service
install/start/stop on systemd, tun device creation, real overlay traffic
between peers, lighthouse reachability from outside, the multi-peer
rollout UI against live remote instances.

## 7. Execution plan (Phase 3, after your go-ahead)

1. Disposable copy of the overlaid checkout (per
   `tools/scratch-instance.md`); all store edits through `dev.code`
   commands over `newbound mcp` — never by hand.
2. Store surgery in order: meta.json root/cargo/desc (via `libsettings`
   path or the sanctioned command for lib meta), delete relic records,
   re-save the 14 commands with rewritten bodies + descs +
   `set_command_meta`, control descs, asset/service.txt update.
3. `rebuild` → tracked `nebula/` crate lands; delete `src/`; verify per
   §6 on the disposable.
4. Republish the app on the disposable (`publishapp`) to regenerate
   `_APPS` + `app.properties` in modern format.
5. Copy the resulting `data/nebula` + `runtime/nebula` + `nebula/` back
   into this repo **as one commit** (store + generated src together),
   README rewrite riding along; push this branch.
6. Platform-side (only if approved): `dev.github.update`/`remove` on a
   `mraiser/newbound` branch, same discipline.
7. `dev-code-remember` deposits into a new `kb.nebula` domain along the
   way (committed on a newbound-agent branch).

Nothing merges to master anywhere without your express permission.

## 8. Decisions I need from you

1. **Platform additions**: add `update` + `remove` to `dev.github`
   (small, benefits all imported libraries)? Or keep this overhaul
   strictly library-side and document the manual steps?
2. **The P2P remote-install button**: keep the `dev/install_lib`-based
   "install app on this peer" flow in the UI (it still works and is the
   only path for peers without the git repo), or lean everything on
   github-import per peer?
3. **`attohttpc` → `curl` via `system_call`**: confirm you're happy
   trading the crate for a runtime dependency on curl being installed.
4. **UI scope**: confirm minimal-touch (keep jquery/MDL era styling,
   fix dead buttons and wording only).
