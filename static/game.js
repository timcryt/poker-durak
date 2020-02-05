var socket = new WebSocket('ws://127.0.0.1:8000/ws', 'echo');
var cards = [];
var is_your_turn = false;
var deck_size = 0;
var net_time = 0;

function send(data) {{
    if (JSON.parse(data) != 'Ping') {
        document.getElementById('req').innerText = data;
    }
    socket.send(data);
}}

function refresh_state(data) {
    if (data == JSON.parse('"Passive"')) {
        set_state(false);
        document.getElementById('comb').innerText = '';
        document.getElementById('board').innerText = '';
    } else {
        data = data['Active']
        set_state(true);
        document.getElementById('comb').innerHTML = print_cards(data['comb']['cards']);
        document.getElementById('board').innerHTML = print_cards(data['cards']);
    }
}

function suit2num(a) {
    if (a == '♠') return 0;
    if (a == '♣') return 1;
    if (a == '♦') return 2;
    if (a == '♥') return 3;
}

function rank2num(a) {
    if (a == '2') return 0;
    if (a == '3') return 1;
    if (a == '4') return 2;
    if (a == '5') return 3;
    if (a == '6') return 4;
    if (a == '7') return 5;
    if (a == '8') return 6;
    if (a == '9') return 7;
    if (a == '10') return 8;
    if (a == 'J') return 9;
    if (a == 'Q') return 10;
    if (a == 'K') return 11;
    if (a == 'A') return 12;       
}

function card_compare(a, b) {
    return (suit2num(a['suit']) + rank2num(a['rank']) * 4) > (suit2num(b['suit']) + rank2num(b['rank']) * 4);
}

function print_cards(cards) {
    s = '';
    cards.sort(card_compare).forEach(card => {
        t = card['rank'] + ' ' + card['suit']
        s += `<button onclick="add_card('${t}')">+</button>` + t + '<br />'
    }); 
    return s;
}

socket.onmessage = function(event) {{
    net_time = 0;
    refresh_netstat();

    data = event.data;
    if (data != '"Pong"') {
        data = JSON.parse(data);
        if (data['YourCards']) {
            document.getElementById('WaitDiv').style.display = 'None'
            document.getElementById('GameDiv').style.display = ''
            document.getElementById('cards').innerHTML = print_cards(data['YourCards'][0]);
            document.getElementById('deck_size').innerText = JSON.stringify(data['YourCards'][1]);
            deck_size = data['YourCards'][1] + 0;
        } else if (data['YourTurn']) {
            data = data['YourTurn'];
            document.getElementById('your_turn').innerText = 'Да';
            is_your_turn = true;
            document.getElementById('cards').innerHTML = print_cards(data[1]);
            document.getElementById('deck_size').innerText = JSON.stringify(data[2]);
            deck_size = data[2] + 0;
            refresh_state(data[0]);    
        } else if (data['YouMadeStep']) {
            data = data['YouMadeStep'];
            document.getElementById('your_turn').innerText = 'Нет';
            is_your_turn = false;
            document.getElementById('cards').innerHTML = print_cards(data[1]);
            document.getElementById('deck_size').innerText = JSON.stringify(data[2]);
            deck_size = data[2] + 0;
            refresh_state(data[0]);
            clear_cards();
        } else if (data == 'GameWinner') {
            location.replace('/game_winner');
        } else if (data == 'GameLoser') {
            location.replace('/game_loser');
        } else if (data['ID']) {
            document.getElementById('GamePID').innerText = JSON.stringify(data['ID']);
        }
        document.getElementById('resp').innerText = event.data;
    }
}}

function set_state(state) {
    if (state) {
        document.getElementById('state').innerText = 'Активное';
        document.getElementById('GetCardBut').style.visibility = 'hidden';
        document.getElementById('GiveCombBut').style.visibility = 'hidden';
        document.getElementById('TransCombBut').style.visibility = 'visible';
        document.getElementById('GetCombBut').style.visibility = 'visible'; 
        document.getElementById('StateActDiv').style.display = '';    
    } else {
        document.getElementById('state').innerText = 'Пассивное';
        document.getElementById('GetCardBut').style.visibility = 'visible';
        document.getElementById('GiveCombBut').style.visibility = 'visible';
        document.getElementById('TransCombBut').style.visibility = 'hidden';
        document.getElementById('GetCombBut').style.visibility = 'hidden';
        document.getElementById('StateActDiv').style.display = 'none';
    }

    if (!is_your_turn) {
        document.getElementById('GetCardBut').style.visibility = 'hidden';
        document.getElementById('GiveCombBut').style.visibility = 'hidden';
        document.getElementById('TransCombBut').style.visibility = 'hidden';
        document.getElementById('GetCombBut').style.visibility = 'hidden';        
    }
    if (deck_size == 0) {
        document.getElementById('GetCardBut').style.visibility = 'hidden';
    }
}

function add_card(card) {
    suit_rank = card.split(' ');
    suit = suit_rank[1];
    rank = suit_rank[0];
    if (!cards.find(function(item, _, _) {
        return item['rank'] == rank && item['suit'] == suit;
    })) {
        cards.push({rank: rank, suit: suit});
        document.getElementById('your_cards').innerText += card + '\n'
    }
}

function clear_cards() {
    cards = []
    document.getElementById('your_cards').innerText = ''
}

function refresh_netstat() {
    if (net_time >= 15) {
        document.getElementById('NetStat').style.color = 'Red';
        document.getElementById('NetStat').innerHTML = 'Обрыв соединения <a href="/">На главную страницу</a>';
        socket.close();
    } else if (net_time >= 5) {
        document.getElementById('NetStat').style.color = 'Orange';
        document.getElementById('NetStat').innerText = 'Проблемы со связью';
    } else {
        document.getElementById('NetStat').style.color = 'Green';
        document.getElementById('NetStat').innerText = 'Соединение установлено';
    }
}

heartbit = function() {
    send('"Ping"');
    net_time += 1;
    refresh_netstat();
}