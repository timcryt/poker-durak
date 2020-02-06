var socket = new WebSocket('ws://{host}/ws', 'echo');
var cards = new Set();
var is_your_turn = false;
var deck_size = 0;
var net_time = 0;
var timeout = 0;

const SUIT = 1;
const RANK = 0;

function send(data) {{
    if (JSON.parse(data) != 'Ping') {
        document.getElementById('req').innerText = data;
    }
    socket.send(data);
}}

function parse_cards() {
    cards_arr = []
    cards.forEach(card =>
        cards_arr.push(card.split(' '))
    )
    return cards_arr
}

function without(a, b) {
    return a.filter(card =>
        b.find((card_b, _index, _) => 
            card[0] == card_b[0] && card[1] == card_b[1]
        ) == undefined
    );
}

function refresh_state(data) {
    if (data == JSON.parse('"Passive"')) {
        set_state(false);
        document.getElementById('comb').innerText = '';
        document.getElementById('board').innerText = '';
    } else {
        data = data['Active']
        set_state(true);
        document.getElementById('comb').innerHTML = print_cards(data['comb']['cards']);
        document.getElementById('board').innerHTML = print_cards(without(data['cards'], data['comb']['cards']));
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
    return (suit2num(a[SUIT]) + rank2num(a[RANK]) * 4) > (suit2num(b[SUIT]) + rank2num(b[RANK]) * 4);
}

function card_from(card) {
    return [
        "🂢", "🃒", "🃂", "🂲",
        "🂣", "🃓", "🃃", "🂳",
        "🂤", "🃔", "🃄", "🂴",
        "🂥", "🃕", "🃅", "🂵",
        "🂦", "🃖", "🃆", "🂶",
        "🂧", "🃗", "🃇", "🂷",
        "🂨", "🃘", "🃈", "🂸",
        "🂩", "🃙", "🃉", "🂹",
        "🂪", "🃚", "🃊", "🂺",
        "🂫", "🃛", "🃋", "🂻",
        "🂭", "🃝", "🃍", "🂽",
        "🂮", "🃞", "🃎", "🂾",
        "🂡", "🃑", "🃁", "🂱",
    ][rank2num(card[RANK]) * 4 + suit2num(card[SUIT])];
}

function print_cards(cards) {
    s = '';
    cards.sort(card_compare).forEach(card => {
        t = card[RANK] + ' ' + card[SUIT]
        c = card_from(card)
        s += `<button id="${t}" onclick="add_card('${t}')" style="font-size: 60px; color: black; background-color: white">${c}</button>`
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
            document.getElementById('GameDiv').style.display = '';
            document.getElementById('cards').innerHTML = print_cards(data['YourCards'][0]);
            document.getElementById('deck_size').innerText = JSON.stringify(data['YourCards'][1]);
            deck_size = data['YourCards'][1] + 0;
        } else if (data['YourTurn']) {
            data = data['YourTurn'];
            document.getElementById('your_turn').innerText = 'Да';
            timeout = data[4];
            is_your_turn = true;
            document.getElementById('cards').innerHTML = print_cards(data[1]);
            document.getElementById('deck_size').innerText = JSON.stringify(data[2]);
            document.getElementById('opponent_deck').innerText = JSON.stringify(data[3]);
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
            cards.clear();
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
    if (!cards.has(card)) {
        document.getElementById(card).style.backgroundColor = 'green';
        document.getElementById(card).style.color = 'white';
        cards.add(card);
    } else {
        document.getElementById(card).style.backgroundColor = 'white';
        document.getElementById(card).style.color = 'black';
        cards.delete(card);
    }
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

function refresh_timeout() {
    if (is_your_turn) {
        document.getElementById('TimeOut').style.display = '';
        document.getElementById('TimeOut').innerText = timeout;
    } else {
        document.getElementById('TimeOut').style.display = 'none';
    }
}

heartbit = function() {
    send('"Ping"');
    net_time += 1;
    refresh_netstat();
    timeout -= 1;
    refresh_timeout();
}