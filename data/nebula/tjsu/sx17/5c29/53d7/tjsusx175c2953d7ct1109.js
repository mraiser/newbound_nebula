var me = this; 
var ME = $('#'+me.UUID)[0];

me.uiReady = function(ui){
  me.ui = ui;
  ui.initNavbar(ME);
  $(ME).find('.homepage').css('display', 'block');

  json('../peer/info', null, function(result){
    me.mypeerid = result.data.uuid;
    me.mypeername = result.data.name;
    var d = {
      "local": true,
      "connectedonly": true,
      "ready": rebuild,
      "cb": function(val){
        var b = val == 'local'
        if (b) me.peer = null;
        else me.peer = val;
        $(ME).find('.localonly').css('display', b  ? 'inline-block' : 'none');
        rebuild();
      }
    };
    installControl($(ME).find('.whichpeer')[0], 'peer', 'peer_select', function(api){}, d);
  });
};

me.ready = function(){
};

function rebuild(){
  $(ME).find('.nebulaversion').html('<i>Checking remote peer...</i>');

  send_info(function(result){
    if (result.data) {
      $(ME).find('.nebulaversion').html(result.data.tag_name+" "+result.data.binary_name);
      $(ME).find('.releasemanagement').css('display', 'block');
      $(ME).find('.installappremotely').css('display', 'none');
      checkNetworks(result);
    }
    else {
      $(ME).find('.nebulaversion').html('The Newbound Nebula app is not running on this peer');
      $(ME).find('.releasemanagement').css('display', 'none');
      $(ME).find('.installappremotely').css('display', 'block');
      $(ME).find('.installednetworks').css('display', 'none');
    }
  }, me.peer);
};

$(ME).find('.addlighthousebutton').click(function(e){
  editLightHouse(null);
});

function editLightHouse(e){
  var d = {};
  var button = this;
      
  d.cb = function(api){
    var content = $(ME).find('.popmeup'); //el.find('.lhcontent');
    var d2 = {
      close: function(){
        document.body.api.ui.closePopup(d, function(api){});
      },
      save: function(data){
        document.body.api.ui.closePopup(d, function(api){});
        var lh = extractLightHouses();
        if (lh[data.peer]) {
          lh = lh[data.peer];
          var row = $(ME).find('.lhrow_'+data.peer);
          var tds = row.find('td');
          $(tds[1]).text(data.private_ip);
          $(tds[2]).text(data.public_ip);
          $(tds[3]).text(data.port);
        }
        else {
          var newhtml = $(buildLHRow(data.peer, data));
          newhtml.find('.editlighthousebutton').click(editLightHouse);
          newhtml.find('.deletelighthousebutton').click(deleteLightHouse);
          $(ME).find('.selectednetworklighthouses').find('tbody').append(newhtml);
        }
      }
    };
    if (e){
      var lh = $(button).closest('tr').find('.lighthouse_peer')[0];
      var peer = $(lh).text();
      d2.value = extractLightHouse(peer, lh);
      d2.value.peer = peer;
    }
    installControl(content[0], 'nebula', 'lighthouse', function(api){}, d2);
  };
  var el = $(ME).find('.popmeup');
  el.width(340);
  el.height(400);
  d.selector = el[0];
  closeselector: ".closelhbutton",
  d.modal = true;
  document.body.api.ui.popup(d, function(api){
    d.popupapi = api;
    d.cb();
  });
}

function nextIPAddress(){
  var n = Object.keys(me.members).length+2;
  var ip_address = $(ME).find('.networksubnet').val();
  var i = ip_address.indexOf('X');
  ip_address = ip_address.substring(0,i)+n+ip_address.substring(i+1);
  return ip_address;
}

$(ME).find('.addmemberbutton').click(function(e){
  var val = {
    "ip_address": nextIPAddress(),
    "groups": "",
    "peer": null
  };
  popupCredentials(val);
});
  
