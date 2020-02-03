var socket = new WebSocket("ws://localhost:8000/ws", "echo");

function send(data) {{
    socket.send(data);
}}

function refresh_state(data) {
    console.log(data);
    if (data == JSON.parse('"Passive"')) {
        document.getElementById('state').innerText = 'Пассивное';
        document.getElementById('comb').innerText = '';
        document.getElementById('board').innerText = '';
    } else {
        data = data['Active']
        document.getElementById('state').innerText = 'Активное';
        document.getElementById('comb').innerText = JSON.stringify(data['comb']['cards']);
        document.getElementById('board').innerText = JSON.stringify(data['cards']);
    }
}

socket.onmessage = function(event) {{
    data = event.data;
    if (data != "\"Pong\"") {
        data = JSON.parse(data);
        if (data["YourCards"]) {
            document.getElementById('cards').innerText = JSON.stringify(data["YourCards"][0]);
            document.getElementById('deck_size').innerText = JSON.stringify(data["YourCards"][1]);
        } else if (data["YourTurn"]) {
            document.getElementById('your_turn').innerText = 'Да';
            document.getElementById('deck_size').innerText = JSON.stringify(data['YourTurn'][1]);
            refresh_state(data['YourTurn'][0]);
            
        } else if (data["YouMadeStep"]) {
            data = data["YouMadeStep"];
            document.getElementById('your_turn').innerText = 'Нет';
            document.getElementById('cards').innerText = JSON.stringify(data[1]);
            document.getElementById('deck_size').innerText = JSON.stringify(data[2]);
            refresh_state(data[0]);
        }
        document.getElementById('result').innerText = event.data ;
    }
}}
heartbit = function() {
    send("\"Ping\"");
}