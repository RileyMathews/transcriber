fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let _filename = args.get(1).map(|s| s.as_str()).expect("no file given");

    Ok(())
}
