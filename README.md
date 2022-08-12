# Yet Another Wave Function Collapse implementation

This is an unfinished pure rust implantation of the [Wave Function Collapse](https://github.com/mxgmn/WaveFunctionCollapse) Algorithm.

Example usage:

```rust
use image::io::Reader as ImageReader;
use image::RgbImage;
use yawfc::overlapping::overlapping;

let generated_size = 30;

// Read the example image
let img = ImageReader::open("in.png").unwrap().decode().unwrap().to_rgb8();
let img:Vec<Vec<_>> = img.rows().map(|x| x.map(|x| x.clone()).collect()).collect();

// Create the wave function from example image.
let mut wave = overlapping(img, generated_size, generated_size, true, false, since_the_epoch.as_millis() as u64);

// Collapse the wave function.
wave.colapse();

// Extract image data from solver and save with image crate.
let mut tiles: Vec<_> = wave.get_collapsed_data().unwrap().iter().flatten().flat_map(|x| x.0).collect();
let generated = RgbImage::from_raw(generated_size as u32, generated_size as u32, tiles).unwrap();
generated.save("out.png").unwrap();
```
