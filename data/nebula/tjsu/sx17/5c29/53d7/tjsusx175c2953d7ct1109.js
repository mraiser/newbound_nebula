var me = this;
var ME = $('#'+me.UUID)[0];

// All state lives here — never parsed back out of the DOM.
var S = {
  peer: null,          // selected target peer uuid, or null for local
  mypeerid: null,
  mypeername: null,
  networks: {},        // name -> {name, owner, service, running, config}
  selected: null,      // name of the open network
  lighthouses: {},     // working copy for the open network: peerid -> {private_ip, public_ip, port}
  members: {},
  releases: null,
  info: null,          // last info answer for the selected instance (tun, http_address, binary_name)
  platform: null,      // this instance's release platform, e.g. 'linux-amd64' (#16)
  dirty: false,        // unsaved configuration edits on the open network (#15)
  runfail: null        // the last failed Run now answer, for the fix-it callout (#2)
};

var IS_MAC = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
$(ME).find('.savekbd').text(IS_MAC ? '\u2318S' : 'Ctrl+S');
$(ME).find('.saveconfigbutton').attr('title', 'Save configuration (' + (IS_MAC ? '\u2318S' : 'Ctrl+S') + ')');

// ---- outcomes (the envelope is 'ok' even when the command failed:
// object commands carry the verdict in data.status, string commands in
// an ERROR prefix — check all three) ----
function outcome(result){
  if (!result) return { ok:false, msg:'no response' };
  if (result.status != 'ok') return { ok:false, msg: result.msg || 'request failed' };
  var d = result.data;
  if (d && typeof d == 'object' && d.status == 'err') return { ok:false, msg: d.msg || 'command failed' };
  if (typeof d == 'string' && d.indexOf('ERROR') == 0) return { ok:false, msg: d };
  return { ok:true, msg: (typeof d == 'string') ? d : '' };
}

// ---- status strip: quiet success, readable errors, sticky until dismissed ----
var statusTimer = null;
function status(msg, kind){
  var el = $(ME).find('.nb-status');
  if (statusTimer) { clearTimeout(statusTimer); statusTimer = null; }
  if (!msg) { el.attr('hidden', true); return; }
  el.removeAttr('hidden').removeClass('ok err busy').addClass(kind || 'ok');
  el.find('.nb-status-msg').text(msg);
  if (kind == 'ok') statusTimer = setTimeout(function(){ el.attr('hidden', true); }, 4000);
}
$(ME).find('.nb-status-x').click(function(){ status(null); });

function busy(sel, on){
  var el = $(ME).find(sel);
  if (on) el.attr('disabled', true).addClass('busy');
  else el.removeAttr('disabled').removeClass('busy');
}

// ---- boot ----
me.ready = function(){
  json('../peer/info', null, function(result){
    S.mypeerid = result.data.uuid;
    S.mypeername = result.data.name;
    installControl($(ME).find('.whichpeer')[0], 'peer', 'peer_select', function(){}, {
      local: true,
      connectedonly: true,
      ready: refresh,
      cb: function(val){
        S.peer = (val == 'local') ? null : val;
        showHome();
        refresh();
      }
    });
  });
};

function peerName(id){
  if (id == null || id == 'local') return 'local';
  if (id == S.mypeerid) return S.mypeername;
  return (document.peers && document.peers[id]) ? document.peers[id].name : id;
}

// ---- home: engine + network list ----
function refresh(){
  $(ME).find('.nebulaversion').text('checking…');
  $(ME).find('.engine-remote').attr('hidden', true);
  send_info(function(result){
    var o = outcome(result);
    if (!o.ok || !result.data || !result.data.tag_name){
      $(ME).find('.nebulaversion').text('unavailable');
      $(ME).find('.installednetworks').attr('hidden', true);
      // a remote peer without the app gets the in-app P2P install path
      if (S.peer) $(ME).find('.engine-remote').removeAttr('hidden');
      else if (!o.ok) status('This instance did not answer: ' + o.msg, 'err');
      return;
    }
    var d = result.data;
    S.info = d;
    S.platform = thisPlatform(d);
    S.systemd = !!d.systemd;
    var noengine = d.tag_name == 'Not Installed';
    if (noengine){
      $(ME).find('.nebulaversion').text('not installed — fetch a release below');
    } else {
      $(ME).find('.nebulaversion').text(d.tag_name + ' (' + d.binary_name + ')');
    }
    // no engine, no certificates: gate network creation until it is installed
    $(ME).find('.addnetworkbutton').prop('disabled', noengine);
    if (noengine) $(ME).find('.noengine-hint').removeAttr('hidden');
    else $(ME).find('.noengine-hint').attr('hidden', true);
    renderNetworks(d.networks || []);
    $(ME).find('.installednetworks').removeAttr('hidden');
  }, S.peer);
}
$(ME).find('.refreshversion').click(refresh);

function renderNetworks(list){
  S.networks = {};
  var box = $(ME).find('.foundnetworks').empty();
  if (!list.length){
    box.append($('<div class="nb-empty">No networks yet.</div>'));
    return;
  }
  for (var i in list){
    var n = list[i];
    S.networks[n.name] = n;
    var owned = n.owner == 'local';
    var row = $('<div class="netrow"></div>').attr('data-name', n.name);
    row.append($('<span class="dot"></span>').toggleClass('on', !!n.running));
    row.append($('<span class="netname"></span>').text(n.name));
    row.append($('<span class="badge"></span>').toggleClass('owner', owned)
      .text(owned ? 'owner' : 'joined · ' + peerName(n.owner)));
    if (n.service) row.append($('<span class="badge">service</span>'));
    row.click(function(){ openNetwork($(this).attr('data-name')); });
    box.append(row);
  }
}

// ---- engine: release check + install ----
// Release listing prefers the server (no browser CORS or connectivity
// assumptions); falls back to fetching from the browser, then to a URL box.
// This instance's release platform (#16): the one in the installed tarball's
// name, else derived from the OS/arch the instance reports. Never a guess
// from the browser - the engine runs on the instance, not here.
function thisPlatform(d){
  var m = (d.binary_name || '').match(/^nebula-(.+?)\.(tar\.gz|zip)$/);
  if (m) return m[1];
  var t = d.tun || {};
  var arch = { x86_64: 'amd64', aarch64: 'arm64', arm: 'arm-7', x86: '386', riscv64: 'riscv64', mips: 'mips', mips64: 'mips64' }[t.arch] || t.arch;
  if (t.os == 'macos') return 'darwin';
  if (t.os && arch) return t.os + '-' + arch;
  return null;
}

function fillSelect(sel, list, value){
  sel.empty();
  for (var i in list) sel.append($('<option></option>').attr('value', list[i].id).text(list[i].name));
  if (value != null) sel.val(value);
}

function offerReleases(list){
  S.releases = {};
  for (var i in list) S.releases[list[i].id] = list[i];
  fillSelect($(ME).find('.sel-version'), list, list.length ? list[0].id : null);
  $(ME).find('.releasefield').removeAttr('hidden');
  if (list.length) selectVersion(list[0].id);
}
$(ME).find('.sel-version').change(function(){ selectVersion($(this).val()); });
$(ME).find('.sel-platform').change(checkPlatform);

// Warn (#16) when the chosen build is not this machine's.
function machineName(){ return S.peer ? peerName(S.peer) : 'this machine'; }
function platformMismatch(name){
  return !!(S.platform && name && name != S.platform);
}
function checkPlatform(){
  var name = $(ME).find('.sel-platform option:selected').text();
  var w = $(ME).find('.platformwarn');
  if (platformMismatch(name)) w.removeAttr('hidden').text('This is the ' + name + ' build, but ' + machineName() + ' is ' + S.platform + '. It won\u2019t run there.');
  else w.attr('hidden', true).text('');
}

