use rand::random;

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
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
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

    // Get display
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keypad[index] = pressed;
    }

    // Loads game rom into ram
    pub fn load_rom(&mut self, data: &[u8]) {
        let start = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
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
            (5, _, _, 0) => self.skip_eq_vy(digits.1, digits.2),

            // Set VX to NN
            (6, _, _, _) => self.set_nn(digits.1, opcode),

            // Increment VX by NN
            (7, _, _, _) => self.incr_nn(digits.1, opcode),

            // Set VX to VY
            (8, _, _, 0) => self.set_vy(digits.1, digits.2),

            // Apply bitwise OR to VX using VY
            (8, _, _, 1) => self.or(digits.1, digits.2),

            // Apply bitwise AND to VX using VY
            (8, _, _, 2) => self.and(digits.1, digits.2),

            // Apply bitwise XOR to VX using VY
            (8, _, _, 3) => self.xor(digits.1, digits.2),

            // Increment VX by VY
            (8, _, _, 4) => self.incr_vy(digits.1, digits.2),

            // Decrease VX by VY
            (8, _, _, 5) => self.decr_vy(digits.1, digits.2),

            // Binary right shift VX
            (8, _, _, 6) => self.brs(digits.1),

            // Set VX to VY - VX
            (8, _, _, 7) => self.sub_vx(digits.1, digits.2),

            // Binary left shift VX
            (8, _, _, 0xE) => self.bls(digits.1),

            // Skip a line if VX != VY
            (9, _, _, 0) => self.skip_neq_vy(digits.1, digits.2),

            // Set value of I register to value in opcode
            (0xA, _, _, _) => self.seti(opcode),

            // Set program counter to V0 + value in opcode
            (0xB, _, _, _) => self.setpc(opcode),

            // Set VX to random number & value in opcode
            (0xC, _, _, _) => self.rand(digits.1, opcode),

            // Draw sprite
            (0xD, _, _, _) => self.draw(digits.1, digits.2, digits.3),

            // Skip if key pressed
            (0xE, _, 9, 0xE) => self.skip_kp(digits.1),

            // Skip if key not pressed
            (0xE, _, 0xA, 1) => self.skip_knp(digits.1),

            // Set VX to delay timer
            (0xF, _, 0, 7) => self.vx_to_dt(digits.1),

            // Wait for key press
            (0xF, _, 0, 0xA) => self.wait(digits.1),

            // Set the delay timer to VX
            (0xF, _, 1, 5) => self.dt_to_vx(digits.1),

            // Set the sound timer to VX
            (0xF, _, 1, 8) => self.st_to_vx(digits.1),

            // Increment the I register by VX
            (0xF, _, 1, 0xE) => self.incr_i(digits.1),

            // Set I register to font address
            (0xF, _, 2, 9) => self.seti_font(digits.1),

            // Store the Binary coded decimal of VX in ram
            (0xF, _, 3, 3) => self.bcd(digits.1),

            // Store V0 -> VX in ram
            (0xF, _, 5, 5) => self.store_v(digits.1),
            
            // Load V0 -> VX from ram
            (0xF, _, 6, 5) => self.load_v(digits.1),

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:?}", digits),
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

    // Sets VX to NN
    fn set_nn(&mut self, x: u16, opcode: u16) {
        let new_value = (opcode & 0xFF) as u8;
        self.v_registers[x as usize] = new_value;
    }

    // Increments VX by NN
    fn incr_nn(&mut self, x: u16, opcode: u16) {
        let increment_by = (opcode & 0xFF) as u8;
        self.v_registers[x as usize] =  self.v_registers[x as usize].wrapping_add(increment_by);
    }

    // Sets VX to VY
    fn set_vy(&mut self, x: u16, y: u16) {
        self.v_registers[x as usize] = self.v_registers[y as usize]
    }

    // Applies bitwise OR to VX using VY
    fn or(&mut self, x: u16, y: u16) {
        self.v_registers[x as usize] |= self.v_registers[y as usize];
    }

    // Applies bitwise AND to VX using VY
    fn and(&mut self, x: u16, y: u16) {
        self.v_registers[x as usize] &= self.v_registers[y as usize];
    }

    // Applies bitwise XOR to VX using VY
    fn xor(&mut self, x: u16, y: u16) {
        self.v_registers[x as usize] ^= self.v_registers[y as usize];
    }

    // Increments VX by VY
    fn incr_vy(&mut self, x: u16, y: u16) {
        let (new_vx, carry) = self.v_registers[x as usize].overflowing_add(self.v_registers[y as usize]);
        let new_vf = if carry { 1 } else { 0 };

        self.v_registers[x as usize] = new_vx;
        self.v_registers[0xF] = new_vf;
    }
    
    // Decreases VX by VY
    fn decr_vy(&mut self, x: u16, y: u16) {
        let (new_vx, borrow) = self.v_registers[x as usize].overflowing_sub(self.v_registers[y as usize]);
        let new_vf = if borrow { 0 } else { 1 };

        self.v_registers[x as usize] = new_vx;
        self.v_registers[0xF] = new_vf;
    }

    // Binary right shifts VX
    fn brs(&mut self, x: u16) {
        let lsb = self.v_registers[x as usize] & 1;
        self.v_registers[x as usize] >>= 1;
        self.v_registers[0xF] = lsb;
    }

    // Sets VX to be VY - VX
    fn sub_vx(&mut self, x: u16, y: u16) {
        let (new_vx, borrow) = self.v_registers[y as usize].overflowing_sub(self.v_registers[x as usize]);
        let new_vf = if borrow { 0 } else { 1 };

        self.v_registers[x as usize] = new_vx;
        self.v_registers[0xF] = new_vf;
    }

    // Binary left shifts VX
    fn bls(&mut self, x: u16) {
        let msb = (self.v_registers[x as usize] >> 7) & 1;
        self.v_registers[x as usize] <<= 1;
        self.v_registers[0xF] = msb;
    }

    // Skips a line if VX != VY
    fn skip_neq_vy(&mut self, x: u16, y: u16) {
        if self.v_registers[x as usize] != self.v_registers[y as usize]{
            self.program_counter += 2
        }
    }

    // Sets the I register to be the value encoded in the opcode
    fn seti(&mut self, opcode: u16) {
        let next_i = opcode & 0xFFF;
        self.i_register = next_i;
    }

    // Sets the program counter to V0 + value in opcode
    fn setpc(&mut self, opcode: u16) {
        let opcode_value = opcode & 0xFFF;
        self.program_counter = (self.v_registers[0] as u16) + opcode_value;
    }

    // Sets VX to be random number & value in opcode
    fn rand(&mut self, x: u16, opcode: u16) {
        let opcode_value = (opcode & 0xFF) as u8;
        let rng: u8 = random();
        self.v_registers[x as usize] = rng & opcode_value;
    }

    // Draws a sprite
    fn draw(&mut self, x: u16, y: u16, z: u16) {
        // Get x, y coords of the sprite
        let x_coord = self.v_registers[x as usize] as u16;
        let y_coord = self.v_registers[y as usize] as u16;

        // Get height of sprite
        let num_rows = z;

        // Keeps track of if any pixels are flipped
        let mut flipped = false;

        // Iterates over all the rows in the sprite

        for y_line in 0..num_rows {
            // Get rows memory address
            let addr =  self.i_register + y_line as u16;
            let pixels = self.ram[addr as usize];

            // Iterates through each column in the row
            for x_line in 0..8 {
                // Use a mask to get the current pixels bit, only flip it if it is a 1
                if (pixels & (0b10000000 >> x_line)) != 0 {
                    let sx = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let sy = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                    let index = sx + SCREEN_WIDTH * sy;

                    flipped |= self.screen[index];
                    self.screen[index] ^= true;
                }
            }
        }

        if flipped {
            self.v_registers[0xF] = 1;
        } else {
            self.v_registers[0xF] = 0;
        }

    }

    // Skips if a key is pressed
    fn skip_kp(&mut self, x: u16) {
        let key = self.v_registers[x as usize];
        if self.keypad[key as usize] {
            self.program_counter += 2
        }
    }

    // Skips if a key is not pressed
    fn skip_knp(&mut self, x: u16) {
        let key = self.v_registers[x as usize];
        if !self.keypad[key as usize] {
            self.program_counter += 2
        }
    }

    // Sets the value of VX to the delay counter
    fn vx_to_dt(&mut self, x: u16) {
        self.v_registers[x as usize] = self.delay_timer;
    }

    // Waits for a key to be pressed
    fn wait(&mut self, x: u16) {
        let mut pressed = false;
        for i in 0..self.keypad.len() {
            if self.keypad[i] {
                self.v_registers[x as usize] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.program_counter -= 2;
        }
    }

    // Sets the delay timer to VX
    fn dt_to_vx(&mut self, x: u16) {
        self.delay_timer = self.v_registers[x as usize];
    }

    // Sets the sound timer to VX
    fn st_to_vx(&mut self, x: u16) {
        self.sound_timer = self.v_registers[x as usize];
    }

    // Increments the I register by VX
    fn incr_i(&mut self, x: u16) {
        let increment_by = self.v_registers[x as usize] as u16;
        self.i_register = self.i_register.wrapping_add(increment_by);
    }

    // Sets the I register to a font address
    fn seti_font(&mut self, x: u16) {
        let number = self.v_registers[x as usize] as u16;
        self.i_register = number * 5;
    }

    // Stores the Binary Coded decimal of VX in ram
    fn bcd(&mut self, x: u16) {
        let vx = self.v_registers[x as usize] as f32;

        let hundreds = (vx / 100.).floor() as u8;
        let tens = ((vx / 10.) % 10.).floor() as u8;
        let units = (vx % 10.).floor() as u8;

        self.ram[self.i_register as usize] = hundreds;
        self.ram[(self.i_register + 1) as usize] = tens;
        self.ram[(self.i_register + 2) as usize] = units;
    }

    // Stores V0 -> VX in ram
    fn store_v(&mut self, x: u16) {
        let i = self.i_register as usize;
        for index in 0..=x as usize {
            self.ram[i + index] = self.v_registers[index];
        }
    } 

    // Loads V0 -> VX from ram
    fn load_v(&mut self, x: u16) {
        let i = self.i_register as usize;
        for index in 0..=x as usize {
            self.v_registers[index] = self.ram[i + index];
        }
    }
}