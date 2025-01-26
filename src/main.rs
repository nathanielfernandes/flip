use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;
use image::{codecs::gif::GifEncoder, Frame};

#[derive(Parser, Debug)]
#[command(name = "flip")]
#[command(author = "Nathaniel F. <nathaniel.s.fernandes@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "flip an image into a gif", long_about = None)]
struct Args {
    image: PathBuf,
    #[clap(
        short = 'o',
        long = "output",
        help = "output file path",
    )]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    
    let start = std::time::Instant::now();
    let Ok(image) = image::open(&args.image) else {
        eprintln!("failed to open image: `{}` :(", args.image.display());
        std::process::exit(1);
    };

    let output_path = match args.output {
        Some(output) => output,
        None => {
            let mut path = args.image.clone();
            path.set_extension("gif");
            path
        }
    };

    let Ok(mut output) = File::create(&output_path) else {
        eprintln!("failed to create output file: `{}` :(", output_path.display());
        std::process::exit(1);
    };

    let out_display = output_path.display();
    print!("{out_display}: flipping...");
    std::io::stdout().flush().expect("whoops...");

    let mut encoder = GifEncoder::new(&mut output);

    let frame = Frame::new(image.into_rgba8());
    if let Err(_) = encoder.encode_frame(frame) {
        eprintln!("failed to encode image: `{}` :(", args.image.display());
        std::process::exit(1);
    }

    let duration = start.elapsed();
    println!("\r{out_display}: done in {duration:.2?}          ");  
}