$(ME).find('.checkupdatebutton').click(function(){
  var msg = $(ME).find('.updatemsg');
  $(ME).find('.engine-install').removeAttr('hidden');
  $(ME).find('.engine-byurl').removeAttr('hidden');
  msg.text('checking releases…');
  busy('.checkupdatebutton', true);
  send_releases(function(result){
    var o = outcome(result);
    if (o.ok && result.data.list && result.data.list.length){
      busy('.checkupdatebutton', false);
      msg.text('');
      var list = [];
      for (var i in result.data.list){
        var r = result.data.list[i];
        if (r.prerelease) continue;
        list.push({ id: r.tag_name, name: r.tag_name, assets: r.assets });
      }
      offerReleases(list);
      return;
    }
    // the instance could not reach GitHub — try from this browser instead
    var ctrl = new AbortController();
    var timer = setTimeout(function(){ ctrl.abort(); }, 10000);
    fetch('https://api.github.com/repos/slackhq/nebula/releases', { signal: ctrl.signal })
      .then(function(r){ if (!r.ok) throw new Error('HTTP ' + r.status); return r.json(); })
      .then(function(raw){
        clearTimeout(timer);
        busy('.checkupdatebutton', false);
        msg.text('');
        var list = [];
        for (var i in raw){
          if (raw[i].prerelease) continue;
          var assets = [];
          for (var j in raw[i].assets) assets.push({ name: raw[i].assets[j].name, url: raw[i].assets[j].browser_download_url });
          list.push({ id: '' + raw[i].id, name: raw[i].tag_name, assets: assets });
        }
        offerReleases(list);
      })
      .catch(function(e){
        clearTimeout(timer);
        busy('.checkupdatebutton', false);
        msg.text('');
        status('Could not reach api.github.com from this instance or this browser (' + e.message + '). Paste a release tarball URL below instead.', 'err');
      });
  }, S.peer);
});

function selectVersion(id){
  var r = S.releases[id];
  var binaries = [];
  for (var i in r.assets){
    var a = r.assets[i];
    var m = a.name.match(/^nebula-(.+?)\.(tar\.gz|zip)$/);
    if (m) binaries.push({ name: m[1], id: a.url });
  }
  // preselect this machine's build (#16); asset order put darwin first
  var pick = null;
  for (var j in binaries) if (binaries[j].name == S.platform) pick = binaries[j].id;
  if (!pick && binaries.length) pick = binaries[0].id;
  fillSelect($(ME).find('.sel-platform'), binaries, pick);
  $(ME).find('.installupdatebutton').removeAttr('hidden');
  checkPlatform();
}

function doInstall(url, version){
  busy('.installupdatebutton', true);
  busy('.installurlbutton', true);
  status('Downloading ' + version + ' on ' + peerName(S.peer) + '…', 'busy');
  send_install_release(url, version, function(result){
    busy('.installupdatebutton', false);
    busy('.installurlbutton', false);
    var o = outcome(result);
    if (!o.ok){ status(o.msg, 'err'); refresh(); return; }
    // A new engine binary loses the Linux network permission (#2): re-check.
    send_info(function(r2){
      var t = (r2 && r2.data && r2.data.tun) || {};
      if (t.ready === false && t.fix && t.os == 'linux')
        status('Installed ' + version + '. The new engine needs its network permission again before Run now works: ' + t.fix, 'err');
      else status('Installed ' + version, 'ok');
      refresh();
    }, S.peer);
  }, S.peer);
}

$(ME).find('.installupdatebutton').click(function(){
  var url = $(ME).find('.sel-platform').val();
  var name = $(ME).find('.sel-platform option:selected').text();
  var version = $(ME).find('.sel-version option:selected').text();
  if (!url) return status('Pick a platform first.', 'err');
  if (platformMismatch(name) && !confirm('Install the ' + name + ' build on ' + machineName() + '?\n\nThat machine is ' + S.platform + ', so this engine won\u2019t run there and Run now will fail until you install the right one.')) return;
  doInstall(url, version);
});

$(ME).find('.installurlbutton').click(function(){
  var url = ($(ME).find('.f-releaseurl').val() || '').trim();
  if (!url) return status('Paste the URL of a nebula release tarball first.', 'err');
  var m = url.match(/nebula-(.+?)\.(tar\.gz|zip)$/);
  if (m && platformMismatch(m[1]) && !confirm('That URL is the ' + m[1] + ' build, but ' + machineName() + ' is ' + S.platform + '. Install it anyway?\n\nIt won\u2019t run there.')) return;
  doInstall(url, m ? ('custom (' + m[1] + ')') : 'custom URL');
});

// ---- P2P install: stream the library to a peer that lacks the app ----
$(ME).find('.installappbutton').click(function(){
  if (!S.peer) return;
  busy('.installappbutton', true);
  status('Streaming the Nebula library to ' + peerName(S.peer) + ' and building it there — this can take a few minutes…', 'busy');
  var d = 'uuid=' + S.mypeerid + '&lib=nebula';
  json('../peer/remote/' + S.peer + '/dev/install_lib', d, function(result){
    busy('.installappbutton', false);
    var o = outcome(result);
    if (!o.ok){ status('Install failed: ' + o.msg, 'err'); return; }
    // activate the app in the peer's config, then one restart finishes it
    json('../peer/remote/' + S.peer + '/app/settings', 'settings={}', function(result2){
      var o2 = outcome(result2);
      if (!o2.ok){ status('Installed, but could not read the peer\'s app settings: ' + o2.msg, 'err'); return; }
      var applist = result2.data.apps || '';
      if (applist.split(',').indexOf('nebula') == -1){
        applist = applist == '' ? 'nebula' : applist + ',nebula';
      }
      json('../peer/remote/' + S.peer + '/app/settings', 'settings=' + encodeURIComponent(JSON.stringify({ apps: applist })), function(result3){
        var o3 = outcome(result3);
        if (!o3.ok){ status('Installed, but activation failed: ' + o3.msg, 'err'); return; }
        status('Installed and activated on ' + peerName(S.peer) + '. Reboot that device once to finish — the first install builds its hot-reload crate.', 'ok');
        $(ME).find('.rebootpeerbutton').removeAttr('hidden');
      });
    });
  });
});

$(ME).find('.rebootpeerbutton').click(function(){
  if (!S.peer) return;
  if (!confirm('Reboot the device running ' + peerName(S.peer) + '?')) return;
  json('../peer/remote/' + S.peer + '/peer/reboot', null, function(){
    status('Reboot signal sent to ' + peerName(S.peer) + '. Refresh here once it is back.', 'ok');
    $(ME).find('.rebootpeerbutton').attr('hidden', true);
  });
});

// ---- create network ----
// Propose values no network on this instance uses yet (#8) - across every
// network name, not just the nebulaN ones this dialog made itself.
function usedBy(field){
  var used = {};
  for (var k in S.networks){
    var c = S.networks[k].config || {};
    if (c[field]) used['' + c[field]] = k;
  }
  return used;
}
$(ME).find('.addnetworkbutton').click(function(){
  var n = 1;
  while (S.networks['nebula' + n]) n++;
  var ports = usedBy('port'), port = 4242;
  while (ports['' + port]) port++;
  var subnets = usedBy('subnet'), net = 100;
  while (subnets['192.168.' + net + '.X/24'] && net < 254) net++;
  openModal('network', {
    value: { name: 'nebula' + n, subnet: '192.168.' + net + '.X/24', port: '' + port },
    usedPorts: ports,
    // is the port bound on the instance's machine (by anything)?
    checkPort: function(p, cb){ send_port_status('' + p, function(r){ var o = outcome(r); cb(o.ok ? r.data : null); }, S.peer); },
    close: closeModal,
    save: function(v){
      closeModal();
      status('Creating ' + v.name + '…', 'busy');
      send_create_network(v.name, v.subnet, '' + v.port, function(result){
        var o = outcome(result);
        status(o.ok ? 'Network ' + v.name + ' created' : o.msg, o.ok ? 'ok' : 'err');
        refresh();
      }, S.peer);
    }
  });
});

// ---- network page ----
function showHome(){
  $(ME).find('.networkpage').attr('hidden', true);
  $(ME).find('.homepage').removeAttr('hidden');
  $(ME).find('.nb-crumb').attr('hidden', true);
  $(ME).find('.nb-back').attr('hidden', true);
  $(ME).find('.nb-home').removeAttr('hidden');
  S.selected = null;
  setDirty(false);
}

// Back (#15): the Dev editors' pattern - "\u2190" top-left, Esc does the same.
function goBack(){
  if (S.dirty && !confirm('Discard the unsaved changes to ' + S.selected + '?')) return;
  showHome();
  refresh();
}
$(ME).find('.nb-back').click(goBack);

