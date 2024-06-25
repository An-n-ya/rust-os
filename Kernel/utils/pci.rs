use super::port::Port;

pub struct PCI {
    address_port: Port,
    data_port: Port,
}

impl PCI {
    pub const fn new() -> Self {
        Self {
            address_port: Port::new(0xCF8),
            data_port: Port::new(0xCFC),
        }
    }

    pub fn read_config(&self, bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
        let (bus, slot, func, offset) = (bus as u32, slot as u32, func as u32, offset as u32);
        let address = 0x8000_0000 | (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC);

        self.address_port.write_u32(address);

        self.data_port.read_u32()
    }

    pub fn print_all_device(&self) {
        for bus in 0..=255 {
            for slot in 0..32 {
                let res = self.read_config(bus, slot, 0, 0);
                if res & 0xffff != 0xffff {
                    let vendor = res & 0xffff;
                    let device = res >> 16;
                    log!(
                        "pci: {:04x}:{:04x} bus: {bus}, slot: {slot}",
                        vendor,
                        device
                    );
                }
            }
        }
    }
}
