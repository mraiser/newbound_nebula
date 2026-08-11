var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (!ME.DATA.value) ME.DATA.value = {};
  else {
    $(ME).find('#lhprivateip').val(ME.DATA.value.private_ip).parent().addClass('is-dirty');
    $(ME).find('#lhpublicip').val(ME.DATA.value.public_ip).parent().addClass('is-dirty');
    $(ME).find('#lhport').val(ME.DATA.value.port).parent().addClass('is-dirty');
  }
  
  var el = $(ME).find('.lighthousepeerselector');
  var d = {
    "local": false,
    "connectedonly": true,
    "value": ME.DATA.value.peer,
    "ready": function(val){
      ME.DATA.value.peer = el.find('select').val();
    },
    "cb": function(val){
      ME.DATA.value.peer = val;
    }
  }
  installControl(el[0], 'peer', 'peer_select', function(api){}, d);
};

me.validate = function(){
  var ok = false;
  var msg = "unknown error";
  if (!ME.DATA.value.private_ip) msg = "Private IP is required";
  else if (!ME.DATA.value.public_ip) msg = "Public IP is required";
  else if (!ME.DATA.value.port) msg = "Port is required";
  else ok = true;
  if (!ok) alert(msg);
  return ok;
}

$(ME).find('#lhprivateip').change(function(e){
  ME.DATA.value.private_ip = $(this).val();
});

$(ME).find('#lhpublicip').change(function(e){
  ME.DATA.value.public_ip = $(this).val();
});

$(ME).find('#lhport').change(function(e){
  ME.DATA.value.port = $(this).val();
});

$(ME).find('.closelhbutton').click(function(e){
  if (ME.DATA.close) ME.DATA.close();
});

$(ME).find('.savelhbutton').click(function(e){
  if (me.validate()) ME.DATA.save(ME.DATA.value);
});