// Unsaved-changes state for Save (#15), like Dev's "no control changes".
function setDirty(on){
  S.dirty = !!on;
  $(ME).find('.saveconfigbutton').prop('disabled', !S.dirty);
  $(ME).find('.nb-dirty').toggleClass('dirty', S.dirty).text(S.dirty ? 'unsaved changes' : 'no changes');
}
$(ME).find('.nb-form input, .f-useyaml, .f-yaml').on('input change', function(){ setDirty(true); });

$(ME).find('.nb-tab').click(function(){
  $(ME).find('.nb-tab').removeClass('selected');
  $(this).addClass('selected');
  $(ME).find('.nb-tabpane').attr('hidden', true);
  $(ME).find('.pane-' + $(this).data('tab')).removeAttr('hidden');
});

function openNetwork(name){
  var n = S.networks[name];
  if (!n) return;
  S.selected = name;
  S.lighthouses = JSON.parse(JSON.stringify(n.config.lighthouses || {}));
  var owned = n.owner == 'local';

  $(ME).find('.homepage').attr('hidden', true);
  $(ME).find('.networkpage').removeAttr('hidden');
  $(ME).find('.nb-home').attr('hidden', true);
  $(ME).find('.nb-back').removeAttr('hidden');
  S.runfail = null;
  $(ME).find('.nb-crumb').removeAttr('hidden')
    .html('/ ' + $('<i>').text(peerName(S.peer)).html() + ' / <b>' + $('<i>').text(name).html() + '</b>');
  $(ME).find('.networkowner').text(owned ? 'this instance' : peerName(n.owner));
  if (owned) $(ME).find('.joined-note').attr('hidden', true);
  else $(ME).find('.joined-note').removeAttr('hidden');
  if (owned) $(ME).find('.tab-members').removeAttr('hidden');
  else $(ME).find('.tab-members').attr('hidden', true);
  if (!owned && $(ME).find('.tab-members').hasClass('selected')) $(ME).find('.nb-tab[data-tab=svc]').click();

  renderServicePane(n);
  renderLighthouses();
  if (owned) loadMembers();

  $(ME).find('.f-useyaml').prop('checked', !!n.config.use_yaml);
  $(ME).find('.f-yaml').val(n.config.yaml || '');
  $(ME).find('.nb-tab[data-tab=svc]').click();
  setDirty(false);
}

// ---- certificate lifetimes (#5) ----
function fmtDate(d){ return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }); }
function renderExpiry(n){
  var ca = n.ca_not_after ? new Date(n.ca_not_after) : null;
  var host = n.host_not_after ? new Date(n.host_not_after) : null;
  if (ca && isNaN(ca)) ca = null;
  if (host && isNaN(host)) host = null;
  var line = $(ME).find('.expiryline'), banner = $(ME).find('.certwarn');
  if (!ca && !host){ line.attr('hidden', true); banner.attr('hidden', true); return; }
  // the network stops working at whichever comes first
  var first = (ca && host) ? (host < ca ? host : ca) : (ca || host);
  // host certs end 1 second before the CA by default: only blame the host
  // cert when it ends noticeably (a day+) earlier than the CA
  var hostFirst = host && (!ca || (ca - host) > 86400000);
  var which = hostFirst ? 'This host\u2019s certificate' : 'The network\u2019s CA';
  var txt = fmtDate(first);
  if (ca && host && Math.abs(ca - host) > 86400000) txt += ' (this host) \u00b7 CA ' + fmtDate(ca);
  line.removeAttr('hidden').find('.certexpiry').text(txt);
  var left = first - Date.now();
  var days = Math.round(left / 86400000);
  if (left < 0){
    banner.removeAttr('hidden').addClass('expired').text(which + ' expired on ' + fmtDate(first) + '. Members can\u2019t connect until the network is recreated (the app can\u2019t renew certificates yet).');
  } else if (days <= 30){
    banner.removeAttr('hidden').removeClass('expired').text(which + ' expires on ' + fmtDate(first) + ' \u2014 ' + (days < 1 ? 'today' : 'in ' + days + (days == 1 ? ' day' : ' days')) + '. After that members can\u2019t connect. The app can\u2019t renew certificates yet, so plan to recreate the network before then.');
  } else banner.attr('hidden', true).removeClass('expired');
}

// ---- Run now help (#2) and the web-UI reachability note (#9) ----
function runHint(t){
  var base = 'The tunnel runs as a child process of Newbound \u2014 no systemd needed. ';
  switch (t.os){
    case 'linux':
      if (t.ready === true) return base + 'The engine already has the network permission it needs, so no sudo either.';
      if (t.ready === false) return base + 'On Linux the engine needs a one-time network permission first (one sudo command, below).';
      return base + 'On Linux the engine may need a one-time network permission; if Run now fails, it shows the command.';
    case 'macos':
      return t.ready === true ? base + 'Newbound runs as root here, so it can create the tunnel.'
        : base + 'But macOS lets only root create the tunnel, so Run now works only when Newbound runs as root.';
    case 'windows':
      return t.ready === true ? base + 'Newbound runs as administrator, so it can create the tunnel.'
        : base + 'But Windows lets only an administrator create the tunnel, so Run now needs Newbound started with \u201cRun as administrator\u201d.';
    default:
      return base + 'Creating the tunnel usually needs root on this system.';
  }
}
function renderRunHelp(n){
  var t = (S.runfail && S.runfail.tun) || (S.info && S.info.tun) || {};
  $(ME).find('.runhint').text(runHint(t));
  var box = $(ME).find('.runfix');
  // show the fix after a failed start, or up front when we already know it's needed
  var show = !n.running && (S.runfail || t.ready === false);
  if (!show){ box.attr('hidden', true); return; }
  if (S.runfail && S.runfail.reason && S.runfail.reason != 'tun_permission'){
    box.find('.runfix-how').text(S.runfail.msg);
    box.find('.nb-cmd').attr('hidden', true);
    box.find('.runfix-note').text('');
  } else {
    box.find('.runfix-how').text(t.how || 'This computer didn\u2019t let the nebula engine create its network interface.');
    if (t.fix) { box.find('.nb-cmd').removeAttr('hidden'); box.find('.runfix-cmd').text(t.fix); }
    else box.find('.nb-cmd').attr('hidden', true);
    box.find('.runfix-note').text(t.note || '');
  }
  var log = S.runfail && S.runfail.details;
  if (log){ box.find('.runfix-details').removeAttr('hidden'); box.find('.runfix-log').text(log); }
  else box.find('.runfix-details').attr('hidden', true);
  box.removeAttr('hidden');
}
// Copy text and flash the button; plain-HTTP pages have no async clipboard,
// so fall back to select + execCommand.
function copyText(txt, btn){
  var label = btn.text();
  function done(){ btn.text('Copied'); setTimeout(function(){ btn.text(label); }, 1500); }
  function fallback(){
    var ta = $('<textarea class="nb-offscreen"></textarea>').val(txt).appendTo(ME);
    ta[0].select();
    var ok = false;
    try { ok = document.execCommand('copy'); } catch (x) {}
    ta.remove();
    if (ok) done(); else status('This browser blocked copying. Select the text and copy it by hand.', 'err');
  }
  if (navigator.clipboard && window.isSecureContext) navigator.clipboard.writeText(txt).then(done, fallback);
  else fallback();
}
$(ME).find('.copyfix').click(function(){ copyText($(ME).find('.runfix-cmd').text(), $(this)); });

function isLoopback(a){ return /^(127\.|localhost$|::1$|\[::1\]$)/.test((a || '').trim()); }
function renderLoopback(){
  var a = S.info && S.info.http_address;
  var el = $(ME).find('.loopback-hint');
  if (!isLoopback(a)){ el.attr('hidden', true); return; }
  var where = S.peer ? peerName(S.peer) + '\u2019s' : 'this';
  el.removeAttr('hidden').text('Phones and other members can\u2019t open Newbound\u2019s web page over this network: it only listens on ' +
    where + ' computer (http_address=' + a + ' in config.properties). Setting http_address=0.0.0.0 and restarting Newbound ' +
    'would change that \u2014 it\u2019s a platform setting, so this app only reports it.');
}

