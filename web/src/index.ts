import init, { Position, World, glider_pattern } from "game-of-life-wasm";

const bootstrap = async () => {
  await init();

  const canvas = document.getElementById("game-of-life-canvas") as HTMLCanvasElement | null;
  if (!canvas) {
    throw new Error("Canvas element #game-of-life-canvas not found.");
  }

  const playPauseButton = document.getElementById("play-pause") as HTMLButtonElement | null;
  const stepButton = document.getElementById("step") as HTMLButtonElement | null;
  const randomizeButton = document.getElementById("randomize") as HTMLButtonElement | null;
  const clearButton = document.getElementById("clear") as HTMLButtonElement | null;
  const speedInput = document.getElementById("speed") as HTMLInputElement | null;
  const toggleGridInput = document.getElementById("toggle-grid") as HTMLInputElement | null;
  const gridSizeEl = document.getElementById("grid-size") as HTMLSpanElement | null;
  const generationEl = document.getElementById("generation") as HTMLSpanElement | null;
  const fpsEl = document.getElementById("fps") as HTMLSpanElement | null;

  if (
    !playPauseButton ||
    !stepButton ||
    !randomizeButton ||
    !clearButton ||
    !speedInput ||
    !toggleGridInput ||
    !gridSizeEl ||
    !generationEl ||
    !fpsEl
  ) {
    throw new Error("One or more UI elements are missing.");
  }

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Canvas rendering context not available.");
  }

  const width = 64;
  const height = 64;
  const cellSize = canvas.width / width;
  let world = glider_pattern(width, height);

  let generation = 0;
  let isPlaying = false;
  let showGrid = toggleGridInput.checked;
  let speed = Number.parseInt(speedInput.value, 10) || 12;

  let isDrawing = false;
  let lastPainted = -1;

  let lastFrameTime = 0;
  let lastFpsUpdate = 0;
  let frames = 0;

  const toPosition = (row: number, col: number) =>
    new Position(BigInt(col + 1), BigInt(row + 1));
  const updateStats = () => {
    gridSizeEl.textContent = `${width} x ${height}`;
    generationEl.textContent = generation.toString();
  };

  const drawGrid = () => {
    ctx.strokeStyle = "rgba(255, 255, 255, 0.06)";
    ctx.lineWidth = 1;

    for (let x = 0; x <= width; x += 1) {
      ctx.beginPath();
      ctx.moveTo(x * cellSize + 0.5, 0);
      ctx.lineTo(x * cellSize + 0.5, height * cellSize);
      ctx.stroke();
    }

    for (let y = 0; y <= height; y += 1) {
      ctx.beginPath();
      ctx.moveTo(0, y * cellSize + 0.5);
      ctx.lineTo(width * cellSize, y * cellSize + 0.5);
      ctx.stroke();
    }
  };

  const drawCells = () => {
    ctx.fillStyle = "#0c0f16";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = "#ffb000";
    const alive = world.alive_positions();
    for (const pos of alive) {
      const col = Number(pos.x) - 1;
      const row = Number(pos.y) - 1;
      if (col < 0 || row < 0 || col >= width || row >= height) {
        continue;
      }

      ctx.fillRect(
        col * cellSize + 1,
        row * cellSize + 1,
        cellSize - 2,
        cellSize - 2
      );
    }

    if (showGrid) {
      drawGrid();
    }
  };

  const tick = () => {
    world.tick();
    generation += 1;
    updateStats();
  };

  const randomize = () => {
    const randomAlive: Position[] = [];
    for (let row = 0; row < height; row += 1) {
      for (let col = 0; col < width; col += 1) {
        if (Math.random() < 0.3) {
          randomAlive.push(toPosition(row, col));
        }
      }
    }

    world = new World(width, height, randomAlive);
    generation = 0;
    updateStats();
  };

  const clear = () => {
    world = new World(width, height, []);
    generation = 0;
    updateStats();
  };

  const getCellFromEvent = (event: MouseEvent) => {
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;

    const x = (event.clientX - rect.left) * scaleX;
    const y = (event.clientY - rect.top) * scaleY;

    const col = Math.max(0, Math.min(width - 1, Math.floor(x / cellSize)));
    const row = Math.max(0, Math.min(height - 1, Math.floor(y / cellSize)));

    return { row, col };
  };

  const paintCell = (event: MouseEvent) => {
    const { row, col } = getCellFromEvent(event);
    const cellIndex = row * width + col;

    if (cellIndex === lastPainted) {
      return;
    }

    lastPainted = cellIndex;
    drawCells();
  };

  const playLoop = (timestamp: number) => {
    if (!isPlaying) {
      return;
    }

    const interval = 1000 / speed;
    if (timestamp - lastFrameTime >= interval) {
      tick();
      drawCells();
      lastFrameTime = timestamp;
      frames += 1;
    }

    if (timestamp - lastFpsUpdate >= 500) {
      const fps = Math.round((frames * 1000) / (timestamp - lastFpsUpdate));
      fpsEl.textContent = fps.toString();
      frames = 0;
      lastFpsUpdate = timestamp;
    }

    requestAnimationFrame(playLoop);
  };

  playPauseButton.addEventListener("click", () => {
    isPlaying = !isPlaying;
    playPauseButton.textContent = isPlaying ? "Pause" : "Play";

    if (isPlaying) {
      lastFrameTime = performance.now();
      lastFpsUpdate = lastFrameTime;
      frames = 0;
      requestAnimationFrame(playLoop);
    } else {
      fpsEl.textContent = "0";
    }
  });

  stepButton.addEventListener("click", () => {
    tick();
    drawCells();
  });

  randomizeButton.addEventListener("click", () => {
    randomize();
    drawCells();
  });

  clearButton.addEventListener("click", () => {
    clear();
    drawCells();
  });

  speedInput.addEventListener("input", () => {
    speed = Number.parseInt(speedInput.value, 10) || 12;
  });

  toggleGridInput.addEventListener("change", () => {
    showGrid = toggleGridInput.checked;
    drawCells();
  });

  canvas.addEventListener("mousedown", (event) => {
    isDrawing = true;
    paintCell(event);
  });

  canvas.addEventListener("mousemove", (event) => {
    if (!isDrawing) {
      return;
    }
    paintCell(event);
  });

  window.addEventListener("mouseup", () => {
    isDrawing = false;
    lastPainted = -1;
  });

  updateStats();
  drawCells();
};

bootstrap().catch((error) => {
  console.error("Failed to start Game of Life UI.", error);
});
