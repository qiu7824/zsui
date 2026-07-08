fn main() -> zsui::ZsuiResult<()> {
    zsui::native_window("ZSUI Native Window")
        .size(900, 620)
        .run()?;
    Ok(())
}
