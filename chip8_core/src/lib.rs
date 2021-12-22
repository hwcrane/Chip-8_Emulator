pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDRESS: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
0x20, 0x60, 0x20, 0x20, 0x70, // 1
0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
0x90, 0x90, 0xF0, 0x10, 0x10, // 4
0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
0xF0, 0x10, 0x20, 0x40, 0x40, // 7
0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
0xF0, 0x90, 0xF0, 0x90, 0x90, // A
0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
0xF0, 0x80, 0x80, 0x80, 0xF0, // C
0xE0, 0x90, 0x90, 0x90, 0xE0, // D
0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct CPU {
    program_counter: u16,                           // Program counter
    ram: [u8; RAM_SIZE],                            // Ram
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],   // Display
    v_registers: [u8; NUM_REGS],                    // V registers
    i_register: u16,                                // I register
    stack_pointer: u16,                             // Pointer to the top of the stack
    stack: [u16; STACK_SIZE],                       // Stack
    keypad: [bool; NUM_KEYS],                       // Keys pressed
    delay_timer: u8,                                // Delay timer
    sound_timer: u8,                                // Sound timer
}

impl CPU {
    pub fn new() -> CPU {
        let mut new_cpu = CPU { 
            program_counter: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; NUM_REGS],
            i_register: 0, 
            stack_pointer: 0, 
            stack: [0; STACK_SIZE], 
            keypad: [false; NUM_KEYS], 
            delay_timer: 0, 
            sound_timer: 0 
        };

        // Loads the fontset into ram
        new_cpu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_cpu
    }

    // Resets cpu back to original state
    pub fn reset(&mut self) {
        self.program_counter = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; NUM_REGS];
        self.i_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keypad = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;
    }

    // Pushes a value to the stack
    fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }

    // pops a value off the stack
    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // BEEP
            }
            self.sound_timer -= 1; 
        }
    }

    pub fn tick(&mut self) {
        // Fetch
        let opcode = self.fetch();
        // Decode 
        let digits = self.decode(opcode);
        // Execute
        self.execute(digits, opcode);
    }

    fn fetch(&mut self) -> u16 {
        // Fetches the first byte of the opcode
        let first_half = self.ram[self.program_counter as usize] as u16;

        // Fetches the second byte of the opcode
        let second_half = self.ram[(self.program_counter + 1) as usize] as u16;

        // Increments Program counter by 2
        self.program_counter += 2;

        // combines the two bytes into the full opcode
        (first_half << 8) | second_half
    }

    // Decodes opcode by seperating it out into induvidual digits
    fn decode(&mut self, opcode: u16) -> (u16, u16, u16, u16){
        (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F)
        )
    }

    fn execute(&mut self, digits: (u16, u16, u16, u16), opcode: u16) {
        match digits {
            // Do nothing
            (0, 0, 0, 0) => return,

            // Clear screen
            (0, 0, 0xE, 0) => self.cls(),

            // Return from subroutine
            (0, 0, 0xE, 0xE) => self.ret(),

            // Jump to new address
            (1, _, _, _) => self.jmp(opcode),

            // Call subroutine
            (2, _, _, _) => self.call(opcode),

            // Skip a line if VX == NN
            (3, _, _, _) => self.skip_eq_nn(digits.1, opcode),

            // Skip a line if VX != NN
            (4, _, _, _) => self.skip_neq_nn(digits.1, opcode),

            // Skip a line if VX == VY
            (5, _, _, _) => self.skip_eq_vy(digits.1, digits.2),

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", opcode),
        }
    }

    // Clears screen
    fn cls(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT]
    }

    // Returns from a subroutine
    fn ret(&mut self) {
        let return_address = self.pop();
        self.program_counter = return_address;
    }

    // Jumps to new address
    fn jmp(&mut self, opcode: u16) {
        let new_address = opcode & 0xFFF;
        self.program_counter = new_address;
    }

    // Calls a subroutine
    fn call(&mut self, opcode: u16) {
        let subroutine_address = opcode & 0xFFF;
        self.push(self.program_counter);
        self.program_counter = subroutine_address;
    }

    // Skips a line if VX == NN
    fn skip_eq_nn(&mut self, x: u16, opcode: u16) {
        let nn = (opcode & 0xFF) as u8;
        if self.v_registers[x as usize] == nn {
            self.program_counter += 2
        }
    }

    // Skips a line if VX != NN
    fn skip_neq_nn(&mut self, x: u16, opcode: u16) {
        let nn = (opcode & 0xFF) as u8;
        if self.v_registers[x as usize] != nn {
            self.program_counter += 2
        }
    }

    // Skips a line if VX == VY
    fn skip_eq_vy(&mut self, x: u16, y: u16) {
        if self.v_registers[x as usize] == self.v_registers[y as usize]{
            self.program_counter += 2
        }
    }
    
}