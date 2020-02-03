var socket = new WebSocket('ws://localhost:8000/ws', 'echo');
var cards = [];

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
    if (a == 'Spades') return 0;
    if (a == 'Clubs') return 1;
    if (a == 'Diamonds') return 2;
    if (a == 'Hearts') return 3;
}

function rank2num(a) {
    if (a == 'Two') return 0;
    if (a == 'Three') return 1;
    if (a == 'Four') return 2;
    if (a == 'Five') return 3;
    if (a == 'Six') return 4;
    if (a == 'Seven') return 5;
    if (a == 'Eight') return 6;
    if (a == 'Nine') return 7;
    if (a == 'Ten') return 8;
    if (a == 'Jack') return 9;
    if (a == 'Queen') return 10;
    if (a == 'King') return 11;
    if (a == 'Ace') return 12;       
}

function card_compare(a, b) {
    return (suit2num(a['suit']) * 13 + rank2num(a['rank'])) > (suit2num(b['suit']) * 13 + rank2num(b['rank']));
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
    data = event.data;
    if (data != '"Pong"') {
        data = JSON.parse(data);
        if (data['YourCards']) {
            document.getElementById('cards').innerHTML = print_cards(data['YourCards'][0]);
            document.getElementById('deck_size').innerText = JSON.stringify(data['YourCards'][1]);
        } else if (data['YourTurn']) {
            data = data['YourTurn'];
            document.getElementById('your_turn').innerText = 'Да';
            document.getElementById('cards').innerHTML = print_cards(data[1]);
            document.getElementById('deck_size').innerText = JSON.stringify(data[2]);
            refresh_state(data[0]);
            
        } else if (data['YouMadeStep']) {
            data = data['YouMadeStep'];
            document.getElementById('your_turn').innerText = 'Нет';
            document.getElementById('cards').innerHTML = print_cards(data[1]);
            document.getElementById('deck_size').innerText = JSON.stringify(data[2]);
            refresh_state(data[0]);
            clear_cards();
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

heartbit = function() {
    send('"Ping"');
}