use crate::utils::port::Port;

pub static PIC: PicController = PicController::new();

pub struct PicController {
    master: Pic,
    slave: Pic,
}

struct Pic {
    control: Port,
    data: Port,
}

impl PicController {
    const fn new() -> Self {
        let master = Pic {
            control: Port::new(0x20),
            data: Port::new(0x21),
        };
        let slave = Pic {
            control: Port::new(0xa0),
            data: Port::new(0xa1),
        };
        Self { master, slave }
    }

    pub fn init(&self) {
        // refer to https://wiki.osdev.org/8259_PIC
        // init master
        self.master.control.write_u8(0x11); // ICW1
        self.master.data.write_u8(0x20); // ICW2
        self.master.data.write_u8(0x04); // ICW3
        self.master.data.write_u8(0x01); // ICW4

        // init slave
        self.slave.control.write_u8(0x11); // ICW1
        self.slave.data.write_u8(0x28); // ICW2
        self.slave.data.write_u8(0x02); // ICW3
        self.slave.data.write_u8(0x01); // ICW4

        self.master.data.write_u8(0xf8); // open IRQ0 - time  IRQ1 - keyboard IRQ2 - slave
        self.slave.data.write_u8(0x3f); // open IRQ14,15 - ide

        log!("pic init complete");
    }

    pub fn eof(&self, index: u16) {
        assert!(index >= 0x20 && index <= 0x2f);
        if index >= 0x20 && index < 0x28 {
            self.master.control.write_u8(0x20);
        } else {
            self.master.control.write_u8(0x20);
            self.slave.control.write_u8(0x20);
        }
    }
}
