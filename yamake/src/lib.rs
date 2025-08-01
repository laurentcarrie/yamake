/// the model providing traits for users to implement
pub mod model;

/// implementation for building C projects.
/// use it as doc
pub mod c_project;

pub(crate) mod run;

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