function renderServicePane(n){
  var c = n.config;
  var st = $(ME).find('.tunnelstate');
  st.find('.dot').toggleClass('on', !!n.running);
  st.find('.tunnelstate-msg').text(
    n.supervised ? 'tunnel running (supervised)'
    : n.service_running ? 'tunnel running (systemd)'
    : 'tunnel not running');
  if (n.supervised) { $(ME).find('.svc-runnow').attr('hidden', true); $(ME).find('.svc-stopnow').removeAttr('hidden'); }
  else { $(ME).find('.svc-runnow').removeAttr('hidden'); $(ME).find('.svc-stopnow').attr('hidden', true); }
  $(ME).find('.svc-boot').prop('checked', !!n.boot);
  // systemd controls only where systemd exists — elsewhere they can only fail
  if (S.systemd) $(ME).find('.systemd-only').removeAttr('hidden');
  else $(ME).find('.systemd-only').attr('hidden', true);
  $(ME).find('.svc-installed').prop('checked', !!n.service);
  $(ME).find('.svc-running').prop('checked', !!n.service_running);
  $(ME).find('.f-amlighthouse').prop('checked', !!c.am_lighthouse);
  $(ME).find('.f-publicaddr').val(c.public_address || '');
  togglePublicAddr();
  $(ME).find('.f-host').val(c.host || '');
  $(ME).find('.f-port').val(c.port || '');
  $(ME).find('.f-subnet').val(c.subnet || '');
  $(ME).find('.f-ipaddr').val(c.ip_address || '');
  $(ME).find('.f-groups').val(c.groups || '');
  renderExpiry(n);
  renderFirewall(n);
  renderUfw(n);
  renderRunHelp(n);
  renderLoopback();
}

// After any service action, re-read reality instead of trusting the click.
function refreshNetworkPage(){
  var name = S.selected;
  send_info(function(result){
    var o = outcome(result);
    if (o.ok && result.data && result.data.networks){
      S.info = result.data;
      renderNetworks(result.data.networks);
      if (name && S.networks[name]){
        var n = S.networks[name];
        S.selected = name;
        renderServicePane(n);
      }
    }
  }, S.peer);
}

function serviceAction(sendFn, label){
  busy('.nb-switchrow .btn', true);
  $(ME).find('.nb-switch').addClass('busy');
  status(label + '…', 'busy');
  sendFn(S.selected, function(result){
    busy('.nb-switchrow .btn', false);
    $(ME).find('.nb-switch').removeClass('busy');
    var o = outcome(result);
    // service commands answer OK or pass through the tool's own words
    status(o.ok ? (o.msg == 'OK' ? label + ': done' : o.msg) : o.msg, o.ok ? 'ok' : 'err');
    refreshNetworkPage();
  }, S.peer);
}

$(ME).find('.svc-installed').change(function(){
  serviceAction($(this).prop('checked') ? send_install_service : send_uninstall_service,
                $(this).prop('checked') ? 'Installing service' : 'Removing service');
});
$(ME).find('.svc-running').change(function(){
  serviceAction($(this).prop('checked') ? send_start_service : send_stop_service,
                $(this).prop('checked') ? 'Starting' : 'Stopping');
});
$(ME).find('.svc-restart').click(function(){ serviceAction(send_restart_service, 'Restarting'); });

// The supervised service model: the tunnel is a child process of Newbound.
// start answers honestly (err with the process output when nebula dies at
// launch), stop kills the tracked child, and info reports the real state.
$(ME).find('.svc-runnow').click(function(){
  busy('.svc-runnow', true);
  status('Starting ' + S.selected + '…', 'busy');
  send_start(S.selected, function(result){
    busy('.svc-runnow', false);
    var o = outcome(result);
    // the status strip gets one sentence; the fix and the raw log go in the pane (#2)
    S.runfail = o.ok ? null : (result && result.data && typeof result.data == 'object' ? result.data : { msg: o.msg });
    status(o.ok ? ((result.data && result.data.msg) || (S.selected + ' is running')) : o.msg, o.ok ? 'ok' : 'err');
    refreshNetworkPage();
  }, S.peer);
});
$(ME).find('.svc-stopnow').click(function(){
  busy('.svc-stopnow', true);
  status('Stopping ' + S.selected + '…', 'busy');
  send_stop(S.selected, function(result){
    busy('.svc-stopnow', false);
    var o = outcome(result);
    status(o.ok ? S.selected + ' stopped' : o.msg, o.ok ? 'ok' : 'err');
    refreshNetworkPage();
  }, S.peer);
});
$(ME).find('.svc-boot').change(function(){
  var on = $(this).prop('checked');
  send_set_boot(S.selected, on, function(result){
    var o = outcome(result);
    status(o.ok ? (S.selected + (on ? ' will start at boot' : ' will not start at boot')) : o.msg, o.ok ? 'ok' : 'err');
    refreshNetworkPage();
  }, S.peer);
});

// ---- save configuration ----
function saveConfig(){
  if (!S.selected || !S.dirty) return;
  var d = {
    host: $(ME).find('.f-host').val(),
    subnet: $(ME).find('.f-subnet').val(),
    ip_address: $(ME).find('.f-ipaddr').val(),
    port: $(ME).find('.f-port').val(),
    groups: $(ME).find('.f-groups').val(),
    am_lighthouse: $(ME).find('.f-amlighthouse').prop('checked'),
    public_address: ($(ME).find('.f-publicaddr').val() || '').trim(),
    lighthouses: S.lighthouses,
    hosts: (S.networks[S.selected].config.hosts || {}),
    use_yaml: $(ME).find('.f-useyaml').prop('checked')
  };
  if (d.use_yaml) d.yaml = $(ME).find('.f-yaml').val();
  busy('.saveconfigbutton', true);
  send_save_config(S.selected, d, function(result){
    busy('.saveconfigbutton', false);
    var o = outcome(result);
    setDirty(!o.ok);
    if (o.ok && result.data){
      S.networks[S.selected].config = result.data;
      $(ME).find('.f-yaml').val(result.data.yaml || '');
    }
    status(o.ok ? 'Configuration saved. Restart the tunnel or service to apply it.' : o.msg, o.ok ? 'ok' : 'err');
  }, S.peer);
}
$(ME).find('.saveconfigbutton').click(saveConfig);

// ---- lighthouses (state, not DOM) ----
function renderLighthouses(){
  var box = $(ME).find('.lighthouselist').empty();
  var ids = Object.keys(S.lighthouses);
  if (!ids.length){ box.append($('<div class="nb-empty">No lighthouses. Fine for Newbound peers; a network with phones needs one.</div>')); return; }
  var t = $('<table class="nb-table"><thead><tr><th>Peer</th><th>Private IP</th><th>Public IP</th><th>Port</th><th></th><th></th></tr></thead><tbody></tbody></table>');
  var tb = t.find('tbody');
  for (var i in ids){
    (function(id){
      var lh = S.lighthouses[id];
      var tr = $('<tr></tr>');
      tr.append($('<td></td>').append($('<div></div>').text(peerName(id)), $('<div class="peerid"></div>').text(id)));
      tr.append($('<td></td>').text(lh.private_ip));
      tr.append($('<td></td>').text(lh.public_ip));
      tr.append($('<td></td>').text(lh.port));
      tr.append($('<td></td>').append($('<button class="rowbtn">edit</button>').click(function(){ editLighthouse(id); })));
      tr.append($('<td></td>').append($('<button class="rowbtn danger">remove</button>').click(function(){
        delete S.lighthouses[id];
        setDirty(true);
        renderLighthouses();
        status('Lighthouse removed from the draft — save the configuration to apply.', 'ok');
      })));
      tb.append(tr);
    })(ids[i]);
  }
  box.append(t);
}

function editLighthouse(id){
  var v = id ? Object.assign({ peer: id }, S.lighthouses[id]) : null;
  openModal('lighthouse', {
    value: v,
    close: closeModal,
    save: function(data){
      closeModal();
      S.lighthouses[data.peer] = { private_ip: data.private_ip, public_ip: data.public_ip, port: data.port };
      setDirty(true);
      renderLighthouses();
      status('Lighthouse noted in the draft — save the configuration to apply.', 'ok');
    }
  });
}
$(ME).find('.addlighthousebutton').click(function(){ editLighthouse(null); });

