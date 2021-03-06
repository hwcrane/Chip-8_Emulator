use chip8_core::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{KeyboardEvent, CanvasRenderingContext2d, HtmlCanvasElement};
use js_sys::Uint8Array;

#[wasm_bindgen]
pub struct CPUWasm {
    chip8: CPU,
    ctx: CanvasRenderingContext2d
}

#[wasm_bindgen]
impl CPUWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<CPUWasm, JsValue>{
        let chip8 = CPU::new();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        
        let ctx = canvas.get_context("2d")
            .unwrap().unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        
        Ok(CPUWasm{chip8, ctx})
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.chip8.tick();
    }

    #[wasm_bindgen]
    pub fn tick_timers(&mut self) {
        self.chip8.tick_timers();
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.chip8.reset();
    }

    #[wasm_bindgen]
    pub fn keypress(&mut self, event: KeyboardEvent, pressed: bool) {
        let key  = event.key();
        if let Some(k) = convert_keycode(&key) {
            self.chip8.keypress(k, pressed);
        };
    }

    #[wasm_bindgen]
    pub fn load_rom(&mut self, data: Uint8Array) {
        self.chip8.load_rom(&data.to_vec());
    }

    #[wasm_bindgen]
    pub fn draw_screen(&mut self, scale: usize) {
        let disp = self.chip8.get_display();
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            if disp[i] {
                let x = i % SCREEN_WIDTH;
                let y = i / SCREEN_WIDTH;
                self.ctx.fill_rect(
                    (x * scale) as f64,
                    (y * scale) as f64,
                    scale as f64, 
                    scale as f64
                )
            }
        }
    }

    #[wasm_bindgen]
    pub fn button_press(&mut self, key: usize, pressed: bool) {
        self.chip8.keypress(key, pressed);
    }
}

fn convert_keycode(key: &str) -> Option<usize> {
    match key {
        "1" => Some(0x1),
        "2" => Some(0x2),
        "3" => Some(0x3),
        "4" => Some(0xC),
        "q" => Some(0x4),
        "w" => Some(0x5),
        "e" => Some(0x6),
        "r" => Some(0xD),
        "a" => Some(0x7),
        "s" => Some(0x8),
        "d" => Some(0x9),
        "f" => Some(0xE),
        "z" => Some(0xA),
        "x" => Some(0x0),
        "c" => Some(0xB),
        "v" => Some(0xF),
        _ => None
    }
}