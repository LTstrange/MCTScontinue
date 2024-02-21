const { invoke } = window.__TAURI__.tauri;
invoke('init_game');

const board = document.getElementById("game-board");
const stones = document.getElementById("stones");
let cursor;

let gameState = [];

// left click
stones.addEventListener("click", (e) => {
    const [clickX, clickY] = getMouseClickCoord(e);
    invoke('click', { x: clickX - 1, y: clickY - 1 }).then((state) => {
        gameState = state;
        draw();
    })
})

// right click
stones.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    invoke('undo').then((state) => {
        gameState = state;
        draw();
    })
    return false;
},
    { capture: true });

function draw() {
    stones.innerHTML = "";
    drawBoard();
    drawLastStoneMarker();
    // create cursor
    const color = gameState.length % 2 === 0 ? "black" : "white";
    const element = createStone(color);
    element.id = "cursor";
    cursor = element;
    stones.appendChild(element);
    cursor.style.opacity = 0;

}
draw();


function drawBoard() {
    for (let i = 0; i < gameState.length; i++) {
        const color = i % 2 === 0 ? "black" : "white";
        const x = Math.floor(gameState[i] % 15) + 1;
        const y = Math.floor(gameState[i] / 15) + 1;

        const stone = createStone(color);
        setPosition(stone, x, y);
        stones.appendChild(stone);
    }
}

function drawLastStoneMarker() {
    if (gameState.length === 0) return;
    const lastStone = gameState[gameState.length - 1];
    const x = Math.floor(lastStone % 15) + 1;
    const y = Math.floor(lastStone / 15) + 1;
    const color = gameState.length % 2 === 1 ? "fff" : "000";

    const marker = document.createElement("div");
    marker.id = "last-stone";
    marker.style.gridRow = y;
    marker.style.gridColumn = x;
    marker.style.background = `linear-gradient(45deg, #${color}, #${color} 50%, #0000 50%, #0000)`
    stones.appendChild(marker);
}

function createStone(color) {
    const stone = document.createElement("img");
    stone.className = "stone";
    switch (color) {
        case "black":
            stone.src = "assets/black_stone.svg";
            stone.alt = "black_stone";
            break;
        case "white":
            stone.src = "assets/white_stone.svg";
            stone.alt = "white_stone";
            break;
        default:
    }
    return stone;
}


function setPosition(stone, x, y) {
    stone.style.gridRow = y;
    stone.style.gridColumn = x;
}

stones.addEventListener("mousemove", (e) => {
    const [clickX, clickY] = getMouseClickCoord(e);
    setPosition(cursor, clickX, clickY);
    cursor.style.opacity = 0.5;
})


stones.addEventListener("mouseleave", (e) => {
    cursor.style.opacity = 0;
})


function getMouseClickCoord(e) {
    const rect = stones.getBoundingClientRect();
    const clickX = Math.ceil((e.clientX - rect.left) * 15 / rect.width);
    const clickY = Math.ceil((e.clientY - rect.top) * 15 / rect.height);
    return [clickX, clickY];
}

function step() {
    invoke('step')
        .then((state) => {
            gameState = state;
            draw();
        })
}

function restart() {
    invoke('restart').then((state) => {
        gameState = state;
        draw();
    })
}