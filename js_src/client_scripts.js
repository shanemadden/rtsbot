const output = `<span>injecting client scripts...</span> 
<script>
(function(){
    if (window.client_scripts_injected) return;
    let Api = angular.element($('section.game')).injector().get('Api');  
    let Connection = angular.element($('body')).injector().get('Connection');
    let roomScope = angular.element(document.getElementsByClassName("room ng-scope")).scope();
    Connection.onRoomUpdate(roomScope, function() {
        if (roomScope.Room.selectedObject) {
            let tick = roomScope.Room.gameTime;
            let object_id = roomScope.Room.selectedObject._id;
            if ((object_id !== window.selection_tracker_object) || (window.selection_tracker_tick && window.selection_tracker_tick + 5 <= tick)) {
                window.selection_tracker_object = object_id;
                window.selection_tracker_tick = tick;
                Api.post('user/console',{
                    expression: "update_selected_object("+tick+", '"+object_id+"');'client object selection updated';",
                    shard: roomScope.Room.shardName,
                    hidden: true
                });
            }
        }
    });

    let cursorLayer = angular.element(document.getElementsByClassName("cursor-layer"))[0];
    cursorLayer.addEventListener("contextmenu", function(e){
        if (roomScope.Room.cursorPos) {
            e.preventDefault();
            let room_name = roomScope.Room.roomName;
            let x = roomScope.Room.cursorPos.x;
            let y = roomScope.Room.cursorPos.y;
            let expr;
            if (roomScope.Room.selectedObject) {
                let object_id = roomScope.Room.selectedObject._id;
                expr = "right_click_position('"+room_name+"', "+x+", "+y+", '"+object_id+"');'right click sent';";
            } else {
                expr = "right_click_position('"+room_name+"', "+x+", "+y+");'right click sent';";
            }
            Api.post('user/console',{
                expression: expr,
                shard: roomScope.Room.shardName,
                hidden: true
            });
        }
    });

    window.client_scripts_injected = true;
})()
</script>`.replace(/(\r\n|\n|\r)\t+|(\r\n|\n|\r) +|(\r\n|\n|\r)/gm, '')

global.script_inject = function() {
    console.log(output);
}

// call this directly if the session wasn't running when the last global reset occurred!
script_inject();
