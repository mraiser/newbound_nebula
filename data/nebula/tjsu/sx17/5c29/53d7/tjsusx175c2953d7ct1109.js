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
  releases: null
};

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
    if (d.tag_name == 'Not Installed'){
      $(ME).find('.nebulaversion').text('not installed — fetch a release below');
    } else {
      $(ME).find('.nebulaversion').text(d.tag_name + ' (' + d.binary_name + ')');
    }
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
$(ME).find('.checkupdatebutton').click(function(){
  var msg = $(ME).find('.updatemsg');
  $(ME).find('.engine-install').removeAttr('hidden');
  msg.text('checking releases…');
  busy('.checkupdatebutton', true);
  var ctrl = new AbortController();
  var timer = setTimeout(function(){ ctrl.abort(); }, 10000);
  fetch('https://api.github.com/repos/slackhq/nebula/releases', { signal: ctrl.signal })
    .then(function(r){ if (!r.ok) throw new Error('HTTP ' + r.status); return r.json(); })
    .then(function(list){
      clearTimeout(timer);
      busy('.checkupdatebutton', false);
      msg.text('');
      S.releases = {};
      for (var i in list) S.releases[list[i].id] = list[i];
      installControl($(ME).find('.updatemsg')[0], 'app', 'select', function(){
        selectVersion(list[0].id);
      }, { list: list, label: 'Version', cb: selectVersion });
    })
    .catch(function(e){
      clearTimeout(timer);
      busy('.checkupdatebutton', false);
      msg.text('');
      status('Could not reach api.github.com (' + e.message + '). If this instance is offline, download a release elsewhere and install it by URL on a reachable mirror.', 'err');
    });
});

function selectVersion(id){
  var r = S.releases[id];
  var binaries = [];
  for (var i in r.assets){
    var a = r.assets[i];
    var m = a.name.match(/^nebula-(.+?)\.(tar\.gz|zip)$/);
    if (m) binaries.push({ name: m[1], id: a.browser_download_url });
  }
  installControl($(ME).find('.downloadselect')[0], 'app', 'select', function(){
    $(ME).find('.installupdatebutton').removeAttr('hidden');
  }, { list: binaries, label: 'Platform' });
}

