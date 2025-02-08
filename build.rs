use std::env;
use std::path::PathBuf;

fn main() {
    // Set linker script path
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Configure for ATmega128
    println!("cargo:rustc-link-arg=-mmcu=atmega128");
    
    // Pass CPU frequency for timing calculations
    println!("cargo:rustc-env=MCU_FREQ_HZ=16000000");
    
    // Debug vs Release configurations
    if env::var("PROFILE").unwrap() == "debug" {
        println!("cargo:rustc-cfg=feature=\"debug\"");
        // Enable debug assertions
        println!("cargo:rustc-cfg=debug_assertions");
    }
    
    // Check if we're building for hardware-in-loop tests
    if env::var("CARGO_FEATURE_HIL_TESTS").is_ok() {
        println!("cargo:rustc-cfg=feature=\"hil_tests\"");
    }
    
    // Ensure target is correct
    let target = env::var("TARGET").unwrap();
    if !target.contains("avr") {
        panic!("This crate only supports AVR targets!");
    }
    
    // Output helpful build information
    println!("cargo:warning=Building for ATmega128 at 16MHz");
    println!("cargo:warning=Output directory: {}", out_dir.display());
    
    // TODO: Add more conditional compilation flags as needed
    // TODO: Generate linker script if custom memory layout needed
} 