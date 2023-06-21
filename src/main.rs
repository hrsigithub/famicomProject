fn main() {
    println!("Hello, world!");
}

// a9 c0 aa e8 00

// LDA #$c0 ; a9 c0

pub struct CPU {
    // アキュムレータ
    pub register_a: u8,

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

                    // [Zero Flag] Aが0の時に設定
                    if self.register_a == 0 {
                        // 1bit目を立てる。
                        self.status = self.status | 0b0000_0010;
                    } else {
                        // 1bit目をクリア
                        self.status = self.status & 0b1111_1101;
                    }

                    // [Negative Flag] A のビット7(0b1000_0000)が設定されている場合に設定
                    if self.register_a & 0b1000_0000 != 0 {
                        // ビット7(0b1000_0000)が設定されている
                        self.status = self.status | 0b1000_0000;
                    } else {
                        // ビット7をクリア
                        self.status = self.status & 0b0111_0000;
                    }
                }

                // BRK(0x00)オペコード
                0x00 => {
                    return;
                }

                _ => todo!(),
            }
        }
    }
}
