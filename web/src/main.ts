import init, {
  WebBoard,
} from "../pkg/game_of_life.js";
import "./style.css";

const WIDTH = 100;
const HEIGHT = 100;

type Source = "workspace" | "pattern" | "random";

const app = document.querySelector<HTMLElement>("#app");
if (!app) throw new Error("Missing app root");

await init();

const board = new WebBoard(WIDTH, HEIGHT);
let source: Source = "pattern";
let selectedPattern = 0;
let running = false;
let speed = 8;
let lastStep = performance.now();
let drawing = false;

const patternOptions = Array.from({ length: WebBoard.pattern_count() }, (_, index) => ({
  index,
  name: WebBoard.pattern_name(index),
  description: WebBoard.pattern_description(index),
}));

app.innerHTML = `
  <div class="shell">
    <header class="hero">
      <div>
        <p class="eyebrow">Rust + WebAssembly</p>
        <h1>Game of Life</h1>
        <p class="lede">Explore Conway’s cellular automaton directly in your browser.</p>
      </div>
      <a class="source-link" href="https://github.com/RoKu1/GameOfLife_Rust_TUI" target="_blank" rel="noreferrer">View source ↗</a>
    </header>
    <section class="layout">
      <div class="board-card">
        <div class="board-header">
          <div>
            <span class="label">SIMULATION</span>
            <h2 id="status">Paused</h2>
          </div>
          <div class="stats">
            <span><strong id="generation">0</strong><small>generation</small></span>
            <span><strong id="live-count">0</strong><small>live cells</small></span>
          </div>
        </div>
        <canvas id="board" width="1000" height="1000" aria-label="Game of Life board"></canvas>
        <p class="board-note">Finite 100 × 100 board · edges are dead · click or drag to edit</p>
      </div>
      <aside class="controls">
        <div class="panel">
            <span class="label">1 · CHOOSE A STARTING POINT</span>
            <div class="source-buttons" role="group" aria-label="Starting source">
            <button data-source="pattern" class="source active">Pattern library</button>
            <button data-source="workspace" class="source">Edit workspace</button>
            <button data-source="random" class="source">Random board</button>
            </div>
          <div id="pattern-controls" class="pattern-controls">
            <label for="pattern">Pattern</label>
            <select id="pattern"></select>
            <p id="pattern-description" class="muted"></p>
          </div>
          <div class="button-row">
            <button id="start" class="primary">Start with this state</button>
            <button id="clear">Clear board</button>
          </div>
        </div>
        <div class="panel">
          <span class="label">2 · RUN THE SIMULATION</span>
          <div class="button-row">
            <button id="pause">Resume</button>
            <button id="step">Step</button>
            <button id="reset">Reset</button>
          </div>
          <label class="range-label" for="speed">Speed <output id="speed-value">8 gen/s</output></label>
          <input id="speed" type="range" min="1" max="20" step="1" value="8" />
        </div>
        <div class="panel help">
          <span class="label">3 · EDIT ANYTIME</span>
          <p>Pick a pattern or edit the workspace. Click or drag on the board to change cells. Start runs continuously until paused.</p>
        </div>
      </aside>
    </section>
    <footer>Simulation rules run in Rust and are compiled to WebAssembly.</footer>
  </div>
`;

const canvas = document.querySelector<HTMLCanvasElement>("#board")!;
const context = canvas.getContext("2d")!;
const status = document.querySelector<HTMLElement>("#status")!;
const generation = document.querySelector<HTMLElement>("#generation")!;
const liveCount = document.querySelector<HTMLElement>("#live-count")!;
const pauseButton = document.querySelector<HTMLButtonElement>("#pause")!;
const patternSelect = document.querySelector<HTMLSelectElement>("#pattern")!;
const patternControls = document.querySelector<HTMLElement>("#pattern-controls")!;
const patternDescription = document.querySelector<HTMLElement>("#pattern-description")!;
const speedValue = document.querySelector<HTMLOutputElement>("#speed-value")!;

function setSource(nextSource: Source) {
  source = nextSource;
  document.querySelectorAll<HTMLButtonElement>("[data-source]").forEach((item) => {
    item.classList.toggle("active", item.dataset.source === nextSource);
  });
  patternControls.hidden = nextSource !== "pattern";
  if (nextSource === "pattern") {
    board.load_pattern(selectedPattern);
    running = false;
  } else if (nextSource === "random") {
    board.randomize(crypto.getRandomValues(new Uint32Array(1))[0]);
    running = false;
  }
  render();
}

