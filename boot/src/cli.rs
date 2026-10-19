//! Hardware-independent line editor and boot command dispatcher (no allocator).
use core::fmt::{self, Write};

pub const LINE_CAPACITY: usize = 128;
const OUTPUT_CAPACITY: usize = 2048;
pub const PROMPT: &str = "boot> ";
pub const SLOT_A: u32 = 0x0802_0000;
pub const SLOT_B: u32 = 0x0812_0000;
pub const SLOT_SIZE: u32 = 896 * 1024;

pub struct BoardInfo {
    pub version: &'static str,
    pub sysclk: u32,
    pub hclk: u32,
    pub pclk1: u32,
    pub pclk2: u32,
    pub usbclk: u32,
    pub uid: [u32; 3],
    pub vectors: [[u32; 2]; 2],
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Reboot,
}

/// Only checks vector plausibility; this is NOT an image integrity check.
pub fn vectors_plausible(base: u32, [sp, reset]: [u32; 2]) -> bool {
    let stack_in_ram =
        (sp > 0x2000_0000 && sp <= 0x2003_0000) || (sp > 0x1000_0000 && sp <= 0x1001_0000);
    stack_in_ram
        && sp & 7 == 0
        && reset & 1 == 1
        && (base + 8..base + SLOT_SIZE).contains(&(reset & !1))
}

pub struct Console {
    line: [u8; LINE_CAPACITY],
    len: usize,
    overflow: bool,
    invalid: bool,
    skip_lf: bool,
    output: [u8; OUTPUT_CAPACITY],
    output_len: usize,
    sent: usize,
}

impl Console {
    pub const fn new() -> Self {
        Self {
            line: [0; LINE_CAPACITY],
            len: 0,
            overflow: false,
            invalid: false,
            skip_lf: false,
            output: [0; OUTPUT_CAPACITY],
            output_len: 0,
            sent: 0,
        }
    }

    pub fn open(&mut self) {
        *self = Self::new();
        self.write_str("\r\nSTM32F439 USB boot CLI\r\nType help for commands.\r\n")
            .unwrap();
        self.write_str(PROMPT).unwrap();
    }

    pub fn pending(&self) -> &[u8] {
        &self.output[self.sent..self.output_len]
    }

    /// Advance only by the number of bytes accepted by the USB driver.
    pub fn consume(&mut self, count: usize) {
        assert!(count <= self.pending().len());
        self.sent += count;
        if self.sent == self.output_len {
            self.sent = 0;
            self.output_len = 0;
        }
    }

    /// The transport must drain output before supplying the next input byte.
    pub fn receive(&mut self, byte: u8, info: &BoardInfo) -> Action {
        assert!(self.pending().is_empty());
        if byte == b'\n' && self.skip_lf {
            self.skip_lf = false;
            return Action::None;
        }
        self.skip_lf = byte == b'\r';
        match byte {
            b'\r' | b'\n' => {
                self.write_str("\r\n").unwrap();
                let action = if self.overflow {
                    self.write_str("ERR line too long (max 128 bytes); discarded\r\n")
                        .unwrap();
                    Action::None
                } else if self.invalid {
                    self.write_str("ERR unsupported input; discarded\r\n")
                        .unwrap();
                    Action::None
                } else {
                    self.dispatch(info)
                };
                self.len = 0;
                self.overflow = false;
                self.invalid = false;
                if action == Action::None {
                    self.write_str(PROMPT).unwrap();
                }
                action
            }
            3 => {
                self.len = 0;
                self.overflow = false;
                self.invalid = false;
                self.write_str("^C\r\n").unwrap();
                self.write_str(PROMPT).unwrap();
                Action::None
            }
            8 | 127 if self.len > 0 && !self.overflow => {
                self.len -= 1;
                self.write_str("\x08 \x08").unwrap();
                Action::None
            }
            b' '..=b'~' | b'\t' if !self.overflow => {
                if self.len == LINE_CAPACITY {
                    // Never execute a truncated command, including after backspace.
                    self.overflow = true;
                    self.write_str("\x07").unwrap();
                } else {
                    self.line[self.len] = byte;
                    self.len += 1;
                    self.write_char(char::from(byte)).unwrap();
                }
                Action::None
            }
            // Do not splice a command together by silently dropping arbitrary
            // binary/control bytes. Backspace on an empty line is harmless.
            8 | 127 => Action::None,
            b' '..=b'~' | b'\t' => Action::None,
            _ => {
                self.invalid = true;
                Action::None
            }
        }
    }

