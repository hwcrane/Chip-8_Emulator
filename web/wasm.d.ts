/* tslint:disable */
/* eslint-disable */
/**
*/
export class CPUWasm {
  free(): void;
/**
*/
  constructor();
/**
*/
  tick(): void;
/**
*/
  tick_timers(): void;
/**
*/
  reset(): void;
/**
* @param {KeyboardEvent} event
* @param {boolean} pressed
*/
  keypress(event: KeyboardEvent, pressed: boolean): void;
/**
* @param {Uint8Array} data
*/
  load_rom(data: Uint8Array): void;
/**
* @param {number} scale
*/
  draw_screen(scale: number): void;
/**
* @param {number} key
* @param {boolean} pressed
*/
  button_press(key: number, pressed: boolean): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_cpuwasm_free: (a: number) => void;
  readonly cpuwasm_new: () => number;
  readonly cpuwasm_tick: (a: number) => void;
  readonly cpuwasm_tick_timers: (a: number) => void;
  readonly cpuwasm_reset: (a: number) => void;
  readonly cpuwasm_keypress: (a: number, b: number, c: number) => void;
  readonly cpuwasm_load_rom: (a: number, b: number) => void;
  readonly cpuwasm_draw_screen: (a: number, b: number) => void;
  readonly cpuwasm_button_press: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
