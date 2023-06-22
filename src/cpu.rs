// a9 c0 aa e8 00

// LDA #$c0 ; a9 c0

pub struct CPU {
    // アキュムレータ
    pub register_a: u8, // 1byte

    // インデックス レジスタ
    pub register_x: u8,

    // プロセッサ ステータス
    pub status: u8,

    // プログラム内の現在位置を追跡する
    // プログラム カウンター
    pub program_counter: u16, // 2Byte

    // メモリー
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
            memory: [0x00; 0xFFFF],
        }
    }

    // メモリアドレッシングモード
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
        // self.register_a = 0;
        // self.register_x = 0;
        // self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    // 0x8000 から、カトリッジのデータをロード
    // プログラムを PRG ROM 空間にロードし、コードへの参照を 0xFFFC メモリ セルに保存する必要があります。
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // [Zero Flag 1ビット目] Aが0の時に設定
        if result == 0 {
            // 1bit目を立てる。
            self.status = self.status | 0b0000_0010;
        } else {
            // 1bit目をクリア
            self.status = self.status & 0b1111_1101;
        }

        // [Negative Flag 7ビット目] A のビット7(0b1000_0000)が設定されている場合に設定
        if result & 0b1000_0000 != 0 {
            // ビット7(0b1000_0000)が設定されている
            self.status = self.status | 0b1000_0000;
        } else {
            // ビット7をクリア
            self.status = self.status & 0b0111_1111;
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        // オーバーフロー対応
        // if self.register_x == 0xFF {
        //     self.register_x = 0;
        // } else {
        //     self.register_x += 1;
        // }
        self.register_x = self.register_x.wrapping_add(1);

        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            println!("opscode:{:X}", opscode);

            match opscode {
                // LDA (0xA9)オペコード
                0xA9 => {
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;
                    self.lda(param);
                }

                // TAX (0xAA)オペコード
                0xAA => self.tax(),

                // INX (0xE8)オペコード
                0xE8 => self.inx(),

                // BRK(0x00)オペコード
                0x00 => return,

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

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        // Aが0x05 に変わってるはずよ。
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
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.load_and_run(vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1);
    }
}
