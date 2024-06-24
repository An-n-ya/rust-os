pub mod buf;
pub mod ide;
pub mod partition_table;

use self::buf::Buf;
use self::ide::ide_start;

pub fn test_ide_read() {
    for i in 0..10 {
        let buf = Buf::new(i);
        ide_start(&buf);
        log!("{i}: {:x?}", buf.data);
    }
}
