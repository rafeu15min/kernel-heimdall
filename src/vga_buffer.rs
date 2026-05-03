use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// O hardware VGA suporta essas 16 cores básicas.
// #[repr(u8)] garante que cada cor ocupe exatamente 1 byte (8 bits) na memória.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// Representa um byte de cor inteiro (4 bits para fundo, 4 bits para letra)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Representa exatamente como a placa de vídeo exige que uma letra seja montada na RAM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// O tamanho padrão da tela VGA é 80 colunas por 25 linhas
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// O Buffer em si. Usamos Volatile para o compilador não deletar nossas escritas!
#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// O nosso motorista do VGA: Ele lembra onde o cursor parou e qual a cor atual
pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    // Função interna para escrever apenas UM caractere (uma letra)
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // Se for Enter, pula linha
            byte => {
                // Se a linha encheu, pula linha automaticamente
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;

                // Escreve fisicamente na memória!
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    // A função simplificada de pular linha (Para a versão 1, ela apenas zera a coluna)
    // No futuro, implementaremos a rolagem da tela (scroll) aqui.
    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // O código de rolar a tela inteira pra cima (Scroll) vai entrar aqui no futuro!
        }
    }

    // Escreve uma string inteira pegando byte por byte
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            // Suporta apenas caracteres ASCII válidos (ou joga o bloco '■' pro resto)
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

// A MÁGICA: Ao implementar a interface 'fmt::Write', nós ganhamos o poder de usar
// as macros nativas do Rust (como write!) e formatar números e variáveis.
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// 1. O ESCRITOR GLOBAL SEGURO
lazy_static! {
    // WRITER é uma variável global, estática, protegida contra colisões de múltiplos núcleos.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// 2. A FUNÇÃO PONTE
// As macros não conseguem chamar os métodos diretamente de forma elegante,
// então criamos essa função escondida que trava o Mutex e escreve a string formatada.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Usamos o lock() para garantir exclusividade na memória de vídeo
    WRITER.lock().write_fmt(args).unwrap();
}

// 3. RECRIANDO AS MACROS NATIVAS
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