function popupCredentials(val){
  var d = {};
  var button = this;
  
  d.cb = function(api){
    var content = el; //.find('.popmeup');
    var d2 = {
      close: function(){
        document.body.api.ui.closePopup(d, function(api){});
      },
      save: function(data){
        document.body.api.ui.closePopup(d, function(api){});
        var servicename = me.selectednetwork.name;
        var target = d2.value.peer;
        var subnet = $(ME).find('.networksubnet').val();
        var ipaddress = d2.value.ip_address;
        var port = $(ME).find('.networkport').val();
        var owner = me.peer ? me.peer : me.mypeerid;
        var groups = d2.value.groups;
        send_add_member(servicename, target, ipaddress, groups, function(result){
          if (result.status != 'ok') alert(result.msg);
          else {
            var ca_crt = result.data.ca_crt;
            var host_crt = result.data.host_crt;
            var host_key = result.data.host_key;
            var lh = extractLightHouses();
            debugger;
            send_join_network(servicename, subnet, ipaddress, port, owner, ca_crt, host_crt, host_key, lh, groups, function(result){
              selectNetwork(me.selectednetwork.name);
              alert(JSON.stringify(result));
            }, target);
          }
        }, me.peer);
      }
    };
    d2.value = val;
    installControl(content[0], 'nebula', 'member', function(api){}, d2);
  };
  var el = $(ME).find('.popmeup');
  el.width(340);
  el.height(340);
  d.selector = el[0];
  d.modal = true;
  document.body.api.ui.popup(d, function(api){
    d.popupapi = api;
    d.cb();
  });
}

$(ME).find('.addnetworkbutton').click(function(e){
  var el = $(ME).find('.popmeup');
  el.width(340);
  el.height(340);
  var d = {
    modal: true,
    selector: el[0],
    closeselector: ".closenwbutton",
    save: function(data){
      $(d.closeselector).click();
      send_create_network(d.value.name, d.value.subnet, d.value.port, function(result){
        console.log(result);
        rebuild();
      }, me.peer);
    }
  }
  
  var n = Object.keys(me.networks).length;
  d.value = {
    "subnet": "192.168.10"+(n++)+".X/24",
    "name": "nebula"+(n++),
    "port": "424"+n
  };
  
  installControl(el[0], 'nebula', 'network', function(api){
    document.body.api.ui.popup(d, function(x){
    });
  }, d);
});

function selectNetwork(name){
  var rdn = me.networks[name];
  me.selectednetwork = rdn;
  $(ME).find('.selectednetworkname').text(name);
  var ownername = document.peers[rdn.owner] ? document.peers[rdn.owner].name : rdn.owner == me.mypeerid ? me.mypeername : rdn.owner;
  var peername = document.peers[me.peer] ? document.peers[me.peer].name : me.peer == null ? 'local' : me.peer;
  $(ME).find('.networkownerspan').text('Owner: '+ownername);
  $(ME).find('.selectedpeername').text(peername);
  var newhtml = '<label><div class="switch"><input id="appfilter-switch-1" type="checkbox" class="switch-input toggleservicebutton" '+(rdn.service ? ' checked' : '')+'/><span class="switch-label">Service</span></div>Service</label>';
  $(ME).find('.networkservicespan').html(newhtml).find('.toggleservicebutton').click(function(e){
    if (rdn.service) uninstallService();
    else installService();
  });
  var newhtml = '<label><div class="switch"><input id="appfilter-switch-2" type="checkbox" class="switch-input togglerunningbutton" '+(rdn.running ? ' checked' : '')+'/><span class="switch-label">Running</span></div>Running</label>';
  $(ME).find('.networkrunningspan').html(newhtml).find('.togglerunningbutton').click(function(e){
    if (rdn.running) stopService();
    else startService();
  });
  $(ME).find('.lighthousecheckbox').prop('checked', rdn.config.am_lighthouse);
  $(ME).find('.yaml-checkbox').prop('checked', rdn.config.use_yaml);
  $(ME).find('.networkhost').val(rdn.config.host).parent().addClass('is-dirty');
  $(ME).find('.networksubnet').val(rdn.config.subnet).parent().addClass('is-dirty');
  $(ME).find('.networkipaddr').val(rdn.config.ip_address).parent().addClass('is-dirty');
  $(ME).find('.networkport').val(rdn.config.port).parent().addClass('is-dirty');
  $(ME).find('.networkgroups').val(rdn.config.groups).parent().addClass('is-dirty');
  $(ME).find('.config-yml').val(rdn.config.yaml).parent().addClass('is-dirty');
  $(ME).find('.raw-yaml').css('display', rdn.config.use_yaml ? 'block' : 'none');
  
  newhtml = '<table class="lht-table" cellspacing="20px"><thead><tr><th class="lht-th">Peer</th><th class="lht-th">Private IP</th><th class="lht-th">Public IP</th><th>Port</th><th class="lht-th"></th><th class="mdl-data-table__cell--non-numeric"></th></tr></thead><tbody>';
  
  for (var id in rdn.config.lighthouses){
    var lh = rdn.config.lighthouses[id];
    newhtml += buildLHRow(id, lh);
  }
  
  newhtml += '</tbody></table>';
  var el = $(ME).find('.selectednetworklighthouses');
  el.html(newhtml);
  
  el.find('.editlighthousebutton').click(editLightHouse);
  el.find('.deletelighthousebutton').click(deleteLightHouse);
  
  send_members(name, function(result){
    me.members = result.data;
    newhtml = '<table class="mdl-data-table mdl-js-data-table mdl-shadow--2dp"><thead><tr><th class="mdl-data-table__cell--non-numeric">Peer</th><th class="mdl-data-table__cell--non-numeric">Private IP</th><th class="mdl-data-table__cell--non-numeric">groups</th><th class="mdl-data-table__cell--non-numeric"></th><th class="mdl-data-table__cell--non-numeric"></th></tr></thead><tbody>';
    for (var id in result.data) if (result.data[id].ip_address){
      var p = document.peers[id];
      var name = p ? p.name + '<br><span class="member_peer">'+id+'</span>' : id;
      var ip = result.data[id].ip_address;
      var groups = result.data[id].groups;
      if (!groups) groups = '';
      newhtml += '<tr class="memberrow_'+id+'" data-peer="'+id+'"><td class="mdl-data-table__cell--non-numeric">'+name+'</td><td class="mdl-data-table__cell--non-numeric">'+ip+'</td><td class="mdl-data-table__cell--non-numeric">'+groups+'</td><td class="mdl-data-table__cell--non-numeric"><button class="updatememberbutton mdl-button mdl-js-button mdl-button--icon"><i class="material-icons">update</i></button></td><td class="mdl-data-table__cell--non-numeric"><button class="deletememberbutton mdl-button mdl-js-button mdl-button--icon"><i class="material-icons">delete</i></button></td></tr>';
    }
    newhtml += '</tbody></table>';
    var el = $(ME).find('.selectednetworkmembers');
    el.html(newhtml);
    el.find('.updatememberbutton').click(updateMember);
    el.find('.deletememberbutton').click(deleteMember);
  }, me.peer);
}