// ---- members (owner only) ----
function connectedPeers(){
  var out = [];
  for (var i in document.peers) if (document.peers[i].connected) out.push(i);
  return out;
}

function loadMembers(){
  send_members(S.selected, function(result){
    var o = outcome(result);
    S.members = o.ok ? (result.data || {}) : {};
    renderMembers();
  }, S.peer);
}

function renderMembers(){
  renderBlocked();
  var box = $(ME).find('.memberlist').empty();
  var nopeers = connectedPeers().length == 0;
  if (nopeers) $(ME).find('.nopeers-hint').removeAttr('hidden');
  else $(ME).find('.nopeers-hint').attr('hidden', true);
  if (nopeers) $(ME).find('.addmemberbutton').attr('disabled', true);
  else $(ME).find('.addmemberbutton').removeAttr('disabled');
  var ids = Object.keys(S.members);
  if (!ids.length){ box.append($('<div class="nb-empty">No members yet.</div>')); return; }
  var t = $('<table class="nb-table"><thead><tr><th>Member</th><th>Private IP</th><th>Groups</th><th>Added</th><th>Expires</th><th></th></tr></thead><tbody></tbody></table>');
  var tb = t.find('tbody');
  for (var i in ids){
    (function(id){
      var m = S.members[id];
      var tr = $('<tr></tr>');
      var who = $('<div></div>').text(m.name || peerName(id));
      if (m.kind == 'phone') who.append($('<span class="nb-badge">phone</span>'));
      var sub = m.kind == 'phone' ? (m.mode == 'b2' ? 'phone made its own key' : 'key made by Newbound') : id;
      tr.append($('<td></td>').append(who, $('<div class="peerid"></div>').text(sub)));
      tr.append($('<td></td>').text(m.ip_address || ''));
      tr.append($('<td></td>').text(m.groups || ''));
      tr.append($('<td class="when"></td>').text(m.added ? fmtDate(new Date(m.added * 1000)) : '\u2014'));
      var exp = m.not_after ? new Date(m.not_after) : null;
      var soon = exp && !isNaN(exp) && (exp - Date.now()) < 30 * 86400000;
      tr.append($('<td class="when"></td>').toggleClass('soon', !!soon).text(exp && !isNaN(exp) ? fmtDate(exp) : '\u2014'));
      tr.append($('<td></td>').append($('<button class="rowbtn danger">remove</button>').click(function(){ removeMember(id, m); })));
      tb.append(tr);
    })(ids[i]);
  }
  box.append(t);
}

// ---- Remove (ENROLLMENT §4): block the cert here, then on every member ----
function removeMember(id, m){
  var name = m.name || peerName(id);
  if (!confirm('Remove ' + name + ' from ' + S.selected + '?\n\nIts certificate goes on this network\u2019s blocklist: this host and every Newbound member will refuse it' +
      (m.kind == 'phone' ? ', and the phone can no longer connect' : '') + '. This can\u2019t be undone \u2014 to bring it back, add it again.')) return;
  var net = S.selected;
  status('Removing ' + name + '\u2026', 'busy');
  send_remove_member(net, id, function(result){
    var o = outcome(result);
    if (!o.ok){ status(o.msg, 'err'); loadMembers(); return; }
    var bl = result.data.blocklist || {};
    if (S.networks[net]) S.networks[net].config.blocklist = bl;
    var peers = Object.keys(S.members).filter(function(k){ return k != id && S.members[k].kind != 'phone'; });
    pushPolicy(net, peers, function(failures){
      var head = result.data.msg;
      if (!peers.length) status(head, result.data.fingerprint ? 'ok' : 'err');
      else if (failures.length) status(head + ' Not yet blocked on: ' + failures.join('; ') + '. Sync mesh resends it.', 'err');
      else status(head + ' Also blocked on ' + peers.length + (peers.length == 1 ? ' member.' : ' members.'), 'ok');
      loadMembers();
    });
  }, S.peer);
}

// The network's policy for Newbound members: the blocklist (removed certs)
// and, on networks made since Phase 3, the member-only inbound rules.
// Sent on Remove and on every Sync mesh, so members that were offline catch up.
function pushPolicy(net, peers, done){
  var cfg = (S.networks[net] && S.networks[net].config) || {};
  var bl = cfg.blocklist || {}, fw = cfg.firewall || '';
  var jobs = [], failures = [];
  peers.forEach(function(p){
    if (Object.keys(bl).length) jobs.push(function(cb){ send_update_blocklist(net, bl, function(r){ var o = outcome(r); if (!o.ok) failures.push(peerName(p) + ' (' + o.msg + ')'); cb(); }, p); });
    if (fw) jobs.push(function(cb){ send_set_firewall(net, fw, function(r){ var o = outcome(r); if (!o.ok) failures.push(peerName(p) + ' (' + o.msg + ')'); cb(); }, p); });
  });
  var n = jobs.length;
  if (!n) return done(failures);
  jobs.forEach(function(job){ job(function(){ if (--n == 0) done(failures); }); });
}

function renderBlocked(){
  var bl = (S.networks[S.selected] && S.networks[S.selected].config.blocklist) || {};
  var fps = Object.keys(bl), box = $(ME).find('.blockedbox');
  if (!fps.length){ box.attr('hidden', true); return; }
  var t = $('<table class="nb-table"><thead><tr><th>Name</th><th>Removed</th><th>Certificate fingerprint</th></tr></thead><tbody></tbody></table>');
  fps.sort(function(a, b){ return (bl[b].removed || 0) - (bl[a].removed || 0); }).forEach(function(fp){
    var e = bl[fp] || {};
    t.find('tbody').append($('<tr></tr>').append(
      $('<td></td>').text(e.name || '\u2014'),
      $('<td class="when"></td>').text(e.removed ? fmtDate(new Date(e.removed * 1000)) : '\u2014'),
      $('<td class="fp"></td>').text(fp.slice(0, 16) + '\u2026')));
  });
  box.find('.blockedlist').empty().append(t);
  box.removeAttr('hidden');
}

// Propose the lowest free host number, checking members, lighthouses, and this host.
function proposeIP(){
  var subnet = $(ME).find('.f-subnet').val() || S.networks[S.selected].config.subnet || '';
  var used = {};
  for (var id in S.members) if (S.members[id].ip_address) used[S.members[id].ip_address.split('/')[0]] = 1;
  for (var id in S.lighthouses) if (S.lighthouses[id].private_ip) used[S.lighthouses[id].private_ip] = 1;
  var own = ($(ME).find('.f-ipaddr').val() || '').split('/')[0];
  if (own) used[own] = 1;
  for (var n = 2; n < 255; n++){
    var ip = subnet.replace('X', '' + n);
    if (!used[ip.split('/')[0]]) return ip;
  }
  return subnet.replace('X', '2');
}

