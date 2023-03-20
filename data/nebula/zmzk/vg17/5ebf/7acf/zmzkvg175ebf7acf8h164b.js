var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  componentHandler.upgradeAllRegistered();
  if (!ME.DATA.value) ME.DATA.value = {
    "name": "nebula1",
    "subnet": "192.168.100.X/24",
    "port": 4242
  };
  $(ME).find('#nwname').val(ME.DATA.value.name).parent().addClass('is-dirty');
  $(ME).find('#nwsubnet').val(ME.DATA.value.subnet).parent().addClass('is-dirty');
  $(ME).find('#nwport').val(ME.DATA.value.port).parent().addClass('is-dirty');
};

$(ME).find('.createnewnetworkbutton').click(function(e){
  if (ME.DATA.save) ME.DATA.save();
});

$(ME).find('#nwname').change(function(e){
  ME.DATA.value.name = $(this).val();
});

$(ME).find('#nwsubnet').change(function(e){
  ME.DATA.value.subnet = $(this).val();
});

$(ME).find('#nwport').change(function(e){
  ME.DATA.value.port = $(this).val();
});
