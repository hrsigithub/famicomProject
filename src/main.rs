fn main() {
    println!("Hello, world!");
}

// a9 c0 aa e8 00

// LDA #$c0 ; a9 c0

pub struct CPU {
    // アキュムレータ
    pub register_a: u8,

    // インデックス レジスタ
    pub register_x: u8,

    // プロセッサ ステータス
    pub status: u8,

    // プログラム内の現在位置を追跡する
    // プログラム カウンター
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            // A9が取れる
            let opscode = program[self.program_counter as usize];
            self.program_counter += 1;
            println!("opscode:{}", opscode);
            // println!("opscode:{}", u8::from_str_radix(opscode, 16)?);

            match opscode {
                // LDA (0xA9)オペコード
                0xA9 => {
                    // パラメータ取得
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;

                    println!("param:{}", param);

                    self.register_a = param;

                    // メモリのバイトをアキュムレータにロードし、必要に応じてゼロと負のフラグを設定します。
                    // プロセッサ ステータス設定

                    // ビットは０から開始
                    // 1000 0000 => 7ビット目が立っている認識

                    // [Zero Flag 1ビット目] Aが0の時に設定
                    if self.register_a == 0 {
                        // 1bit目を立てる。
                        self.status = self.status | 0b0000_0010;
                    } else {
                        // 1bit目をクリア
                        self.status = self.status & 0b1111_1101;
                    }

                    // [Negative Flag 7ビット目] A のビット7(0b1000_0000)が設定されている場合に設定
                    if self.register_a & 0b1000_0000 != 0 {
                        // ビット7(0b1000_0000)が設定されている
                        self.status = self.status | 0b1000_0000;
                    } else {
                        // ビット7をクリア
                        self.status = self.status & 0b0111_1111;
                    }
                }

                // BRK(0x00)オペコード
                0x00 => {
                    return;
                }

                // TAX (0xAA)オペコード
                0xAA => {
                    self.register_x = self.register_a;

                    // [Zero Flag 1ビット目] Aが0の時に設定
                    if self.register_x == 0 {
                        // 1bit目を立てる。
                        self.status = self.status | 0b0000_0010;
                    } else {
                        // 1bit目をクリア
                        self.status = self.status & 0b1111_1101;
                    }

                    // [Negative Flag 7ビット目] A のビット7(0b1000_0000)が設定されている場合に設定
                    if self.register_x & 0b1000_0000 != 0 {
                        // ビット7(0b1000_0000)が設定されている
                        self.status = self.status | 0b1000_0000;
                    } else {
                        // ビット7をクリア
                        self.status = self.status & 0b0111_1111;
                    }
                }

                _ => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);

        // Aが0x05 に変わってるはずよ。
        assert_eq!(cpu.register_a, 0x05);

        assert!(cpu.status & 0b0000_0010 == 0b0000_0000);
        assert!(cpu.status & 0b1000_0000 == 0b0000_0000);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);

        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x80, 0x00]);

        assert!(cpu.status & 0b1000_0010 != 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.interpret(vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }
}