// ---- the peer mesh: Newbound is the lighthouse ----
// A nebula lighthouse does two jobs: it tells members each other's public
// endpoints, and it triggers hole punching. The encrypted Newbound peer
// network already reaches every member, so it can do the first job itself:
// gather each member's local candidates + the addresses the peer layer
// OBSERVED for their connections, then push the merged map to everyone.
// save_config renders it into static_host_map with punchy on — members
// punch through to each other directly, no publicly reachable host needed.
function syncMesh(quiet){
  var name = S.selected;
  var cfg = S.networks[name].config;
  var port = cfg.port;
  var ownerUuid = S.peer ? S.peer : S.mypeerid;
  // phones aren't on the Newbound peer network: they get the map in their join code
  var ids = Object.keys(S.members).filter(function(id){ return S.members[id].kind != 'phone'; });
  if (!ids.length){ if (!quiet) status('No members yet — the mesh has nothing to sync.', 'ok'); return; }
  busy('.syncmeshbutton', true);
  if (!quiet) status('Syncing the mesh: gathering addresses over the peer network…', 'busy');

  var hosts = {};          // overlay ip -> [ 'ip:port', ... ]
  var ownerVotes = {};     // what members observe as the owner's public face
  var failures = [];
  var ownerIp = (cfg.ip_address || '').split('/')[0];
  var pending = 2 * ids.length + 1;

  function addHost(ip, eps){
    if (!hosts[ip]) hosts[ip] = [];
    for (var i in eps) if (eps[i] && hosts[ip].indexOf(eps[i]) == -1) hosts[ip].push(eps[i]);
  }

  function gathered(){
    if (--pending > 0) return;
    for (var a in ownerVotes){ addHost(ownerIp, [a + ':' + port]); }
    // push the same map to every member, then apply it locally
    var pending2 = ids.length + 1;
    function pushed(){
      if (--pending2 > 0) return;
      // then the blocklist and firewall mode, so offline members catch up
      pushPolicy(name, ids, function(pf){
        busy('.syncmeshbutton', false);
        failures = failures.concat(pf);
        if (failures.length) status('Mesh synced with problems: ' + failures.join('; '), 'err');
        else if (!quiet) status('Mesh synced across ' + (ids.length + 1) + ' hosts — members reach each other without a public lighthouse.', 'ok');
        refreshNetworkPage();
      });
    }
    for (var k in ids){
      (function(id){
        send_update_hosts(name, hosts, function(result){
          var o = outcome(result);
          if (!o.ok) failures.push(peerName(id) + ': ' + o.msg);
          pushed();
        }, id);
      })(ids[k]);
    }
    send_update_hosts(name, hosts, function(result){
      var o = outcome(result);
      if (!o.ok) failures.push('this instance: ' + o.msg);
      pushed();
    }, S.peer);
  }

  // the owner's own local candidates
  send_endpoints(name, '', function(result){
    var o = outcome(result);
    if (o.ok && result.data.endpoints) addHost(ownerIp, result.data.endpoints);
    gathered();
  }, S.peer);

  for (var k in ids){
    (function(id){
      var mip = (S.members[id].ip_address || '').split('/')[0];
      // what the owner's peer layer observed for this member
      send_endpoints(name, id, function(result){
        var o = outcome(result);
        if (o.ok && result.data.observed) addHost(mip, [result.data.observed + ':' + port]);
        gathered();
      }, S.peer);
      // the member's own local candidates + its view of the owner
      send_endpoints(name, ownerUuid, function(result){
        var o = outcome(result);
        if (o.ok){
          if (result.data.endpoints) addHost(mip, result.data.endpoints);
          if (result.data.observed) ownerVotes[result.data.observed] = 1;
        } else failures.push(peerName(id) + ': ' + o.msg);
        gathered();
      }, id);
    })(ids[k]);
  }
}
$(ME).find('.syncmeshbutton').click(function(){ syncMesh(false); });

$(ME).find('.addmemberbutton').click(function(){
  openModal('member', {
    value: { ip_address: proposeIP(), groups: '', peer: null },
    close: closeModal,
    save: function(data){
      closeModal();
      var servicename = S.selected;
      var target = data.peer;
      var cfg = S.networks[servicename].config;
      var subnet = $(ME).find('.f-subnet').val() || cfg.subnet;
      var port = $(ME).find('.f-port').val() || cfg.port;
      var owner = S.peer ? S.peer : S.mypeerid;
      status('Signing a certificate for ' + peerName(target) + '…', 'busy');
      send_add_member(servicename, target, data.ip_address, data.groups || '', function(result){
        var o = outcome(result);
        if (!o.ok){ status('Could not sign the member certificate: ' + o.msg, 'err'); return; }
        var b = result.data;
        status('Delivering credentials to ' + peerName(target) + '…', 'busy');
        send_join_network(servicename, subnet, data.ip_address, '' + port, owner,
                          b.ca_crt, b.host_crt, b.host_key, S.lighthouses, data.groups || '',
        function(result2){
          var o2 = outcome(result2);
          status(o2.ok ? peerName(target) + ' joined ' + servicename + ' — syncing the mesh so everyone can reach it…'
                       : 'Signed, but the peer could not join: ' + o2.msg,
                 o2.ok ? 'ok' : 'err');
          send_members(servicename, function(result3){
            var o3 = outcome(result3);
            S.members = o3.ok ? (result3.data || {}) : {};
            renderMembers();
            if (o2.ok) syncMesh(true);
          }, S.peer);
        }, target);
      }, S.peer);
    }
  });
});

// ---- modal plumbing: Esc and backdrop close every popup ----
function openModal(ctl, data){
  var card = $(ME).find('.popmeup').empty();
  data.close = closeModal;
  $(ME).find('.nb-modal').removeAttr('hidden');
  installControl(card[0], 'nebula', ctl, function(){}, data);
}
function closeModal(){
  var hadPhone = !!PHONE;
  wipePhone();
  $(ME).find('.nb-modal').attr('hidden', true);
  $(ME).find('.popmeup').empty();
  if (hadPhone && S.selected) loadMembers();
}
$(ME).find('.nb-modal').click(function(e){ if (e.target === this) closeModal(); });
$(document).keydown(function(e){
  var modal = !$(ME).find('.nb-modal').attr('hidden');
  if (e.key == 'Escape'){
    if (modal) closeModal();
    else if (S.selected) { e.preventDefault(); goBack(); }   // Back (Esc), as in Dev
    return;
  }
  // Ctrl/\u2318+S saves the open network's configuration, as in Dev
  if ((e.ctrlKey || e.metaKey) && (e.key == 's' || e.key == 'S') && S.selected && !modal){
    e.preventDefault();
    saveConfig();
  }
});

// ---- public address of this host, when it is its own lighthouse ----
function togglePublicAddr(){
  if ($(ME).find('.f-amlighthouse').prop('checked')) $(ME).find('.publicaddr').removeAttr('hidden');
  else $(ME).find('.publicaddr').attr('hidden', true);
}
$(ME).find('.f-amlighthouse').change(togglePublicAddr);

// ---- Add a phone: Mode A of ENROLLMENT.md (one scan) ----
// add_phone signs the cert and returns the join payload, private key
// included. The key lives only in PHONE (this page's memory) until the code
// expires or the dialog closes; then it is dropped. Nothing is stored.
var PHONE = null;   // { join, url, deadline, timer }

function wipePhone(){
  if (!PHONE) return;
  if (PHONE.timer) clearInterval(PHONE.timer);
  if (PHONE.join) PHONE.join.key = null;
  PHONE.join = null; PHONE.url = null;
  PHONE = null;
}

function loadQR(cb){
  if (window.QRCode && window.QRCode.Model) return cb();
  var s = document.createElement('script');
  s.src = '../app/asset/nebula/vendor/qrcode.js';
  s.onload = function(){ cb(); };
  s.onerror = function(){ cb('The QR encoder didn’t load.'); };
  document.head.appendChild(s);
}

// ENROLLMENT §1.1: JSON -> raw DEFLATE -> base64url (no padding) -> newbound://join?v=1&d=
function b64url(bytes){
  var s = '';
  for (var i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]);
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
function joinURL(obj){
  if (typeof CompressionStream == 'undefined')
    return Promise.reject(new Error('This browser can’t compress the code (no CompressionStream). Use a current Chrome, Edge, Firefox or Safari.'));
  var stream = new Blob([JSON.stringify(obj)]).stream().pipeThrough(new CompressionStream('deflate-raw'));
  return new Response(stream).arrayBuffer().then(function(buf){
    return 'newbound://' + obj.kind + '?v=1&d=' + b64url(new Uint8Array(buf));
  });
}

function qrSVG(text){
  var L = QRCode.CorrectLevel.L;
  var m = new QRCode.Model(QRCode.typeNumberFor(text, L), L);
  m.addData(text); m.make();
  var n = m.getModuleCount(), q = 4, d = '';
  for (var r = 0; r < n; r++) for (var c = 0; c < n; c++) if (m.isDark(r, c)) d += 'M' + (c + q) + ' ' + (r + q) + 'h1v1h-1z';
  var size = n + 2 * q, px = Math.max(4, Math.floor(560 / size));
  return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ' + size + ' ' + size + '" width="' + (size * px) + '" height="' + (size * px) +
    '" shape-rendering="crispEdges" role="img" aria-label="Join code for this phone"><rect width="' + size + '" height="' + size + '" fill="#fff"/><path d="' + d + '" fill="#000"/></svg>';
}

function slug(s){ return (s || 'phone').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'phone'; }
function download(name, text, type){
  var a = document.createElement('a');
  var url = URL.createObjectURL(new Blob([text], { type: type || 'text/plain' }));
  a.href = url; a.download = name; document.body.appendChild(a); a.click();
  setTimeout(function(){ URL.revokeObjectURL(url); a.remove(); }, 1000);
}

