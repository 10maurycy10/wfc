use crate::Wave;
use crate::Tile;

use std::fmt::Debug;
use std::collections::HashMap;
use std::hash::Hash;

pub type Overlapping<PixelType, const N: usize> = Wave<PixelType, N>;

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
struct Pattern<T: PartialEq + Hash + Clone + Debug> {
    pixel_data: [[T; 3]; 3],
    freq: usize
}
//```
//(0,0) (0,1) (0,2)
//(1,0) (1,1) (0,2)
//(2,0) (2,1) (2,2)
//```
impl<T: PartialEq + Hash + Clone + Debug> Pattern<T> {
    fn extract(src: &Vec<Vec<T>>, startx: usize, starty: usize) -> Pattern<T> {
        Pattern {
            pixel_data: [
                [src[startx+0][starty+0].clone(),src[startx+0][starty+1].clone(),src[startx+0][starty+2].clone()],
                [src[startx+1][starty+0].clone(),src[startx+1][starty+1].clone(),src[startx+1][starty+2].clone()],
                [src[startx+2][starty+0].clone(),src[startx+2][starty+1].clone(),src[startx+2][starty+2].clone()]
            ],
            freq: 1
        }
    }
    
    fn fromdata(src: [[T; 3]; 3], f:usize) -> Pattern<T> {
        Pattern {
            pixel_data: src,
            freq: f
        }
    }
    fn y_mirror(&self) -> Pattern<T> {
        let mut data = self.pixel_data.clone();
        data.iter_mut().map(|x| x.reverse()).for_each(drop);
        //assert!(self.pixel_data != data);
        Pattern::fromdata(data, self.freq)
    }
    fn x_mirror(&self) -> Pattern<T> {
        let mut data = self.pixel_data.clone();
        data.reverse();
        Pattern::fromdata(data, self.freq)
    }
}

/// Full deduplication
fn dedup<T: Hash+Clone+Debug+PartialEq+Eq>(array: &mut Vec<Pattern<T>>) {
    let mut items: HashMap<Pattern<T>, usize> = HashMap::new();
    let mut cursor = 0;
    while array.len() > cursor {
        match items.get(&array[cursor]) {
            Some(idx) => {
                array.remove(cursor);
                array[*idx].freq += 1;
            }
            None => {
                items.insert(array[cursor].clone(), cursor);
                cursor += 1;
            }
        }
    }
}

#[test]
fn dedup_test() {
    let mut arr = vec![0, 1, 2, 1, 3, 3, 3, 4, 5, 4,1,1,1,1,7,1];
    dedup(&mut arr);
    assert_eq!(arr, vec![0,1,2,3,4,5,7]);
}

