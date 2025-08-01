//! Configuration generation for network devices

pub mod cisco_ios;

/// Configuration generator trait
pub trait ConfigurationGenerator {
    /// Input type for the generator
    type Input;
    /// Output type (usually String)
    type Output;
    /// Error type
    type Error;
    
    /// Generate configuration from input
    fn generate(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
    
    /// Validate generated configuration
    fn validate(&self, output: &Self::Output) -> Result<(), Self::Error>;
}