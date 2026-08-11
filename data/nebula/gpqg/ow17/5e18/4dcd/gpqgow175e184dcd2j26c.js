var me = this;
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (!ME.DATA.value) ME.DATA.value = {};
  var v = ME.DATA.value;
  $(ME).find('#lhprivateip').val(v.private_ip || '');
  $(ME).find('#lhpublicip').val(v.public_ip || '');
  $(ME).find('#lhport').val(v.port || '');

  var el = $(ME).find('.lighthousepeerselector');
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

$(ME).find('.savelhbutton').click(function(){
  var v = {
    peer: ME.DATA.value.peer,
    private_ip: ($(ME).find('#lhprivateip').val() || '').trim(),
    public_ip: ($(ME).find('#lhpublicip').val() || '').trim(),
    port: ('' + $(ME).find('#lhport').val()).trim()
  };
  if (!v.peer) return err('Pick the peer that acts as the lighthouse.');
  if (!v.private_ip) return err('The private (overlay) IP is required.');
  if (!v.public_ip) return err('The public IP is required — a lighthouse must be reachable from the internet.');
  if (!v.port || isNaN(Number(v.port))) return err('The lighthouse’s UDP port is required.');
  err(null);
  if (ME.DATA.save) ME.DATA.save(v);
});

$(ME).find('.closelhbutton, .cancellhbutton').click(function(){
  if (ME.DATA.close) ME.DATA.close();
});