/// Create a wave function collapse solver for texture generation with the overlapping model.
///
/// N is hard coded at 3 (for now).
///
/// The generated output has 2 constraints:
/// - All N*N patterns in the output are present in the input. (Rotation and mirroring is  optionally allowed).
/// - The ratios of the patterns in the output should be approximately the same as the input for a large enough sample size. (This is achieved through weighting the rng.)
///
/// This is achieved by extracting all N*N patterns from the image. (No boundary conditions for
/// now, you have to pad your image.)
///
/// Then computing all possible ways the patterns can overlap without conflicting. (This is blazing
/// fast O(n^2*N^2))
///
/// These rules are then passed to the solver.
pub fn overlapping<T: Debug + Hash + PartialEq + Eq + Clone>(
    image: Vec<Vec<T>>,
    resulty: usize,
    resultx: usize,
    mirror: bool,
    debug: bool,
    seed: u64,
) -> Overlaping<T,5> {
   
    let mut patterns: Vec<Pattern<T>> = vec![]; 

    let imagex = image.len();
    assert!(imagex >= 3);
    let imagey = image[0].len();
    assert!(imagey >= 3);

    // For all pattern locations
    for x in 0..(imagex-2) {
        for y in 0..(imagey-2) {
            // extract the patterns into the buffer
            patterns.push(Pattern::extract(&image, x, y))
        }
    }

    // deduplicate
    dedup(&mut patterns);


    if mirror {
        let mut buf = vec![];
        for pattern in &patterns {
            buf.push(pattern.y_mirror());
        }
        patterns.append(&mut buf);
        for pattern in &patterns {
            buf.push(pattern.x_mirror());
        }
        patterns.append(&mut buf);
    }
    // TODO transform

    // Repeat deduplication because tranformations create a *lot* of duplicates.
    dedup(&mut patterns);

    // Find valid ajacent patterns O(n^2)
    // For all ids -> 3*3 array -> array of pattern idxs
    let mut valid_neighbors: Vec<[[Vec<usize>; 5]; 5]> = vec![];

    for _ in 0..patterns.len() {
        valid_neighbors.push([
            [vec![], vec![], vec![], vec![], vec![]],
            [vec![], vec![], vec![], vec![], vec![]],
            [vec![], vec![], vec![], vec![], vec![]],
            [vec![], vec![], vec![], vec![], vec![]],
            [vec![], vec![], vec![], vec![], vec![]],
        ])
    }

    // For all pattern combinations
    for a_idx in 0..patterns.len() {
        for b_idx in 0..patterns.len() {
            let a = &patterns[a_idx];
            let b = &patterns[b_idx];
            // For all possible offsets...
            for x in 0..5 {
                for y in 0..5 {
                    let dx = x - (5/2 as isize);
                    let dy = y - (5/2 as isize);
                    let mut allowed = true;
                    // For all pixels in pattern a...
                    for pattern_a_x in 0..(3 as isize) {
                        for pattern_a_y in 0..(3 as isize) {
                            let pattern_b_x = pattern_a_x - dx;
                            let pattern_b_y = pattern_a_y - dy;
                            // If that pixel is in pattern b
                            if pattern_b_x >= 0 && pattern_b_y >= 0 && pattern_b_x < 3 && pattern_b_y < 3 {
                                // if the overlaping pixels dont match, disallow pairing
                                if b.pixel_data[pattern_b_x as usize][pattern_b_y as usize] != a.pixel_data[pattern_a_x as usize][pattern_a_y as usize] {
                                    allowed = false;
                                }
                            }
                        }
                    }
                    // If it was not regected, add to parings
                    if allowed {
                        valid_neighbors[a_idx][x as usize][y as usize].push(b_idx);
                    }
                }
            }
        }
    }

    // debug information
    if debug {
        for (n,pattern) in patterns.iter().enumerate() {
            println!("-- PATTERN -- {}", n);
            for line in &pattern.pixel_data {
                println!("{:?}", line);
            }
        }
    
        for (n,t) in valid_neighbors.iter().enumerate() {
            println!("{}: {:?}",n, t);
        }
    }

    // Construct tile data
    let mut tile_buffer = vec![];

    for (idx, pattern) in patterns.iter().enumerate() {
        // Start by disallowing all connections, setting the addironal data to the center pixel of
        // the pattern.
        let mut tile = Tile::disallow_all(patterns.len(), pattern.pixel_data[1][1].clone());

        // Allow centers
        for i in 0..patterns.len() {
            tile.mask[2][2][i] = false;
        }
        
        let neighbors = &valid_neighbors[idx];

        for x in 0..5 {
            for y in 0..5 {
                for neighbor_idx in &neighbors[x][y] {
                    tile.mask[x][y][*neighbor_idx] = false;
                }
            }
        }

        tile.weight = pattern.freq as u32;

        tile_buffer.push(tile);
    }

    // Construct the wave function.
    return Wave::new(tile_buffer, resultx, resulty, seed);
}
#[test]
fn test() {
    let img = vec![
        vec![0, 1, 2],
        vec![3, 4, 5],
        vec![6, 7, 8]
    ];
    let l = overlapping(img, 5, 5, true, 123).colapse();
}
