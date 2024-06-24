// use core::io;

// pub fn read_struct<T>(r: &mut impl Read) -> io::Result<T> {
//     let size = core::mem::size_of::<T>();
//     unsafe {
//         let mut s = core::mem::MaybeUninit::<T>::uninit();
//         let buf = core::slice::from_raw_parts_mut(s.as_mut_ptr() as *mut u8, size);
//         match r.read_exact(buf) {
//             Ok(_) => Ok(s.assume_init()),
//             Err(e) => {
//                 core::mem::forget(s);
//                 Err(e)
//             }
//         }
//     }
// }