// ENROLLMENT §2.1: a plain nebula YAML with the PEMs inline, for
// Defined Networking's Mobile Nebula app until nebula mobile ships.
function mobileNebulaYAML(j){
  function pem(s){ return '|\n' + String(s || '').trim().split('\n').map(function(l){ return '    ' + l; }).join('\n') + '\n'; }
  var shm = '', hosts = '';
  for (var i in j.lighthouses){
    var lh = j.lighthouses[i];
    shm += '  "' + lh.nebula_ip + '": [' + lh.addrs.map(function(a){ return '"' + a + '"'; }).join(', ') + ']\n';
    hosts += '    - "' + lh.nebula_ip + '"\n';
  }
  for (var ip in (j.static_hosts || {})){
    var isLh = j.lighthouses.some(function(l){ return l.nebula_ip == ip; });
    if (!isLh) shm += '  "' + ip + '": [' + j.static_hosts[ip].map(function(a){ return '"' + a + '"'; }).join(', ') + ']\n';
  }
  return '# ' + j.device_name + ' on ' + j.network + ' (made by the Newbound nebula app)\n' +
    '# Contains this phone’s private key: import it, then delete the file.\n' +
    'pki:\n  ca: ' + pem(j.ca) + '  cert: ' + pem(j.cert) + '  key: ' + pem(j.key) +
    'static_host_map:\n' + shm +
    'lighthouse:\n  am_lighthouse: false\n  interval: 60\n  hosts:\n' + hosts +
    'listen:\n  host: 0.0.0.0\n  port: 0\n' +
    'punchy:\n  punch: true\n  respond: true\n' +
    'tun:\n  mtu: 1300\n' +
    'firewall:\n  outbound:\n    - port: any\n      proto: any\n      host: any\n' +
    '  inbound:\n    - port: any\n      proto: icmp\n      host: any\n';
}

function phoneStep1(err, fix){
  var card = $(ME).find('.popmeup').empty();
  var d = $('<div class="nbdlg phonedlg">' +
    '<div class="nbdlg-head"><span class="nbdlg-title">Add a phone</span><button class="nbdlg-x" title="Close">&#10005;</button></div>' +
    '<div class="nbdlg-body">' +
      '<label><span class="lbl">phone name</span><input type="text" class="ph-name" maxlength="64" placeholder="e.g. Alex’s iPhone"></label>' +
      '<label><span class="lbl">extra groups (optional)</span><input type="text" class="ph-groups" placeholder="Comma separated — member and phone are always added"></label>' +
      '<label class="ph-publabel" hidden><span class="lbl">the phone\u2019s public key</span><textarea class="ph-pub" spellcheck="false" placeholder="-----BEGIN NEBULA X25519 PUBLIC KEY-----&#10;…&#10;-----END NEBULA X25519 PUBLIC KEY-----"></textarea></label>' +
      '<div class="nb-hint ph-explain"></div>' +
      '<button type="button" class="ph-modeswitch"></button>' +
      '<div class="ph-b1"><button type="button" class="btn" disabled>Invite over home Wi-Fi</button><span class="nb-hint ph-b1why"></span></div>' +
      '<div class="nbdlg-err" hidden></div>' +
      '<div class="ph-fix" hidden></div>' +
    '</div>' +
    '<div class="nbdlg-foot"><button class="btn ph-cancel">Cancel</button><button class="btn btn-primary ph-make">Make join code</button></div>' +
  '</div>');
  // Mode A (default): Newbound makes the key. B2: the phone made its own and
  // only its public key comes here (ENROLLMENT §3.2).
  function setMode(b2){
    d.data('b2', b2);
    if (!b2) d.find('.ph-publabel').attr('hidden', true); else d.find('.ph-publabel').removeAttr('hidden');
    d.find('.ph-make').text(b2 ? 'Sign the key' : 'Make join code');
    d.find('.ph-modeswitch').text(b2 ? 'Let Newbound make the key instead (one scan)' : 'The phone made its own key? Paste its public key');
    d.find('.ph-explain').text(b2
      ? 'Stricter: the phone\u2019s private key never leaves the phone. Paste the public key it shows; Newbound signs it and gives back a code with the certificate (nothing secret in it).'
      : 'Newbound makes the phone\u2019s key and puts everything in one QR code. Anyone who sees the code can join ' + S.selected + ' as this phone, so show it only to the phone. The code disappears after 10 minutes.');
  }
  d.find('.ph-modeswitch').click(function(){ setMode(!d.data('b2')); });
  // Mode B1 (invite over home Wi-Fi) waits on platform decisions: docs/INVITES.md
  d.find('.ph-b1why').text('Coming later: it needs a Newbound platform decision. Pasting the phone\u2019s public key keeps its key on the phone too.' +
    (isLoopback(S.info && S.info.http_address) ? ' (Newbound also isn\u2019t reachable on your network yet.)' : ''));
  setMode(false);
  d.find('.nbdlg-x, .ph-cancel').click(closeModal);
  d.find('.ph-make').click(makePhone);
  d.find('.ph-name').on('keydown', function(e){ if (e.key == 'Enter') makePhone(); });
  card.append(d);
  $(ME).find('.nb-modal').removeAttr('hidden');
  if (err) phoneErr(err, fix);
  d.find('.ph-name').focus();
}

function phoneErr(msg, fix){
  var card = $(ME).find('.popmeup');
  card.find('.nbdlg-err').removeAttr('hidden').text(msg);
  var box = card.find('.ph-fix').empty().attr('hidden', true);
  if (fix){
    box.removeAttr('hidden').append($('<button class="btn"></button>').text(fix.label).click(function(){ closeModal(); fix.go(); }));
  }
}

function gotoTab(tab, focus){
  $(ME).find('.nb-tab[data-tab=' + tab + ']').click();
  if (focus) setTimeout(function(){ $(ME).find(focus).focus(); }, 50);
}

function makePhone(){
  var card = $(ME).find('.popmeup');
  var name = (card.find('.ph-name').val() || '').trim();
  var groups = (card.find('.ph-groups').val() || '').trim();
  var b2 = !!card.find('.phonedlg').data('b2');
  var pub = b2 ? (card.find('.ph-pub').val() || '').trim() : '';
  if (!name) return phoneErr('Give the phone a name first.');
  if (b2 && !pub) return phoneErr('Paste the public key the phone shows.');
  if (S.dirty) return phoneErr('Save your changes first — the phone gets the saved lighthouses.', { label: 'Save configuration', go: saveConfig });
  card.find('.nbdlg-err').attr('hidden', true);
  busy('.ph-make', true);
  send_add_phone(S.selected, name, groups, pub, function(result){
    busy('.ph-make', false);
    var o = outcome(result);
    if (!o.ok){
      var r = result && result.data && result.data.reason;
      var fix = null;
      if (r == 'no_lighthouse'){
        var self = $(ME).find('.f-amlighthouse').prop('checked');
        fix = self ? { label: 'Open the Service tab', go: function(){ gotoTab('svc', '.f-publicaddr'); } }
                   : { label: 'Open the Lighthouses tab', go: function(){ gotoTab('lh'); } };
      }
      return phoneErr(o.msg, fix);
    }
    var j = result.data.join;
    if (j.kind == 'cert'){
      // B2: nothing secret, so no blur and no countdown
      PHONE = { join: j, deadline: null };
      return joinURL(j).then(function(url){
        if (!PHONE) return;
        PHONE.url = url;
        loadQR(function(err){ if (PHONE) certStep(err); });
      }, function(e){ phoneErr(e.message); });
    }
    // count down from now, not from the server clock
    PHONE = { join: j, deadline: Date.now() + (j.expires - j.created) * 1000 };
    joinURL(j).then(function(url){
      if (!PHONE) return;     // closed meanwhile
      PHONE.url = url;
      loadQR(function(err){ if (PHONE) phoneStep2(err); });
    }, function(e){ phoneErr(e.message); });
  }, S.peer);
}