$(ME).find('.installupdatebutton').click(function(){
  var url = $(ME).find('.downloadselect').find('select').val();
  var vel = $(ME).find('.updatemsg').find('select')[0];
  var version = vel.options[vel.selectedIndex].text;
  busy('.installupdatebutton', true);
  status('Downloading ' + version + ' on ' + peerName(S.peer) + '…', 'busy');
  send_install_release(url, version, function(result){
    busy('.installupdatebutton', false);
    var o = outcome(result);
    status(o.ok ? 'Installed ' + version : o.msg, o.ok ? 'ok' : 'err');
    refresh();
  }, S.peer);
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
$(ME).find('.addnetworkbutton').click(function(){
  var n = 0;
  while (S.networks['nebula' + (n + 1)]) n++;
  openModal('network', {
    value: { name: 'nebula' + (n + 1), subnet: '192.168.' + (100 + n) + '.X/24', port: '' + (4242 + n) },
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
  S.selected = null;
}
$(ME).find('.backbutton').click(function(){ showHome(); refresh(); });

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
}

function renderServicePane(n){
  var c = n.config;
  $(ME).find('.svc-installed').prop('checked', !!n.service);
  $(ME).find('.svc-running').prop('checked', !!n.running);
  $(ME).find('.f-amlighthouse').prop('checked', !!c.am_lighthouse);
  $(ME).find('.f-host').val(c.host || '');
  $(ME).find('.f-port').val(c.port || '');
  $(ME).find('.f-subnet').val(c.subnet || '');
  $(ME).find('.f-ipaddr').val(c.ip_address || '');
  $(ME).find('.f-groups').val(c.groups || '');
}

// After any service action, re-read reality instead of trusting the click.
function refreshNetworkPage(){
  var name = S.selected;
  send_info(function(result){
    var o = outcome(result);
    if (o.ok && result.data && result.data.networks){
      renderNetworks(result.data.networks);
      if (name && S.networks[name]){
        var n = S.networks[name];
        S.selected = name;
        $(ME).find('.svc-installed').prop('checked', !!n.service);
        $(ME).find('.svc-running').prop('checked', !!n.running);
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

// The v51 service model: run the tunnel as a supervised child process.
$(ME).find('.svc-runnow').click(function(){
  busy('.svc-runnow', true);
  status('Starting ' + S.selected + ' as a supervised process…', 'busy');
  send_start(S.selected, function(result){
    busy('.svc-runnow', false);
    var o = outcome(result);
    status(o.ok ? S.selected + ' launched. Output goes to the Newbound log; info does not yet report supervised-process state.' : o.msg,
           o.ok ? 'ok' : 'err');
  }, S.peer);
});

// ---- save configuration ----
$(ME).find('.saveconfigbutton').click(function(){
  var d = {
    host: $(ME).find('.f-host').val(),
    subnet: $(ME).find('.f-subnet').val(),
    ip_address: $(ME).find('.f-ipaddr').val(),
    port: $(ME).find('.f-port').val(),
    groups: $(ME).find('.f-groups').val(),
    am_lighthouse: $(ME).find('.f-amlighthouse').prop('checked'),
    lighthouses: S.lighthouses,
    use_yaml: $(ME).find('.f-useyaml').prop('checked')
  };
  if (d.use_yaml) d.yaml = $(ME).find('.f-yaml').val();
  busy('.saveconfigbutton', true);
  send_save_config(S.selected, d, function(result){
    busy('.saveconfigbutton', false);
    var o = outcome(result);
    if (o.ok && result.data){
      S.networks[S.selected].config = result.data;
      $(ME).find('.f-yaml').val(result.data.yaml || '');
    }
    status(o.ok ? 'Configuration saved. Restart the service to apply it.' : o.msg, o.ok ? 'ok' : 'err');
  }, S.peer);
});

// ---- lighthouses (state, not DOM) ----
function renderLighthouses(){
  var box = $(ME).find('.lighthouselist').empty();
  var ids = Object.keys(S.lighthouses);
  if (!ids.length){ box.append($('<div class="nb-empty">No lighthouses. Members behind NAT need at least one.</div>')); return; }
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
  var box = $(ME).find('.memberlist').empty();
  var nopeers = connectedPeers().length == 0;
  if (nopeers) $(ME).find('.nopeers-hint').removeAttr('hidden');
  else $(ME).find('.nopeers-hint').attr('hidden', true);
  if (nopeers) $(ME).find('.addmemberbutton').attr('disabled', true);
  else $(ME).find('.addmemberbutton').removeAttr('disabled');
  var ids = Object.keys(S.members);
  if (!ids.length){ box.append($('<div class="nb-empty">No members yet.</div>')); return; }
  var t = $('<table class="nb-table"><thead><tr><th>Peer</th><th>Private IP</th><th>Groups</th><th></th></tr></thead><tbody></tbody></table>');
  var tb = t.find('tbody');
  for (var i in ids){
    (function(id){
      var m = S.members[id];
      var tr = $('<tr></tr>');
      tr.append($('<td></td>').append($('<div></div>').text(m.name || peerName(id)), $('<div class="peerid"></div>').text(id)));
      tr.append($('<td></td>').text(m.ip_address || ''));
      tr.append($('<td></td>').text(m.groups || ''));
      tr.append($('<td></td>').append($('<button class="rowbtn danger">remove</button>').click(function(){
        if (!confirm('Remove ' + peerName(id) + ' from ' + S.selected + '?\n\nIts issued certificate stays valid until it expires or the CA is rotated — removal only stops future re-issue.')) return;
        send_remove_member(S.selected, id, function(result){
          var o = outcome(result);
          status(o.ok ? 'Member removed. The certificate remains valid until expiry or CA rotation.' : o.msg, o.ok ? 'ok' : 'err');
          loadMembers();
        }, S.peer);
      })));
      tb.append(tr);
    })(ids[i]);
  }
  box.append(t);
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
          status(o2.ok ? peerName(target) + ' joined ' + servicename + '. Start its service from the peer selector above.'
                       : 'Signed, but the peer could not join: ' + o2.msg,
                 o2.ok ? 'ok' : 'err');
          loadMembers();
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
  $(ME).find('.nb-modal').attr('hidden', true);
  $(ME).find('.popmeup').empty();
}
$(ME).find('.nb-modal').click(function(e){ if (e.target === this) closeModal(); });
$(document).keydown(function(e){
  if (e.key == 'Escape' && !$(ME).find('.nb-modal').attr('hidden')) closeModal();
});
