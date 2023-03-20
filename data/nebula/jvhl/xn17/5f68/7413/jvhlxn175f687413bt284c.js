var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (ME.DATA.value){
    $(ME).find('#memberipaddress').val(ME.DATA.value.ip_address).parent().addClass('is-dirty');
    $(ME).find('#membergroups').val(ME.DATA.value.groups).parent().addClass('is-dirty');
  }
  else ME.DATA.value = {};
  
  var el = $(ME).find('.memberpeerselector');
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

$(ME).find('.closememberbutton').click(function(e){
  if (ME.DATA.close) ME.DATA.close();
});

$(ME).find('.createnewmemberbutton').click(function(e){
  if (ME.DATA.save) ME.DATA.save();
});

$(ME).find('#memberipaddress').change(function(e){
  ME.DATA.value.ip_address = $(this).val();
});

$(ME).find('#membergroups').change(function(e){
  ME.DATA.value.groups = $(this).val();
});

