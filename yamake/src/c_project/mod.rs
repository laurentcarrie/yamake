///
/// this module is light demo implementation of tool to compile C programs
///

/// scan a C file
pub mod c_file;

/// scan a H file
pub mod h_file;

/// build an object file
pub mod o_file;

/// link objects to produce an executable file
pub mod x_file;

/// scan a C or H file
pub mod c_scan;

/// call the gcc compiler
pub(crate) mod c_compile;

/// call the gcc linker
pub(crate) mod c_link;
