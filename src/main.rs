use std::{fs::File, io::Write, path::PathBuf};

use clap::{Parser, ValueEnum};
use glob::glob;
use image::{Frame, GenericImageView, codecs::gif::GifEncoder, imageops::FilterType};

#[derive(ValueEnum, Clone, Debug, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum Filter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

#[derive(Parser, Debug)]
#[command(name = "flip")]
#[command(author = "Nathaniel F. <nathaniel.s.fernandes@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "flip an image into a gif", long_about = None)]
struct Args {
    #[clap(help = "the images to flip, supports glob patterns")]
    images: String,

    #[clap(
        short = 'd',
        long = "destory",
        help = "destroy image after flip",
        default_value = "false"
    )]
    destroy: bool,

    #[clap(
        short = 's',
        long = "scale",
        help = "scale output gif by this value",
        default_value = "1.0"
    )]
    scale: f32,

    #[clap(
        long = "filter",
        help = "which sampling filter to use for scaling",
        default_value = "lanczos3"
    )]
    filter: Filter,

    #[clap(
        short = 'c',
        long = "crop",
        help = "crop in input image by this value in pixels",
        default_value = "0"
    )]
    crop: u32,
}

macro_rules! error {
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!($fmt, $($arg)*);
        std::process::exit(1);
    };
}

fn flip(image_path: &PathBuf, scale: f32, filter: FilterType, crop: u32) -> Result<(), String> {
    let start = std::time::Instant::now();

    let Ok(mut image) = image::open(image_path) else {
        return Err(format!(
            "failed to open image: `{}` :(",
            image_path.display()
        ));
    };

    let (w, h) = image.dimensions();

    if crop > 0 {
        // we don't crop more than the image dimensions
        let crop_x = (2 * crop).min(w);
        let crop_y = (2 * crop).min(h);

        // only crop if we have enough image left
        if w > crop_x && h > crop_y {
            let new_w = w - crop_x;
            let new_h = h - crop_y;

            // crop equally from all sides
            image = image.crop(crop, crop, new_w, new_h);
        } else {
            return Err(format!(
                "crop value {} too large for image dimensions {}x{}, skipping crop",
                crop, w, h
            ));
        }
    }

    // apply scaling after crop
    let (w, h) = image.dimensions();
    image = image.resize(
        ((w as f32 * scale).round() as u32).max(2),
        ((h as f32 * scale).round() as u32).max(2),
        filter,
    );

    let mut output_path = image_path.clone();
    output_path.set_extension("gif");

    let Ok(mut output) = File::create(&output_path) else {
        return Err(format!(
            "failed to create output file: `{}` :(",
            output_path.display()
        ));
    };

    let output_display = output_path.display();
    print!("{output_display}: flipping...");
    std::io::stdout().flush().expect("whoops...");

    let mut encoder = GifEncoder::new(&mut output);

    let frame = Frame::new(image.into_rgba8());
    if let Err(_) = encoder.encode_frame(frame) {
        return Err(format!(
            "failed to encode image: `{}` :(",
            image_path.display()
        ));
    }

    let duration = start.elapsed();
    println!("\r{output_display}: done in {duration:.2?}          ");

    return Ok(());
}

fn main() {
    let args = Args::parse();

    let start = std::time::Instant::now();
    let Ok(paths) = glob(&args.images) else {
        error!("failed to read glob pattern: {}", &args.images);
    };

    let scale = args.scale.min(10.0);
    let filter = match args.filter {
        Filter::Nearest => FilterType::Nearest,
        Filter::Triangle => FilterType::Triangle,
        Filter::CatmullRom => FilterType::CatmullRom,
        Filter::Gaussian => FilterType::Gaussian,
        Filter::Lanczos3 => FilterType::Lanczos3,
    };

    let mut destroy: Vec<PathBuf> = Vec::new();
    for entry in paths.flatten() {
        match flip(&entry, scale, filter, args.crop) {
            Ok(()) => {
                destroy.push(entry);
            }
            Err(msg) => eprintln!("{msg}"),
        }
    }

    let n = destroy.len();
    if args.destroy {
        for entry in destroy {
            if let Err(e) = std::fs::remove_file(&entry) {
                eprintln!(
                    "failed to delete original file: {} ({})",
                    entry.display(),
                    e
                );
            }
        }
    }

    let duration = start.elapsed();
    println!("{} images flipped in {duration:.2?}", n);
}