    fn dispatch(&mut self, info: &BoardInfo) -> Action {
        // Copy so command parsing does not borrow the output writer.
        let line = self.line;
        let text = core::str::from_utf8(&line[..self.len]).unwrap();
        let mut args = text.split_ascii_whitespace();
        let Some(command) = args.next() else {
            return Action::None;
        };
        if args.next().is_some() {
            self.write_str("ERR commands take no arguments\r\n")
                .unwrap();
            return Action::None;
        }
        match command {
            "help" | "?" => self
                .write_str(concat!(
                    "help / ?  Show commands\r\n",
                    "info      Firmware, chip UID and clocks\r\n",
                    "layout    Planned Boot / App A/B flash map\r\n",
                    "slots     Inspect A/B vectors (NOT image verification)\r\n",
                    "reboot    System reset; return to this CLI\r\n",
                    "No flash programming or app jump in this demo.\r\n",
                ))
                .unwrap(),
            "info" => {
                writeln!(self, "boot {} / STM32F439ZI\r", info.version).unwrap();
                writeln!(
                    self,
                    "UID={:08X}{:08X}{:08X}\r",
                    info.uid[0], info.uid[1], info.uid[2]
                )
                .unwrap();
                writeln!(self, "SYSCLK={} HCLK={} Hz\r", info.sysclk, info.hclk).unwrap();
                writeln!(
                    self,
                    "PCLK1={} PCLK2={} USB={} Hz\r",
                    info.pclk1, info.pclk2, info.usbclk
                )
                .unwrap();
            }
            "layout" => self
                .write_str(concat!(
                    "Planned map (app still builds as standalone):\r\n",
                    "Boot    08000000..0801FFFF  128 KiB S0-S4\r\n",
                    "App A   08020000..080FFFFF  896 KiB S5-S11\r\n",
                    "State 0 08100000..08103FFF   16 KiB S12\r\n",
                    "State 1 08104000..08107FFF   16 KiB S13\r\n",
                    "Params  08108000..0810FFFF   32 KiB S14-S15\r\n",
                    "Reserve 08110000..0811FFFF   64 KiB S16\r\n",
                    "App B   08120000..081FFFFF  896 KiB S17-S23\r\n",
                ))
                .unwrap(),
            "slots" => {
                for (name, base, vectors) in [
                    ("A", SLOT_A, info.vectors[0]),
                    ("B", SLOT_B, info.vectors[1]),
                ] {
                    let status = if vectors_plausible(base, vectors) {
                        "plausible"
                    } else {
                        "invalid"
                    };
                    writeln!(
                        self,
                        "{} @ {:08X}: MSP={:08X} RESET={:08X} {}\r",
                        name, base, vectors[0], vectors[1], status
                    )
                    .unwrap();
                }
                self.write_str("Vector check only; CRC/signature/boot state not implemented.\r\n")
                    .unwrap();
            }
            "reboot" => {
                self.write_str("OK rebooting...\r\n").unwrap();
                return Action::Reboot;
            }
            _ => self
                .write_str("ERR unknown command; type help\r\n")
                .unwrap(),
        }
        Action::None
    }
}

impl Write for Console {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.output_len + text.len();
        if end > OUTPUT_CAPACITY {
            return Err(fmt::Error);
        }
        self.output[self.output_len..end].copy_from_slice(text.as_bytes());
        self.output_len = end;
        Ok(())
    }
}

