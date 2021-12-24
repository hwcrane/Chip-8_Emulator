import init, * as wasm from "./wasm.js";

const WIDTH = 64;
const HEIGHT = 32;
let SCALE = Math.floor(window.innerWidth / 80) - Math.floor(window.innerWidth / 800);
const TICKS_PER_FRAME = 8;
let anim_frame = 0;

const canvas = document.getElementById("canvas");
canvas.width = WIDTH * SCALE;
canvas.height = HEIGHT * SCALE;
const ctx = canvas.getContext("2d");
ctx.fillStyle = "black";
ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);
const roms = document.getElementById("roms");
const start = document.getElementById("start");

async function run() {
    await init();
    let chip8 = new wasm.CPUWasm();

    document.addEventListener("keydown", function(event) {
        chip8.keypress(event, true);
    })

    document.addEventListener("keyup", function(event) {
        chip8.keypress(event, false);
    })

    start.addEventListener("click", function(event) {
        if (anim_frame != 0) {
            window.cancelAnimationFrame(anim_frame);
        }

        let file = roms.value;
        if (file == "NONE") {
            alert("Please select a ROM to load");
            return;
        }

        fetch("./roms/" + file)
            .then(i => i.arrayBuffer())
            .then(buffer => {
                const rom = new Uint8Array(buffer);
                chip8.reset();
                chip8.load_rom(rom);
                mainloop(chip8);
            });
    }, false);
}

function mainloop(chip8) {
    for (let i = 0; i < TICKS_PER_FRAME; i++) {
        chip8.tick();
    }
    chip8.tick_timers();

    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);

    ctx . fillStyle = "lime";
    chip8.draw_screen(SCALE);

    anim_frame = window.requestAnimationFrame(() => {
        mainloop(chip8);
    })
}

run().catch(console.error);

window.addEventListener("resize", function(event) {
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);

    SCALE = Math.floor(window.innerWidth / 80) - Math.floor(window.innerWidth / 800);
    canvas.width = WIDTH * SCALE;
    canvas.height = HEIGHT * SCALE;
})
