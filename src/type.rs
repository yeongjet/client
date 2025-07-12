use network_direct::Win32::System::IO::OVERLAPPED;
pub type Overlap = OVERLAPPED;
// bug: {U2, U540, U960, U8, op} removed the unused import -> use typenum::op;

// type U1920 = op!(U960 * U2);
// type U1080 = op!(U540 * U2);

// pub type Res1920x1080 = op!(U1920 * U1080);

// pub mod buffer_size {


//     use crate::r#type::{Res1920x1080, U1080};

//     pub type Window1 = Res1920x1080;
//     pub type Total = Window1;
// }

// // type BufferSize = op!(op!(Window1Res + Window2Len) + Window3Len);