// These tests use the host's allocator and test harness, not defmt-test on ARM.
#[cfg(all(test, not(target_os = "none")))]
mod tests {
    extern crate std;

    use self::std::string::String;
    use super::*;

    fn info() -> BoardInfo {
        BoardInfo {
            version: "test",
            sysclk: 168_000_000,
            hclk: 168_000_000,
            pclk1: 42_000_000,
            pclk2: 84_000_000,
            usbclk: 48_000_000,
            uid: [1, 2, 3],
            vectors: [[u32::MAX; 2]; 2],
        }
    }

    fn feed(console: &mut Console, bytes: &[u8]) -> (String, Action) {
        let mut output = String::new();
        let mut action = Action::None;
        for &byte in bytes {
            action = console.receive(byte, &info());
            output.push_str(core::str::from_utf8(console.pending()).unwrap());
            console.consume(console.pending().len());
        }
        (output, action)
    }

    #[test]
    fn crlf_is_one_command_and_fragmented_lines_survive() {
        let mut console = Console::new();
        feed(&mut console, b"in");
        let (output, _) = feed(&mut console, b"fo\r\n");
        assert!(output.contains("USB=48000000"));
        assert_eq!(output.matches(PROMPT).count(), 1);
        assert_eq!(feed(&mut console, b"reboot\n").1, Action::Reboot);
    }

    #[test]
    fn editing_cancellation_and_multiple_lines() {
        let mut console = Console::new();
        assert!(
            feed(&mut console, b"helx\x08p\n")
                .0
                .contains("Show commands")
        );
        assert!(feed(&mut console, b"infX\x7fo\n").0.contains("UID="));
        let (output, action) = feed(&mut console, b"reboot\x03help\nslots\n");
        assert_eq!(action, Action::None);
        assert!(!output.contains("OK rebooting"));
        assert!(output.contains("MSP=FFFFFFFF"));
    }

    #[test]
    fn overlong_line_is_discarded_then_recovers() {
        let mut console = Console::new();
        feed(&mut console, &[b' '; LINE_CAPACITY]);
        let (output, action) = feed(&mut console, b"reboot\x08\nhelp\n");
        assert_eq!(action, Action::None);
        assert!(output.contains("line too long"));
        assert!(!output.contains("OK rebooting"));
        assert!(output.contains("Show commands"));
    }

    #[test]
    fn invalid_commands_never_reset() {
        let mut console = Console::new();
        for command in [
            b"reboot now\n".as_slice(),
            b"boot a\n",
            b"erase\n",
            b"\n",
            b"\xff\n",
            b"reb\xffoot\n",
            b"reb\x00oot\n",
        ] {
            assert_eq!(feed(&mut console, command).1, Action::None);
        }
    }

    #[test]
    fn pending_output_survives_partial_writes_and_reopen_resets_input() {
        let mut console = Console::new();
        console.open();
        let output = console.pending().to_vec();
        console.consume(3);
        console.consume(0);
        assert_eq!(console.pending(), &output[3..]);
        console.consume(output.len() - 3);
        feed(&mut console, b"rebo");
        console.open();
        console.consume(console.pending().len());
        assert_eq!(feed(&mut console, b"ot\n").1, Action::None);
    }

    #[test]
    fn validates_stack_thumb_and_slot_boundaries() {
        assert!(vectors_plausible(SLOT_A, [0x2003_0000, SLOT_A + 9]));
        assert!(vectors_plausible(SLOT_B, [0x1001_0000, SLOT_B + 9]));
        for vectors in [
            [u32::MAX; 2],
            [0x2000_0000, SLOT_A + 9],
            [0x2003_0008, SLOT_A + 9],
            [0x2002_0004, SLOT_A + 9],
            [0x2002_0000, SLOT_A + 8],
            [0x2002_0000, SLOT_A + 1],
            [0x2002_0000, SLOT_B + 9],
            [0x2002_0000, SLOT_A + SLOT_SIZE + 1],
        ] {
            assert!(!vectors_plausible(SLOT_A, vectors));
        }
    }
}
