#[derive(Debug)]
#[repr(C, packed)]
// refer to https://wiki.osdev.org/Partition_Table#MBR
// actually cylinder should have been 10bit, and sector should have been 6 bit,
// but we don't use these field in fact, and these fields don't result in skew offset
// so I just use u8 to represent them
struct DiskPartitionTable {
    loadable: u8,
    start_head: u8,
    start_sector: u8,
    start_cylinder: u8,
    _type: u8,
    end_head: u8,
    end_sector: u8,
    end_cylinder: u8,
    offset: u32,
    size: u32,
}

struct Partition {
    start: u32,
    size: u32,
}

// fn test() {
//     let mut file = File::open("fs.img").expect("cannot open fs file");
//     let mut buf = vec![];
//     file.read_to_end(&mut buf).expect("cannot read from fs.img");
//     let mut cursor = Cursor::new(buf);
//     cursor.set_position(0x1be);
//     // read 4 partition table
//     let mut tables = vec![];
//     let mut extend_tables = vec![];
//     let mut partitions = vec![];
//     let mut main_offset = 0;
//     for _ in 0..4 {
//         let table =
//             read_struct::<DiskPartitionTable>(&mut cursor).expect("read partition table failed");
//         if table._type == 0x05 || table._type == 0x0f {
//             // extended table entry
//             main_offset = table.offset;
//         } else if table._type != 0 {
//             partitions.push(Partition {
//                 start: table.offset,
//                 size: table.size,
//             });
//             tables.push(table);
//         }
//     }

//     dbg!(main_offset);

//     if main_offset != 0 {
//         cursor.set_position((main_offset * 512) as u64 + 0x1be);
//         let mut last_offset = 0;
//         loop {
//             let mut extend_offset = 0;
//             for _ in 0..4 {
//                 let table = read_struct::<DiskPartitionTable>(&mut cursor)
//                     .expect("read partition table failed");
//                 if table._type == 0x05 || table._type == 0x0f {
//                     // extended table entry
//                     extend_offset = table.offset;
//                 } else if table._type != 0 {
//                     partitions.push(Partition {
//                         start: table.offset + main_offset + last_offset,
//                         size: table.size,
//                     });
//                     extend_tables.push(table);
//                 }
//             }
//             // dbg!(extend_offset);
//             if extend_offset != 0 {
//                 last_offset = extend_offset;
//                 cursor.set_position(
//                     (main_offset * 512) as u64 + (extend_offset * 512) as u64 + 0x1be,
//                 );
//             } else {
//                 break;
//             }
//         }
//     }

//     // let mut buf = [0u8; 64];
//     // cursor
//     //     .read_exact(&mut buf)
//     //     .expect("cannot read from cursor");
//     // println!("{:x?}", buf);
//     println!("{:x?}", tables);
//     for t in extend_tables {
//         println!("{:x?}", t);
//     }
//     for (i, p) in partitions.iter().enumerate() {
//         println!(
//             "partition {i}: start: {:x}, end: {:x}",
//             p.start,
//             p.start + p.size
//         );
//     }
// }