$(ME).find('.saveconfigbutton').click(function(e){
  var lh = extractLightHouses();
  var d = {};
  d.host = $(ME).find('.networkhost').val();
  d.subnet = $(ME).find('.networksubnet').val();
  d.ip_address = $(ME).find('.networkipaddr').val();
  d.port = $(ME).find('.networkport').val();
  d.groups = $(ME).find('.networkgroups').val();
  d.am_lighthouse = $(ME).find('.lighthousecheckbox').prop("checked");
  d.lighthouses = lh;
  d.use_yaml = $(ME).find('.yaml-checkbox').prop("checked");
  if (d.use_yaml) d.yaml = $(ME).find('.config-yml').val();
  send_save_config(me.selectednetwork.name, d, function(result){
    alert(JSON.stringify(result));
  }, me.peer);
});

function extractLightHouses(){
  var lh = {};
  var els = $(ME).find('.lighthouse_peer');
  var n = els.length;
  for (var i=0;i<n;i++){
    var el = els[i];
    var peer = $(el).html();
    lh[peer] = extractLightHouse(peer, el);
  }
  return lh;
}

function extractLightHouse(peer, el){
  var d = {};
  el = $(el).parent().next();
  d.private_ip = el.text();
  el = el.next();
  d.public_ip = el.text();
  el = el.next();
  d.port = el.text(); //Number(el.text());
  return d;
}

function installService(){
  send_install_service(me.selectednetwork.name, function(result){
    alert(JSON.stringify(result));
  }, me.peer);
}

function uninstallService(){
  send_uninstall_service(me.selectednetwork.name, function(result){
    alert(JSON.stringify(result));
  }, me.peer);
}

function startService(){
  send_start_service(me.selectednetwork.name, function(result){
    alert(JSON.stringify(result));
  }, me.peer);
}

function stopService(){
  send_stop_service(me.selectednetwork.name, function(result){
    alert(JSON.stringify(result));
  }, me.peer);
}


function updateMember(){
  alert("NO");
}

function deleteMember(){
  alert("NO");
}

function deleteLightHouse(e){
  $(this).closest('tr').remove();
}

function buildLHRow(id, lh){
    var p = getByProperty(document.peers, "id", id);
    var name = p ? p.name + '<br><span class="lighthouse_peer">'+id+'</span>' : id;
    return '<tr class="lhrow_'+id+'"><td class="lht-td">'+name+'</td><td class="lht-td">'+lh.private_ip+'</td><td class="lht-td">'+lh.public_ip+'</td><td>'+lh.port+'</td><td class="lht-td"><img src="../app/asset/app/pencil_icon.png" class="editlighthousebutton roundbutton-small"></td><td class="lht-td"><img src="../app/asset/app/delete_icon.png" class="deletelighthousebutton roundbutton-small"></td></tr>';
}

