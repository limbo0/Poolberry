//! Detect price discrepancies across liquidity sources.  
//! ____________________________________________________________________________
pub mod identify;
pub mod select;

pub use identify::*;
pub use select::*;
