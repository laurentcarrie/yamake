///
/// scan and build rules for building a program written in C
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
