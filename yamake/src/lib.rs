//! yamake
//!
//!
//!
#![doc = include_str!("doc.md")]

// use simple_mermaid::mermaid;

#[doc = mermaid!("graph1.md")]

/// the model providing traits for users to implement
pub mod model;

/// implementation for building C projects.
/// use it as doc
pub mod rules;

pub(crate) mod run;

pub(crate) mod target_hash;

pub(crate) mod test_c;
pub(crate) mod test_latex;

pub mod error;
pub mod helpers;

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