for (const pattern of patternOptions) {
  patternSelect.add(new Option(pattern.name, String(pattern.index)));
}

function updatePatternDetails() {
  const pattern = patternOptions[selectedPattern];
  patternDescription.textContent = pattern.description;
  patternSelect.value = String(selectedPattern);
}

function render() {
  const cells = board.cells();
  const cellSize = canvas.width / WIDTH;
  context.fillStyle = "#0b1424";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = "#72f2a3";
  for (let y = 0; y < HEIGHT; y += 1) {
    for (let x = 0; x < WIDTH; x += 1) {
      if (cells[y * WIDTH + x]) {
        context.fillRect(x * cellSize + 0.5, y * cellSize + 0.5, cellSize - 1, cellSize - 1);
      }
    }
  }
  context.strokeStyle = "rgba(163, 185, 210, 0.08)";
  context.lineWidth = 1;
  for (let i = 0; i <= WIDTH; i += 1) {
    const position = i * cellSize + 0.5;
    context.beginPath();
    context.moveTo(position, 0);
    context.lineTo(position, canvas.height);
    context.stroke();
    context.beginPath();
    context.moveTo(0, position);
    context.lineTo(canvas.width, position);
    context.stroke();
  }
  generation.textContent = String(board.generation());
  liveCount.textContent = String(board.live_count());
  status.textContent = running ? "Running" : "Paused";
  status.className = running ? "running" : "";
  pauseButton.textContent = running ? "Pause" : "Resume";
}

function pointerCell(event: PointerEvent): [number, number] | null {
  const bounds = canvas.getBoundingClientRect();
  const x = Math.floor(((event.clientX - bounds.left) / bounds.width) * WIDTH);
  const y = Math.floor(((event.clientY - bounds.top) / bounds.height) * HEIGHT);
  return x >= 0 && x < WIDTH && y >= 0 && y < HEIGHT ? [x, y] : null;
}

canvas.addEventListener("pointerdown", (event) => {
  drawing = true;
  canvas.setPointerCapture(event.pointerId);
  const cell = pointerCell(event);
  if (cell) board.toggle_cell(cell[0], cell[1]);
  setSource("workspace");
  running = false;
  render();
});
canvas.addEventListener("pointermove", (event) => {
  if (!drawing) return;
  const cell = pointerCell(event);
  if (cell) board.set_cell(cell[0], cell[1], true);
  render();
});
canvas.addEventListener("pointerup", () => { drawing = false; });
canvas.addEventListener("pointercancel", () => { drawing = false; });

document.querySelectorAll<HTMLButtonElement>("[data-source]").forEach((button) => {
  button.addEventListener("click", () => {
    setSource(button.dataset.source as Source);
  });
});

patternSelect.addEventListener("change", () => {
  selectedPattern = Number(patternSelect.value);
  updatePatternDetails();
  if (source === "pattern") {
    board.load_pattern(selectedPattern);
    running = false;
    render();
  }
});

document.querySelector<HTMLButtonElement>("#start")!.addEventListener("click", () => {
  if (source === "pattern") board.load_pattern(selectedPattern);
  if (source === "random") board.randomize(crypto.getRandomValues(new Uint32Array(1))[0]);
  if (source === "workspace") board.save_seed();
  running = true;
  render();
});
document.querySelector<HTMLButtonElement>("#clear")!.addEventListener("click", () => { board.clear(); running = false; render(); });
pauseButton.addEventListener("click", () => { running = !running; render(); });
document.querySelector<HTMLButtonElement>("#step")!.addEventListener("click", () => { running = false; board.step(); render(); });
document.querySelector<HTMLButtonElement>("#reset")!.addEventListener("click", () => { board.reset_to_seed(); running = false; render(); });
document.querySelector<HTMLInputElement>("#speed")!.addEventListener("input", (event) => {
  speed = Number((event.target as HTMLInputElement).value);
  speedValue.value = `${speed} gen/s`;
});

function loop(now: number) {
  if (running && now - lastStep >= 1000 / speed) {
    board.step();
    lastStep = now;
    render();
  }
  requestAnimationFrame(loop);
}

updatePatternDetails();
board.load_pattern(selectedPattern);
render();
requestAnimationFrame(loop);