function phoneStep2(qrErr){
  var j = PHONE.join, url = PHONE.url, long = url.length > 1800;
  var card = $(ME).find('.popmeup').empty();
  var d = $('<div class="nbdlg phonedlg">' +
    '<div class="nbdlg-head"><span class="nbdlg-title">Add a phone</span><button class="nbdlg-x" title="Close">&#10005;</button></div>' +
    '<div class="nbdlg-body">' +
      '<div class="ph-meta"></div>' +
      '<div class="ph-qrbox"><div class="ph-qr blurred"></div><button class="btn btn-primary ph-show">Show code</button></div>' +
      '<div class="nb-hint ph-long" hidden>This code is long, so the QR is dense. If the phone can’t read it, use the file or the link.</div>' +
      '<div class="ph-actions">' +
        '<button class="btn ph-dl">Download .nbjoin</button>' +
        '<button class="btn ph-copy">Copy link</button>' +
        '<button class="btn ph-yaml" title="A nebula YAML for Defined Networking’s Mobile Nebula app">Download for Mobile Nebula</button>' +
      '</div>' +
      '<div class="nb-hint ph-scanhint">Scan with the phone’s camera once nebula mobile is installed. Until then, the Mobile Nebula file works with Defined Networking’s app. The files hold the phone’s private key: send them only to the phone, then delete them.</div>' +
    '</div>' +
    '<div class="nbdlg-foot"><button class="btn btn-primary ph-done">Done</button></div>' +
  '</div>');
  d.find('.ph-meta').append($('<b></b>').text(j.device_name), document.createTextNode(' · ' + j.ip.split('/')[0] + ' on ' + j.network + ' · '),
    $('<span class="ph-when"></span>').append(document.createTextNode('code disappears in '), $('<span class="ph-left"></span>')));
  if (long){ d.find('.ph-actions').insertBefore(d.find('.ph-qrbox')); d.find('.ph-long').removeAttr('hidden'); }
  if (qrErr) d.find('.ph-qrbox').empty().append($('<div class="ph-expired"></div>').text(qrErr + ' Use the file or the link instead.'));
  else d.find('.ph-qr').html(qrSVG(url));
  d.find('.ph-show').click(function(){ d.find('.ph-qr').removeClass('blurred'); $(this).remove(); });
  d.find('.ph-dl').click(function(){ if (PHONE) download(j.network + '-' + slug(j.device_name) + '.nbjoin', PHONE.url + '\n', 'application/octet-stream'); });
  d.find('.ph-copy').click(function(){ if (PHONE) copyText(PHONE.url, $(this)); });
  d.find('.ph-yaml').click(function(){ if (PHONE) download(j.network + '-' + slug(j.device_name) + '.yml', mobileNebulaYAML(PHONE.join), 'application/x-yaml'); });
  d.find('.nbdlg-x, .ph-done').click(closeModal);
  card.append(d);
  function tick(){
    if (!PHONE) return;
    var left = Math.max(0, Math.round((PHONE.deadline - Date.now()) / 1000));
    d.find('.ph-left').text(Math.floor(left / 60) + ':' + ('0' + (left % 60)).slice(-2));
    if (left > 0) return;
    // expired: drop the key and everything that encodes it
    clearInterval(PHONE.timer); PHONE.timer = null;
    if (PHONE.join) PHONE.join.key = null;
    PHONE.join = null; PHONE.url = null;
    d.find('.ph-qrbox, .ph-actions, .ph-long, .ph-scanhint').remove();
    d.find('.ph-when').text('code expired');
    d.find('.ph-meta').after($('<div class="ph-expired"></div>').text('This code has expired and can’t be shown again. ' +
      j.device_name + ' is listed under Members; if it didn’t join, remove it there and add the phone again.'));
  }
  PHONE.timer = setInterval(tick, 1000);
  tick();
}

// B2 result: the `cert` code (Mode A fields minus the key, plus
// pubkey_fingerprint). Also the plain PEMs, for apps that take them by hand
// (Defined Networking's Mobile Nebula: paste the certificate and the CA).
function certStep(qrErr){
  var j = PHONE.join, url = PHONE.url;
  var card = $(ME).find('.popmeup').empty();
  var d = $('<div class="nbdlg phonedlg certdlg">' +
    '<div class="nbdlg-head"><span class="nbdlg-title">Phone key signed</span><button class="nbdlg-x" title="Close">&#10005;</button></div>' +
    '<div class="nbdlg-body">' +
      '<div class="ph-meta"></div>' +
      '<div class="ph-qrbox"><div class="ph-qr"></div></div>' +
      '<div class="ph-actions">' +
        '<button class="btn ph-dl">Download .nbjoin</button>' +
        '<button class="btn ph-copy">Copy link</button>' +
      '</div>' +
      '<div class="nb-hint">nebula mobile: scan this or open the link on the phone that made the key. Nothing in it is secret.</div>' +
      '<div class="ph-dnn">' +
        '<div class="lbl">Defined Networking\u2019s Mobile Nebula, by hand</div>' +
        '<div class="nb-hint ph-dnnhint"></div>' +
        '<div class="ph-actions"><button class="btn ph-copycert">Copy certificate</button><button class="btn ph-copyca">Copy CA</button></div>' +
      '</div>' +
    '</div>' +
    '<div class="nbdlg-foot"><button class="btn btn-primary ph-done">Done</button></div>' +
  '</div>');
  d.find('.ph-meta').append($('<b></b>').text(j.device_name), document.createTextNode(' \u00b7 ' + j.ip.split('/')[0] + ' on ' + j.network + ' \u00b7 key fingerprint ' + j.pubkey_fingerprint.slice(0, 12) + '\u2026'));
  var lh = (j.lighthouses || []).map(function(l){ return l.nebula_ip + ' \u2192 ' + l.addrs.join(', '); }).join('; ');
  d.find('.ph-dnnhint').text('Paste the certificate and the CA into the site you made the key in, and add the lighthouse ' + lh + '.');
  if (qrErr) d.find('.ph-qrbox').empty().append($('<div class="ph-expired"></div>').text(qrErr + ' Use the file or the link instead.'));
  else d.find('.ph-qr').html(qrSVG(url));
  d.find('.ph-dl').click(function(){ download(j.network + '-' + slug(j.device_name) + '.nbjoin', url + '\n', 'application/octet-stream'); });
  d.find('.ph-copy').click(function(){ copyText(url, $(this)); });
  d.find('.ph-copycert').click(function(){ copyText(j.cert, $(this)); });
  d.find('.ph-copyca').click(function(){ copyText(j.ca, $(this)); });
  d.find('.nbdlg-x, .ph-done').click(closeModal);
  card.append(d);
}

$(ME).find('.addphonebutton').click(function(){ phoneStep1(); });

// ---- ufw on Linux (Phase 5 QA): a phone can join and still be blocked ----
function renderUfw(n){
  var el = $(ME).find('.fwhint'), c = n.config || {};
  if (!(S.info && S.info.ufw) || !c.port){ el.attr('hidden', true); return; }
  var net = n.name, port = S.info.http_port;
  el.find('.fwhint-how').text('The app can\u2019t read the rules without sudo. If a phone joins ' + net +
    ' but can\u2019t connect, or connects but can\u2019t open Newbound, these open exactly what\u2019s needed: the tunnel\u2019s UDP port, and Newbound\u2019s web port only on the ' + net + ' interface.');
  el.find('.fw-cmd1').text('sudo ufw allow ' + c.port + '/udp comment \'nebula ' + net + '\'');
  if (port && !isLoopback(S.info.http_address)){
    el.find('.fw-web').removeAttr('hidden');
    el.find('.fw-cmd2').text('sudo ufw allow in on ' + net + ' to any port ' + port + ' proto tcp comment \'newbound over ' + net + '\'');
  } else el.find('.fw-web').attr('hidden', true);
  el.removeAttr('hidden');
}
$(ME).find('.copyfw1').click(function(){ copyText($(ME).find('.fw-cmd1').text(), $(this)); });
$(ME).find('.copyfw2').click(function(){ copyText($(ME).find('.fw-cmd2').text(), $(this)); });

// ---- inbound rule of the open network (ENROLLMENT §5) ----
function renderFirewall(n){
  var c = n.config || {};
  var el = $(ME).find('.fwline');
  if (n.owner != 'local' && !c.firewall){ el.attr('hidden', true); return; }
  el.removeAttr('hidden').find('.fwmode').text(c.firewall == 'member'
    ? 'members only (certificates in group member)'
    : 'any certificate from this network\u2019s CA (made before member-only rules)');
}
