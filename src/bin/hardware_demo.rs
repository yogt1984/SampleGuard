use sample_guard::hardware::HardwareDriver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SampleGuard Hardware Emulation Demo");
    println!("====================================\n");
    
    let mut driver = HardwareDriver::new();
    
    // Demonstrate system architecture understanding
    driver.demonstrate_architecture()?;
    
    Ok(())
}

