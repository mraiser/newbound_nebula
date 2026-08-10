var me = this;
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (!ME.DATA.value) ME.DATA.value = { name: 'nebula1', subnet: '192.168.100.X/24', port: '4242' };
  $(ME).find('#nwname').val(ME.DATA.value.name);
  $(ME).find('#nwsubnet').val(ME.DATA.value.subnet);
  $(ME).find('#nwport').val(ME.DATA.value.port);
};

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
