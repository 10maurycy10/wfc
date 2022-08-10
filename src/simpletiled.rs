use crate::Wave;
use crate::Tile;

use std::fmt::Debug;

type ImageData<PixelType> = Vec<Vec<PixelType>>;

type STMWave<PixelType, const N: usize> = Wave<ImageData<PixelType>, N>;

fn simple_tiled<T: Debug, const N: usize>(
    image: Vec<Vec<T>>,
    tilex: usize, tiley: usize
) -> STMWave<T,N> {
    
    let imagex = image.len();
    assert!(imagex > 0);
    let imagey = image[0].len();
    assert!(imagey > 0);
    
    let mut array: Vec<Vec<Vec<&T>>> = vec![];
    for x in 0..(imagex/tilex) {
        for y in 0..(imagey/tiley) {
            let start_x = x*tilex;
            let start_y = y*tiley;
            let mut tile = vec![];
            for i in 0..tilex {
                let mut col_buf = vec![];
                let col = &image[i+start_x];
                for e in 0..tiley {
                    col_buf.push(&col[e+start_y]);
                }
                tile.push(col_buf);
            }
            array.push(tile);
        }
    }


    println!("{:?}", array);
    panic!();
}
