use image::imageops::FilterType;
use indicatif::{ProgressBar, ProgressStyle};

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Get the first argument passed to the program
    let args: Vec<String> = std::env::args().collect();
    let img_path = match args.get(1) {
        Some(path) => path.to_string(),
        None => {
            eprintln!("Usage: {} <image>", args[0]);
            std::process::exit(1);
        }
    };
    println!("Image path: {}", img_path);


    println!("Handling image...");

    // Create a progress bar for the grayscale conversion
    let style = match ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
    {
        Ok(style) => style.progress_chars("#>-"),
        Err(err) => {
            eprintln!("Failed to create progress style: {}", err);
            std::process::exit(1);
        }
    };
    let pb1 = ProgressBar::new(100);
    pb1.set_style(style.clone());

    // Create a channel for progress bar to work
    let (tx, rx) = mpsc::channel();
    let pb1_clone = pb1.clone();
    thread::spawn(move || {
        let mut i = 0;
        while rx.try_recv().is_err() {
            pb1_clone.inc(1);
            let delay = (100_f64*1.02_f64.powf(i as f64)).floor() as u64;
            thread::sleep(Duration::from_millis(delay)); // Exponential backoff
            i += 1;
        }
        pb1_clone.finish_with_message("Grayscale conversion done");
    });

    // Load the image and convert it to grayscale
    let grayscale =  match image::open(&img_path){
        Ok(img) => img.to_luma8(),
        Err(err) => {
            eprintln!("Failed to open the image\nPerhaps the image path is incorrect or the image is not supported\nError: {}", err);
            std::process::exit(1);
        }
    };

    // Send a message to the channel to stop the progress bar
    tx.send(()).unwrap();

    // Resize the image
    let (width, height) = grayscale.dimensions();
    println!("Image dimensions: {}x{}", width, height);
    let scale = ((width as f32 / 1920.0 * 20.0).ceil()) as u32; // Calculate the scale factor to fit the image to the terminal
    println!("Scale: 1/{}", scale);
    println!("Resizing...");
    let img = image::imageops::resize(&grayscale, width / scale, height / scale, FilterType::Nearest);

    // Get the dimensions of the resized image
    let (width, height) = img.dimensions();
    println!("Resized dimensions: {}x{}", width, height);

    // Create a progress bar for the ASCII conversion
    let pb2 = ProgressBar::new(height as u64);
    pb2.set_style(style);

    let mut ascii_string = String::new(); // A string to store the horizontal ASCII line
    let mut ascii_image = String::new(); // A string to store the whole ASCII image

    for y in 0..height{
        ascii_string.clear(); // Clear the string for the new line
        for x in 0..width{
            let gray_pixel = img.get_pixel(x, y);
            let scaled_value = (gray_pixel[0] as f32 / 255.0 * 12.0).round() as u8;
            let ascii_pixel = scaled_value % 12;
            let pixel_char: &str = match ascii_pixel { // .,-~:;=!*#$@
                0 => "@@",
                1 => "$$",
                2 => "##",
                3 => "**",
                4 => "!!",
                5 => "==",
                6 => ";;",
                7 => "::",
                8 => "~~",
                9 => "--",
               10 => ",,",
               11 => "..",
                _ => panic!("Error")
            };
            ascii_string.push_str(&pixel_char);
        }
        pb2.inc(1);
        ascii_string.push('\n');
        ascii_image.push_str(&ascii_string);
    }
    pb2.finish_with_message("Done!");
    print!("{}[2J", 27_u8 as char); // Clear the terminal
    println!("{}", ascii_image);
}