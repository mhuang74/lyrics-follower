use hound;
use cqt_rs::{CQTParams, Cqt};
use ndarray::{Array, Array2};
use image::{ImageBuffer, Rgb};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use std::env;

fn visualize_array2_as_image(input: &Array2<f32>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    // Convert to DB value by taking log10 and times 20
    let array = input.mapv(f32::abs).mapv(f32::log10).mapv(|val| val * 20.0);
    
    // Normalize the values in the array to the [0, 255] range for visualization
    let min_val = array.iter().copied().reduce(f32::min).unwrap();
    let max_val = array.iter().copied().reduce(f32::max).unwrap();
    println!("min: {}, max: {}", min_val, max_val);
    
    let normalized_array = (255.0 * (array.to_owned().mapv(|x| (x - min_val) / (max_val - min_val)))).mapv(|x| x as u8);

    
    // Create an image buffer from the 2D array
    let width = normalized_array.shape()[0] as u32;
    let height = normalized_array.shape()[1] as u32;
    let mut img = ImageBuffer::new(width as u32, height as u32);

    // Fill the image buffer with pixel values
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let val = normalized_array[[x as usize, (height-1_u32-y) as usize]];
        *pixel = Rgb([val, 30, 30])
    }


    // Save the image to the specified filename
    img.save(filename)?;

    Ok(())
}

fn main() {

    // Get the command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if at least one argument (filename) is provided
     if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    // The first argument (index 1) is the filename
    let filename = &args[1];   

    let mut reader = hound::WavReader::open(filename).unwrap();

    println!("wav spec: {:?}, wav duration: {:?}", reader.spec(), reader.duration());

    let sampling_rate:usize = reader.spec().sample_rate as usize;

    println!("Sampling Rate: {}", sampling_rate);

    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();

    let input_signal: Vec<f32> = samples.iter().map(|&x| x as f32).collect();; // Your input audio signal


    let cqt_params = CQTParams::new(
        100.0, // Min frequency
        20000.0_f32, // Max frequency
        25, // Number of bins
        sampling_rate, // Sampling rate
        sampling_rate/28 // Window length
    ).expect("Error creating CQTParams");

    let cqt = Cqt::new(cqt_params);



    let cqt_features:Array2<f32> = cqt.process(&input_signal, 1600).expect("Error computing CQT features");


    visualize_array2_as_image(&cqt_features, "output.png").unwrap();


}


