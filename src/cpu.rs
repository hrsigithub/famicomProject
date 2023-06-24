// a9 c0 aa e8 00

// LDA #$c0 ; a9 c0

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    Relative,
    Implied,
    NoneAddressing,
}

const FLAG_CARRY: u8 = 1 << 0;
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_INTERRUPT: u8 = 1 << 2;
const FLAG_DECIMAL: u8 = 1 << 3;
const FLAG_BREAK: u8 = 1 << 4;
const FLAG_BREAK2: u8 = 1 << 5; // 5 は未使用。
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_NEGATIVE: u8 = 1 << 7;

const SIGN_BIT: u8 = 1 << 7;

#[derive(Debug)]
pub struct CPU {
    // アキュムレータ
    pub register_a: u8, // 1byte

    // インデックス レジスタ
    pub register_x: u8,
    pub register_y: u8,

    // プロセッサ ステータス
    pub status: u8,

    // プログラム内の現在位置を追跡する
    // プログラム カウンター
    pub program_counter: u16, // 2Byte

    // メモリー
    pub memory: [u8; 0x10000],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0x00; 0x10000],
        }
    }

    // メモリアドレッシングモード
    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        // println!("mode :{:?} ", mode);
        // println!("program_counter: {:?} ", self.program_counter);

        match mode {
            AddressingMode::Implied => {
                panic!("mode {:?} is not supported", mode);
            }

            AddressingMode::Accumulator => {
                panic!("mode {:?} is not supported", mode);
            }

            // LDA #$44
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            // Absolute 3Byte
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            // Absolute 3Byte
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            // BCC *+4 => 90 04
            AddressingMode::Relative => {
                let base = self.mem_read(self.program_counter);
                let np = (base as i8) as i32 + self.program_counter as i32;
                return np as u16;
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    // リトルエンディアン アドレス指定
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    // リトルエンディアン アドレス指定
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;

        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi)
    }

    // すべてのレジスタの状態を復元し、0xFFFC に格納されている2バイトの値で
    // プログラム カウンタを初期化する必要があります
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    // 0x8000 から、カトリッジのデータをロード
    // プログラムを PRG ROM 空間にロードし、コードへの参照を 0xFFFC メモリ セルに保存する必要がある。
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn memory_print(&self) {
        for (index, byte) in self.memory.iter().enumerate() {
            print!("{:02X} ", byte);
            if (index + 1) % 25 == 0 {
                println!();
            }
        }
        println!();
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // A = 0の場合に設定
        self.status = if result == 0 {
            self.status | FLAG_ZERO
        } else {
            self.status & !FLAG_ZERO
        };

        // [Negative Flag 7ビット目] A のビット7(0b1000_0000)が設定されている場合に設定
        if result & 0b1000_0000 != 0 {
            // ビット7(0b1000_0000)が設定されている
            self.status = self.status | FLAG_NEGATIVE;
        } else {
            // ビット7をクリア
            self.status = self.status & !FLAG_NEGATIVE;
        }
    }

    fn _brach(&mut self, mode: &AddressingMode, flag: u8, non_zero: bool) {
        let addr = self.get_operand_address(mode);

        if non_zero {
            if self.status & flag != 0 {
                self.program_counter = addr;
            }
        } else {
            if self.status & flag == 0 {
                self.program_counter = addr;
            }
        }
    }

    ///////////////
    fn cpy(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_y, mode)
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_x, mode)
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_a, mode)
    }

    fn _cmp(&mut self, target: u8, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if target >= value {
            self.sec(&AddressingMode::Implied);
        } else {
            self.clc(&AddressingMode::Implied);
        }

        let (value, _) = target.overflowing_sub(value);

        self.update_zero_and_negative_flags(value);
    }

    fn clv(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_OVERFLOW
    }

    fn sei(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_INTERRUPT
    }

    fn cli(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_INTERRUPT
    }

    fn sed(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_DECIMAL
    }

    fn cld(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_DECIMAL
    }

    fn sec(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_CARRY
    }

    fn clc(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_CARRY
    }

    fn bvs(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_OVERFLOW, true);
    }

    fn bvc(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_OVERFLOW, false);
    }

    fn brk(&mut self, mode: &AddressingMode) {
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status = self.status | FLAG_BREAK;
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let zero = self.register_a & value; // A&M

        if zero == 0 {
            self.status = self.status | FLAG_ZERO;
        } else {
            self.status = self.status & !FLAG_ZERO;
        }

        let flags = FLAG_NEGATIVE | FLAG_OVERFLOW;

        self.status = (self.status & !flags) | (value & flags);
    }

    fn bcc(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_CARRY, false);
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_CARRY, true);
    }

    fn beq(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_ZERO, true);
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_NEGATIVE, true);
    }

    fn bpl(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_NEGATIVE, false);
    }

    fn bne(&mut self, mode: &AddressingMode) {
        self._brach(mode, FLAG_ZERO, false);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry = self.status & FLAG_CARRY;
        let (rhs, carry_flag1) = value.overflowing_add(carry);
        let (n, carry_flag2) = self.register_a.overflowing_add(rhs);

        let overflow = (self.register_a & SIGN_BIT) == (value & SIGN_BIT)
            && (value & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

        self.status = if carry_flag1 || carry_flag2 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };

        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a & value;

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a ^ value;

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a | value;

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value;
            (value, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };

        self.update_zero_and_negative_flags(value);
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value | (self.status & FLAG_CARRY);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            let value = value | (self.status & FLAG_CARRY);
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };

        self.update_zero_and_negative_flags(value);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry == 1 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };

        self.update_zero_and_negative_flags(value);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            self.register_a = self.register_a | ((self.status & FLAG_CARRY) << 7);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            let value = value | ((self.status & FLAG_CARRY) << 7);
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry == 1 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        // A-M-(1-C)
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry = self.status & FLAG_CARRY;
        let (v1, carry_flag1) = self.register_a.overflowing_sub(value);
        let (n, carry_flag2) = v1.overflowing_sub(1 - carry);

        let overflow = (self.register_a & SIGN_BIT) != (value & SIGN_BIT)
            && (self.register_a & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

        self.status = if !carry_flag1 && !carry_flag2 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };

        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(self.register_a);
    }

    // LDA メモリのバイトをアキュムレータにロードし、必要に応じてゼロと負のフラグを設定します。
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // アキュムレータの内容をメモリに保存します。
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        // オーバーフロー対応
        self.register_x = self.register_x.wrapping_add(1);

        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn run(&mut self) {
        loop {
            // let opscode = self.mem_read(self.program_counter);
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;

            println!("code:{:X}", code);

            match code {

                // CPY
                0xC0 => {
                    self.cpy(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // CPX
                0xE0 => {
                    self.cpx(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // CMP
                0xC9 => {
                    self.cmp(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // CLV
                0xB8 => {
                    self.clv(&AddressingMode::Implied);
                }

                // SEI
                0x78 => {
                    self.sei(&AddressingMode::Implied);
                }

                // CLI
                0x58 => {
                    self.cli(&AddressingMode::Implied);
                }

                // SED
                0xF8 => {
                    self.sed(&AddressingMode::Implied);
                }

                // CLD
                0xD8 => {
                    self.cld(&AddressingMode::Implied);
                }

                // SEC
                0x38 => {
                    self.sec(&AddressingMode::Implied);
                }

                // CLC
                0x18 => {
                    self.clc(&AddressingMode::Implied);
                }

                // BVS
                0x70 => {
                    self.bvs(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BVC
                0x50 => {
                    self.bvc(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BPL
                0x10 => {
                    self.bpl(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BMI
                0x30 => {
                    self.bmi(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BIT
                0x24 => {
                    self.bit(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0x2C => {
                    self.bit(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }

                // BEQ
                0xF0 => {
                    self.beq(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BNE
                0xD0 => {
                    self.bne(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BCC
                0x90 => {
                    self.bcc(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BCS
                0xB0 => {
                    self.bcs(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // ROR
                0x6A => {
                    self.ror(&AddressingMode::Accumulator);
                }

                0x66 => {
                    self.ror(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                // ROL
                0x2A => {
                    self.rol(&AddressingMode::Accumulator);
                }

                0x26 => {
                    self.rol(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                // LSR
                0x4A => {
                    self.lsr(&AddressingMode::Accumulator);
                }

                0x46 => {
                    self.lsr(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                // ASL
                0x0A => {
                    self.asl(&AddressingMode::Accumulator);
                }

                0x06 => {
                    self.asl(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                // ADC
                0x69 => {
                    self.adc(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // AND
                0x29 => {
                    self.and(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // EOR
                0x49 => {
                    self.eor(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // ORA
                0x09 => {
                    self.ora(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // SBC
                0xE9 => {
                    self.sbc(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                // LDA (0xA9)オペコード
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xBD => {
                    self.lda(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0xB9 => {
                    self.lda(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }

                0xA1 => {
                    self.lda(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0xB1 => {
                    self.lda(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }

                /* STA */
                0x85 => {
                    self.sta(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0x95 => {
                    self.sta(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0x8D => {
                    self.sta(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x9D => {
                    self.sta(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x99 => {
                    self.sta(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0x81 => {
                    self.sta(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0x91 => {
                    self.sta(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }

                // TAX (0xAA)オペコード
                0xAA => self.tax(),

                // INX (0xE8)オペコード
                0xE8 => self.inx(),

                // BRK
                0x00 => {
                    //self.brk(&AddressingMode::Implied);
                    break;
                }

                _ => todo!(),
            }
        }
    }

    //  ロード、リセット、実行
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn run<F>(program: Vec<u8>, f: F) -> CPU
    where
        F: Fn(&mut CPU),
    {
        let mut cpu = CPU::new();
        cpu.load(program);
        cpu.reset();
        f(&mut cpu);
        cpu.run();
        cpu
    }

    fn assert_status(cpu: &CPU, flags: u8) {
        assert_eq!(cpu.status, flags)
    }

    fn dump(cpu: &CPU) {
        // ターミナルのバッファを拡張する必要あり。
        for (index, byte) in cpu.memory.iter().enumerate() {
            print!("{:02X} ", byte);
            if (index + 1) % 20 == 0 {
                println!();
            }
        }
        println!();
    }

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b0000_0000);
        assert!(cpu.status & 0b1000_0000 == 0b0000_0000);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x00]);

        assert!(cpu.status & 0b1000_0010 != 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let cpu = run(vec![0xaa, 0x00], |cpu| {
            cpu.register_a = 10;
        });
        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let cpu = run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();

        cpu.load(vec![0xe8, 0xe8, 0x00]);
        cpu.reset();

        cpu.register_x = 0xff;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }

    #[test]
    fn test_lda_from_memory_zero_page() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_from_memory_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xb5, 0x10, 0x00]);
        cpu.reset();
        cpu.mem_write(0x11, 0x56);
        cpu.register_x = 1;
        cpu.run();

        assert_eq!(cpu.register_a, 0x56);
    }

    #[test]
    fn test_lda_from_memory_absolute() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xAD, 0x10, 0xAA, 0x00]);
        cpu.reset();
        cpu.mem_write(0xAA10, 0x57);
        cpu.run();

        assert_eq!(cpu.register_a, 0x57);
    }

    #[test]
    fn test_lda_from_memory_absolute_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xBD, 0x10, 0xAA, 0x00]);
        cpu.reset();
        cpu.mem_write(0xAA15, 0x58);
        cpu.register_x = 0x05;
        cpu.run();

        assert_eq!(cpu.register_a, 0x58);
    }

    #[test]
    fn test_lda_from_memory_absolute_y() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xB9, 0x10, 0xAA, 0x00]);
        cpu.reset();
        cpu.mem_write(0xAA18, 0x59);
        cpu.register_y = 0x08;
        cpu.run();

        assert_eq!(cpu.register_a, 0x59);
    }

    #[test]
    fn test_lda_from_memory_indirect_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xA1, 0x10, 0x00]);
        cpu.reset();
        cpu.mem_write(0x18, 0x05);
        cpu.mem_write(0x19, 0xFF);
        cpu.mem_write(0xFF05, 0x5A);
        cpu.register_x = 0x08;
        cpu.run();

        assert_eq!(cpu.register_a, 0x5A);
    }

    #[test]
    fn test_lda_from_memory_indirect_y() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xB1, 0x10, 0x00]);
        cpu.reset();
        cpu.mem_write(0x10, 0x06);
        cpu.mem_write(0x11, 0xFF);
        cpu.mem_write(0xFF09, 0x5B);
        cpu.register_y = 0x03;
        cpu.run();

        assert_eq!(cpu.register_a, 0x5B);
    }

    #[test]
    fn test_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x007, 0x06);
        dump(&cpu);
        assert_eq!(cpu.mem_read(0x007), 0x06);
    }

    #[test]
    fn test_sta_from_memory() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x85, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0xBA;
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0xBA);
    }

    #[test]
    fn test_adc_no_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0x20;
        cpu.run();

        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_adc_has_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0x20;
        cpu.status = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register_a, 0x31);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_adc_occur_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x01, 0x00]);
        cpu.reset();
        cpu.register_a = 0xFF;
        cpu.run();

        assert_eq!(cpu.register_a, 0x00);
        assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
    }

    #[test]
    fn test_adc_occur_overflow_plus() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0x7F;
        cpu.run();

        assert_eq!(cpu.register_a, 0x8F);
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_adc_occur_overflow_plus_with_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x6F, 0x00]);
        cpu.reset();
        cpu.register_a = 0x10;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.status, 0xC0);
    }

    #[test]
    fn test_adc_occur_overflow_minus() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x81, 0x00]);
        cpu.reset();
        cpu.register_a = 0x81;
        cpu.run();

        assert_eq!(cpu.register_a, 0x02);
        assert_eq!(cpu.status, 0x41);
    }

    #[test]
    fn test_adc_occur_overflow_minus_with_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x80, 0x00]);
        cpu.reset();
        cpu.register_a = 0x80;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, 0x41);
    }

    #[test]
    fn test_adc_no_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x7F, 0x00]);
        cpu.reset();
        cpu.register_a = 0x82;
        cpu.run();

        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_sbc_no_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE9, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0x20;
        cpu.run();

        assert_eq!(cpu.register_a, 0x0F);
        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_sbc_has_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE9, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0x20;
        cpu.status = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register_a, 0x10);
        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_sbc_occur_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE9, 0x02, 0x00]);
        cpu.reset();
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFE);
        assert_eq!(cpu.status, 0x80);
    }

    #[test]
    fn test_sbc_occur_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE9, 0x81, 0x00]);
        cpu.reset();
        cpu.register_a = 0x7F;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFD);
        assert_eq!(cpu.status, 0xC0);
    }

    #[test]
    fn test_sbc_occur_overflow_with_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE9, 0x81, 0x00]);
        cpu.reset();
        cpu.register_a = 0x7F;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFE);
        assert_eq!(cpu.status, 0xC0);
    }

    #[test]
    fn test_sbc_no_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE9, 0x7F, 0x00]);
        cpu.reset();
        cpu.register_a = 0x7E;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFF);
        assert_eq!(cpu.status, 0x80);
    }

    //
    #[test]
    fn test_and() {
        let cpu = run(vec![0x29, 0x0C, 0x00], |cpu| {
            cpu.register_a = 0x0A;
        });
        assert_eq!(cpu.register_a, 0x08);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_eor() {
        let cpu = run(vec![0x49, 0x0C, 0x00], |cpu| {
            cpu.register_a = 0x0A;
        });
        assert_eq!(cpu.register_a, 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_ora() {
        let cpu = run(vec![0x09, 0x0C, 0x00], |cpu| {
            cpu.register_a = 0x0A;
        });
        assert_eq!(cpu.register_a, 0x0E);
        assert_status(&cpu, 0);
    }

    // ASL
    #[test]
    fn test_asl_a() {
        let cpu = run(vec![0x0A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x03 * 2);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_asl_zero_page() {
        let cpu = run(vec![0x06, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
        assert_status(&cpu, 0);
    }
    #[test]
    fn test_asl_a_carry() {
        let cpu = run(vec![0x0A, 0x00], |cpu| {
            cpu.register_a = 0x81;
        });
        assert_eq!(cpu.register_a, 0x02);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_asl_zero_page_occur_carry() {
        let cpu = run(vec![0x06, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x81);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x02);
        assert_status(&cpu, FLAG_CARRY);
    }

    // LSR
    #[test]
    fn test_lsr_a() {
        let cpu = run(vec![0x4A, 0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_lsr_zero_page() {
        let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x02);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_lsr_zero_page_zero_flag() {
        let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x01);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x00);
        assert_status(&cpu, FLAG_ZERO | FLAG_CARRY);
    }

    #[test]
    fn test_lsr_a_occur_carry() {
        let cpu = run(vec![0x4A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_lsr_zero_page_occur_carry() {
        let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    // ROL
    #[test]
    fn test_rol_a() {
        let cpu = run(vec![0x2A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x03 * 2);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page() {
        let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
        assert_status(&cpu, 0);
    }
    #[test]
    fn test_rol_a_with_carry() {
        let cpu = run(vec![0x2A, 0x00], |cpu| {
            cpu.register_a = 0x03;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x03 * 2 + 1);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page_with_carry() {
        let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x03 * 2 + 1);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_a_zero_with_carry() {
        let cpu = run(vec![0x2A, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page_zero_with_carry() {
        let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x00);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    // ROR
    #[test]
    fn test_ror_a() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_ror_zero_page() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x02);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_ror_a_occur_carry() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_ror_zero_page_occur_carry() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_ror_a_with_carry() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x03;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x81);
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_zero_page_with_carry() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x81);
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_a_zero_with_carry() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x80);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_zero_page_zero_with_carry() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x00);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x80);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // BCC
    #[test]
    fn test_bcc() {
        let cpu = run(vec![0x90, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_bcc_with_carry() {
        let cpu = run(vec![0x90, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_CARRY;
        });

        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);

        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_bcc_negative() {
        let cpu = run(vec![0x90, 0xFC, 0x00], |cpu| {
            cpu.mem_write(0x7FFE, 0xE8);
            cpu.mem_write(0x7FFF, 0x00);
        });

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8000);
        assert_status(&cpu, 0);
    }

    // BCS
    #[test]
    fn test_bcs() {
        let cpu = run(vec![0xB0, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);

        assert_status(&cpu, 0);
    }

    #[test]
    fn test_bcs_with_carry() {
        let cpu = run(vec![0xB0, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_CARRY;
        });

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_bcs_negative() {
        let cpu = run(vec![0xB0, 0xFC, 0x00], |cpu| {
            cpu.mem_write(0x7FFE, 0xE8);
            cpu.mem_write(0x7FFF, 0x00);
            cpu.status = FLAG_CARRY;
        });

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8000);
        assert_status(&cpu, FLAG_CARRY);
    }

    // BEQ

    #[test]
    fn test_beq() {
        let cpu = run(vec![0xF0, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);

        assert_status(&cpu, 0);
    }

    #[test]
    fn test_beq_with_zero_flag() {
        let cpu = run(vec![0xF0, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_ZERO;
        });

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, 0);
    }

    // BNE
    #[test]
    fn test_bne() {
        let cpu = run(vec![0xD0, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_bne_with_zero_flag() {
        let cpu = run(vec![0xD0, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_ZERO;
        });

        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);

        assert_status(&cpu, FLAG_ZERO);
    }

    // BIT
    #[test]
    fn test_bit() {
        let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.mem_write(0x000, 0x00);
        });

        assert_status(&cpu, FLAG_ZERO);
    }

    #[test]
    fn test_bit_negative_flag() {
        let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.mem_write(0x000, 0x80);
        });

        assert_status(&cpu, FLAG_NEGATIVE | FLAG_ZERO);
    }

    #[test]
    fn test_bit_overflow_flag() {
        let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
            cpu.register_a = 0x40;
            cpu.mem_write(0x000, 0x40);
        });

        assert_status(&cpu, FLAG_OVERFLOW);
    }

    // BMI
    #[test]
    fn test_bmi() {
        let cpu = run(vec![0x30, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_bmi_with_negative_flag() {
        let cpu = run(vec![0x30, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, 0);
    }

    // BPL
    #[test]
    fn test_bpl() {
        let cpu = run(vec![0x10, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_bpl_with_negative_flag() {
        let cpu = run(vec![0x10, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });

        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // BVC
    #[test]
    fn test_bvc() {
        let cpu = run(vec![0x50, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_bvc_with_overflow_flag() {
        let cpu = run(vec![0x50, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu, FLAG_OVERFLOW);
    }

    // BVS
    #[test]
    fn test_bvs() {
        let cpu = run(vec![0x70, 0x02, 0x00, 0x00, 0xE8, 0x00], |_| {});

        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_bvs_with_overflow_flag() {
        let cpu = run(vec![0x70, 0x02, 0x00, 0x00, 0xE8, 0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu, FLAG_OVERFLOW);
    }

    // CLC
    #[test]
    fn test_clc() {
        let cpu = run(vec![0x18, 0x00], |cpu| {
            cpu.status = FLAG_CARRY | FLAG_NEGATIVE;
        });

        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // SEC
    #[test]
    fn test_sec() {
        let cpu = run(vec![0x38, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });

        assert_status(&cpu, FLAG_NEGATIVE | FLAG_CARRY);
    }

    // CLD
    #[test]
    fn test_cld() {
        let cpu = run(vec![0xD8, 0x00], |cpu| {
            cpu.status = FLAG_CARRY | FLAG_NEGATIVE | FLAG_DECIMAL;
        });

        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    // SED
    #[test]
    fn test_sed() {
        let cpu = run(vec![0xF8, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });

        assert_status(&cpu, FLAG_NEGATIVE | FLAG_DECIMAL);
    }

    // CLI
    #[test]
    fn test_cli() {
        let cpu = run(vec![0x58, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE | FLAG_DECIMAL | FLAG_INTERRUPT;
        });

        assert_status(&cpu, FLAG_NEGATIVE | FLAG_DECIMAL);
    }

    // SEI
    #[test]
    fn test_sei() {
        let cpu = run(vec![0x78, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });

        assert_status(&cpu, FLAG_NEGATIVE | FLAG_INTERRUPT);
    }

    // CLV
    #[test]
    fn test_clv() {
        let cpu = run(vec![0xB8, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE | FLAG_DECIMAL | FLAG_INTERRUPT | FLAG_OVERFLOW;
        });

        assert_status(&cpu, FLAG_NEGATIVE | FLAG_DECIMAL | FLAG_INTERRUPT);
    }

    // CMP
    #[test]
    fn test_cmp() {
        let cpu = run(vec![0xC9, 0x01], |cpu| {
            cpu.register_a = 0x02;
        });

        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_cmp_eq() {
        let cpu = run(vec![0xC9, 0x02], |cpu| {
            cpu.register_a = 0x02;
        });

        assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
    }

    #[test]
    fn test_cmp_negative() {
        let cpu = run(vec![0xC9, 0x03], |cpu| {
            cpu.register_a = 0x02;
        });

        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // CPX
    #[test]
    fn test_cpx() {
        let cpu = run(vec![0xE0, 0x01], |cpu| {
            cpu.register_x = 0x02;
        });

        assert_status(&cpu, FLAG_CARRY);
    }

    // CPY
    #[test]
    fn test_cpy() {
        let cpu = run(vec![0xC0, 0x01], |cpu| {
            cpu.register_y = 0x02;
        });

        assert_status(&cpu, FLAG_CARRY);
    }

}