$(ME).find('.backbutton').click(function(){
  $(ME).find('.networkpage').css('display', 'none');
  $(ME).find('.homepage').css('display', 'block');
});

$(ME).find('.refreshversion').click(rebuild);

$(ME).find('.installupdatebutton').click(function(e){
  $(this).css('display', 'none');
  var url = $(ME).find('.downloadselect').find('select').val();
  var el = $(ME).find('.updatemsg').find('select');
  var version = el[0].options[el[0].selectedIndex].text;
  send_install_release(url, version, function(result){
    if (result.status != 'ok') me.ui.snackbarMsg("ERROR: "+result.msg, 600);
    else me.ui.snackbarMsg("Installation complete");
    $(ME).find('.installupdatebutton').css('display', 'block');
    rebuild();
  }, me.peer);
});

$(ME).find('.installappbutton').click(function(e){
  $(this).css('display', 'none');
  $(ME).find('.nebulaversion').html('<i>Installing Nebula app...</i>');
  var d = "uuid="+me.mypeerid+"&lib=nebula&guid="+me.UUID;
  json("../peer/remote/"+me.peer+"/dev/install_lib", d, function(result){
    if (result.status != "ok") me.ui.snackbarMsg("ERROR: "+result.msg);
    else {
    
      json('../peer/remote/'+me.peer+'/app/settings', 'settings={}', function(result){
        if (result.status != 'ok') me.ui.snackbarMsg("ERROR: "+result.msg);
        else {
          let applist = result.data.apps;
          if (applist != '') applist += ',';
          applist += 'nebula';
          var d = {
            apps: applist
          };
          json('../peer/remote/'+me.peer+'/app/settings', 'settings='+encodeURIComponent(JSON.stringify(d)), function(result){
            if (result.status != 'ok') me.ui.snackbarMsg("ERROR: "+result.msg);
            else {
              var loc = window.location.href;
              window.location.href = loc;
            }
          });
        }
      });
    
    }
  });
});

$(ME).find('.checkupdatebutton').click(function(e){
  $(this).css('display', 'none');
  $(ME).find('.updatemsg').html('<i>checking for update...</i>');
  $.getJSON('https://api.github.com/repos/slackhq/nebula/releases', function(result){
    me.releases = {};
    for (var i in result){
      var r = result[i];
      var tag = r.tag_name;
      var id = r.id;
      me.releases[id] = r;
    }
    var d = {
      "list": result,
      "label": "Select Version",
      "cb": selectVersion
    };
    installControl($(ME).find('.updatemsg')[0], 'app', 'select', function(api){
      selectVersion(result[0].id);
    }, d);
  });
});

function selectVersion(val){
  var r = me.releases[val];
  var binaries = [];
  for (var i in r.assets){
    var o = r.assets[i];
    if (o.name.startsWith('nebula-')){
      var d = {
        "name": o.name.substring(7, o.name.length - 7),
        "id": o.browser_download_url
      };
      binaries.push(d);
    }
  }
  var d = {
    "list": binaries,
    "label": "Select Binary"
  };
  installControl($(ME).find('.downloadselect')[0], 'app', 'select', function(api){
    $(ME).find('.installupdatebutton').css('display', 'block');
  }, d);
}

function checkNetworks(result){
  me.networks = {};
  $(ME).find('.installednetworks').css('display', 'block');
  if (result.data.networks && result.data.networks.length > 0){
    var newhtml = '';
    for (var i in result.data.networks){
      var rdn = result.data.networks[i];
      me.networks[rdn.name] = rdn;
      var owner = rdn.owner == 'local' ? 'local' : document.peers[rdn.owner] ? document.peers[rdn.owner].name : rdn.owner == me.mypeerid ? me.mypeername : rdn.owner;
      newhtml += ' <button data-service="'+rdn.name+'" class="networklistbutton '+(rdn.running ? 'colored' : 'accent')+'button networkbutton">'+rdn.name+' ('+owner+')</button>';
      newhtml += '<br>';
    }
    $(ME).find('.foundnetworks').html(newhtml).find('button').click(function(e){
      $(ME).find('.homepage').css('display', 'none');
      $(ME).find('.networkpage').css('display', 'block');
      var name = $(this).data('service');
      selectNetwork(name);
    });
  }
  else $(ME).find('.foundnetworks').html('<i>No networks</i>');
}

$(document).click(function(event) {
  window.lastElementClicked = event.target;
  window.lastClick = event;
});
