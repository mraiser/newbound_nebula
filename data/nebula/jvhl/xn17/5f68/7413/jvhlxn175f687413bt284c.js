var me = this;
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (!ME.DATA.value) ME.DATA.value = {};
  var v = ME.DATA.value;
  $(ME).find('#memberipaddress').val(v.ip_address || '');
  $(ME).find('#membergroups').val(v.groups || '');

  var el = $(ME).find('.memberpeerselector');
  installControl(el[0], 'peer', 'peer_select', function(){}, {
    local: false,
    connectedonly: true,
    value: v.peer,
    ready: function(){ v.peer = el.find('select').val(); },
    cb: function(val){ v.peer = val; }
  });
};

function err(msg){
  var el = $(ME).find('.nbdlg-err');
  if (!msg) { el.attr('hidden', true); return; }
  el.removeAttr('hidden').text(msg);
}

$(ME).find('.createnewmemberbutton').click(function(){
  var v = {
    peer: ME.DATA.value.peer,
    ip_address: ($(ME).find('#memberipaddress').val() || '').trim(),
    groups: ($(ME).find('#membergroups').val() || '').trim()
  };
  if (!v.peer) return err('Pick the peer that will join the network.');
  if (!v.ip_address || v.ip_address.indexOf('/') == -1) return err('The member needs an overlay IP with a /prefix, like 192.168.100.7/24.');
  if (v.groups && /\s/.test(v.groups)) return err('Groups are comma separated with no spaces.');
  err(null);
  if (ME.DATA.save) ME.DATA.save(v);
});

$(ME).find('.closememberbutton, .cancelmemberbutton').click(function(){
  if (ME.DATA.close) ME.DATA.close();
});
