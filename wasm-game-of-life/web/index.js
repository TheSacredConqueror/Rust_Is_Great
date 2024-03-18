import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#addfad";
const ALIVE_COLOR = "#78184a";

const universe = Universe.new();
const width = universe.get_width();
const height = universe.get_height();

const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');
const drawGrid = () => {
	ctx.beginPath();
	ctx.strokeStyle = GRID_COLOR;

	// Vertical lines.
	for (let i = 0; i <= width; i++) {
		ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
		ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
	}

	// Horizontal lines.
	for (let j = 0; j <= height; j++) {
		ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
		ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
	}

	ctx.stroke();
};

const getIndex = (row, column) => { return row * width + column; };

const bitIsSet = (n, arr) => {
  const byte = Math.floor(n / 8);
  const mask = 1 << (n % 8);
  return (arr[byte] & mask) === mask;
};

const drawCells = () => {
	const cellsPtr = universe.get_cells();
	const cells = new Uint8Array(memory.buffer, cellsPtr, width * height / 8);

	ctx.beginPath();

	for (let row = 0; row < height; row++) {
		for (let col = 0; col < width; col++) {
			const idx = getIndex(row, col);

			ctx.fillStyle = bitIsSet(idx, cells)
			? ALIVE_COLOR
			: DEAD_COLOR;

			ctx.fillRect(
				col * (CELL_SIZE + 1) + 1,
    			row * (CELL_SIZE + 1) + 1,
				CELL_SIZE,
				CELL_SIZE
			);
    }
  }

  ctx.stroke();
};

let animationId = null;

const isPaused = () => {
	return animationId === null;
};

//Buttons and Widgets

const playPauseButton	= document.getElementById("play-pause");
const clearButton		= document.getElementById("clear-map");
const restartButton		= document.getElementById("random-restart");
const frameWidget		= document.getElementById("frame-widget");

const play = () => {
  playPauseButton.textContent = "⏸";
  renderLoop();
};

const pause = () => {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
};


let ticksPerFrame = parseInt(frameWidget.value);

const renderLoop = () => {
	drawGrid();
	drawCells();

	for (let i = 0; i < ticksPerFrame; i++) {
		universe.tick();
	}
	
	animationId = requestAnimationFrame(renderLoop);
};

//EVENTS

frameWidget.addEventListener("input", () => {
	ticksPerFrame = parseInt(frameWidget.value);
});

playPauseButton.addEventListener("click", event => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

clearButton.addEventListener("click", event => {
	if (isPaused()) {
		universe.clear_cells();
		drawGrid();
		drawCells();
	}
});

restartButton.addEventListener("click", event => {
	pause();
	universe.random_restart();
	drawGrid();
	drawCells();
});

canvas.addEventListener("click", event => {
	const boundingRect = canvas.getBoundingClientRect();

	const scaleX = canvas.width / boundingRect.width;
	const scaleY = canvas.height / boundingRect.height;

	const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
	const canvasTop = (event.clientY - boundingRect.top) * scaleY;

	const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
	const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

	if (event.ctrlKey) {
		universe.glider(row, col);
	} else if (event.shiftKey) {
		universe.pulsar(row, col);
	} else {
		universe.toggle_cell(row, col);
	}

	drawGrid();
	drawCells();
});

//main script

drawGrid();
drawCells();

pause();
