var me = this;
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (!ME.DATA.value) ME.DATA.value = { name: 'nebula1', subnet: '192.168.100.X/24', port: '4242' };
  $(ME).find('#nwname').val(ME.DATA.value.name);
  $(ME).find('#nwsubnet').val(ME.DATA.value.subnet);
  $(ME).find('#nwport').val(ME.DATA.value.port);
  checkPort();
};

// Warn (#8) when another network here already uses the port, or anything on
// the machine has it bound - nebula would fail to start on it.
var portTimer = null;
function checkPort(){
  var el = $(ME).find('.portwarn');
  var p = ('' + $(ME).find('#nwport').val()).trim();
  el.attr('hidden', true).text('');
  if (!p || isNaN(Number(p))) return;
  var other = ME.DATA.usedPorts && ME.DATA.usedPorts[p];
  if (other) { el.removeAttr('hidden').text('Network ' + other + ' already uses UDP ' + p + ' here. Pick another port.'); return; }
  if (!ME.DATA.checkPort) return;
  ME.DATA.checkPort(p, function(r){
    if (('' + $(ME).find('#nwport').val()).trim() != p) return;   // typed on meanwhile
    if (r && r.bound) el.removeAttr('hidden').text('UDP ' + p + ' is already in use on this machine (another program or tunnel). nebula won\u2019t start on it \u2014 pick another port.');
  });
}
$(ME).find('#nwport').on('input', function(){
  clearTimeout(portTimer);
  portTimer = setTimeout(checkPort, 300);
});

function err(msg){
  var el = $(ME).find('.nbdlg-err');
  if (!msg) { el.attr('hidden', true); return; }
  el.removeAttr('hidden').text(msg);
}

$(ME).find('.createnewnetworkbutton').click(function(){
  var v = {
    name: ($(ME).find('#nwname').val() || '').trim(),
    subnet: ($(ME).find('#nwsubnet').val() || '').trim(),
    port: ('' + $(ME).find('#nwport').val()).trim()
  };
  if (!v.name) return err('A name is required.');
  if (!/^[A-Za-z0-9_-]+$/.test(v.name)) return err('Names are letters, digits, - and _ (it becomes the service and tun device name).');
  if (v.subnet.indexOf('X') == -1 || v.subnet.indexOf('/') == -1) return err('The subnet needs an X placeholder and a /prefix, like 192.168.100.X/24.');
  if (!v.port || isNaN(Number(v.port))) return err('A UDP port is required.');
  err(null);
  if (ME.DATA.save) ME.DATA.save(v);
});

$(ME).find('.closenwbutton, .cancelbutton').click(function(){
  if (ME.DATA.close) ME.DATA.close();
});
