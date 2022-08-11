use crate::Wave;
use crate::Tile;

use std::fmt::Debug;
use std::collections::{HashSet, HashMap};
use std::hash::Hash;

pub type Overlaping<PixelType, const N: usize> = Wave<PixelType, N>;

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
struct Pattern<T: PartialEq + Hash + Clone + Debug> {
    pixel_data: [[T; 3]; 3]
}
///```
///(0,0) (0,1) (0,2)
///(1,0) (1,1) (0,2)
///(2,0) (2,1) (2,2)
///```
impl<T: PartialEq + Hash + Clone + Debug> Pattern<T> {
    fn extract(src: &Vec<Vec<T>>, startx: usize, starty: usize) -> Pattern<T> {
        Pattern {
            pixel_data: [
                [src[startx+0][starty+0].clone(),src[startx+0][starty+1].clone(),src[startx+0][starty+2].clone()],
                [src[startx+1][starty+0].clone(),src[startx+1][starty+1].clone(),src[startx+1][starty+2].clone()],
                [src[startx+2][starty+0].clone(),src[startx+2][starty+1].clone(),src[startx+2][starty+2].clone()]
            ]
        }
    }
    fn x(&self) -> [&T; 3] {
        [&self.pixel_data[0][1], &self.pixel_data[1][1], &self.pixel_data[2][1]]
    }
    fn y(&self) -> [&T; 3] {
        [&self.pixel_data[1][0], &self.pixel_data[1][1], &self.pixel_data[1][2]]
    }
    fn top(&self) -> [&T; 3] {
        [&self.pixel_data[0][0], &self.pixel_data[0][1], &self.pixel_data[0][2]]
    }
    fn bottom(&self) -> [&T; 3] {
        [&self.pixel_data[2][0], &self.pixel_data[2][1], &self.pixel_data[2][2]]
    }
    fn left(&self) -> [&T; 3] {
        [&self.pixel_data[0][0], &self.pixel_data[1][0], &self.pixel_data[2][0]]
    }
    fn right(&self) -> [&T; 3] {
        [&self.pixel_data[0][2], &self.pixel_data[1][2], &self.pixel_data[2][2]]
    }

    fn bottom_right(&self) -> [&T; 4] {
        [&self.pixel_data[1][1], &self.pixel_data[0][1], &self.pixel_data[2][1], &self.pixel_data[2][2]]
    }
    fn bottom_left(&self) -> [&T; 4] {
        [&self.pixel_data[1][0], &self.pixel_data[1][1], &self.pixel_data[2][0], &self.pixel_data[2][1]]
    }
    fn top_left(&self) -> [&T; 4] {
        [&self.pixel_data[0][0], &self.pixel_data[0][1], &self.pixel_data[1][0], &self.pixel_data[1][1]]
    }
    fn top_right(&self) -> [&T; 4] {
        [&self.pixel_data[0][1], &self.pixel_data[0][2], &self.pixel_data[1][1], &self.pixel_data[0][2]]
    }
    
    fn fromdata(src: [[T; 3]; 3]) -> Pattern<T> {
        Pattern {
            pixel_data: src
        }
    }
}

/// Full deduplication
fn dedup<T: Hash + PartialEq + Eq + Clone>(array: &mut Vec<T>) {
    let mut items: HashSet<T> = HashSet::new();
    let mut cursor = 0;
    while array.len() > cursor {
        match items.get(&array[cursor]) {
            Some(_) => {
                array.remove(cursor);
            }
            None => {
                items.insert(array[cursor].clone());
                cursor += 1;
            }
        }
    }
}

//fn mirror<T: Hash + Clone + Debug + PartialEq + Eq>(input: &Vec<Pattern<T>>) -> Vec<Pattern<T>> {
//    let buffer = vec![];
//    for pattern in input {
//        let mut xmirror = pattern.pixel_data.clone();
//        xmirror.reverse();
//        let ymirror: Vec<_> = pattern.pixel_data.iter().map(|x| {let mut m = x.clone();m.reverse();m}).collect();
//        let xymirror = ymirror.iter().clone();
//        buffer.push(Pattern::fromdata(xmirror));
//        buffer.push(Pattern::fromdata(ymirror));
//        buffer.push(Pattern::fromdata(xymirror));
//    }
//    return buffer;
//}

pub fn overlaping<T: Debug + Hash + PartialEq + Eq + Clone>(
    image: Vec<Vec<T>>,
    resulty: usize,
    resultx: usize,
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

    // TODO transform

    // Repeat deduplication because tranformations create a *lot* of duplicates.
    dedup(&mut patterns);

    // Find valid ajacent patterns O(n^2)
    // For all ids -> 3*3 array -> array of pattern idxs
    let mut valid_neighbors: Vec<[[Vec<usize>; 5]; 5]> = vec![];

    for i in 0..patterns.len() {

        // (0,0) (0,1) (0,2) (0,3) (0,4)
        // (1,0) (1,1) (1,2) (1,3) (1,4)
        // (2,0) (2,1) (2,2) (2,3) (2,4)
        // (3,0) (3,1) (3,2) (3,3) (3,4)
        // (4,0) (4,1) (4,2) (4,3) (4,4)
        //
        // (0,0) (0,1) (0,2)
        // (1,0) (1,1) (1,2)
        // (2,0) (2,1) (2,2)
        //
        // C U U U C
        // U C C C U
        // U C   C U
        // U C C C U
        // C U U U C
        //
        //       DS            DS
        //          DD  D   DD 
        //          D   A   D  
        //          DD  D   DD
        //       DS            DS
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
                    println!("x {} y {} dx {} dy {}", x, y, dx, dy);
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
                    if (allowed) {
                        valid_neighbors[a_idx][x as usize][y as usize].push(b_idx);
                    }
                }
            }
        }
    }

    // debug information
    for (n,pattern) in patterns.iter().enumerate() {
        println!("-- PATTERN -- {}", n);
        for line in &pattern.pixel_data {
            println!("{:?}", line);
        }
    }

    for (n,t) in valid_neighbors.iter().enumerate() {
        println!("{}: {:?}",n, t);
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
    let l = overlaping(img, 5, 5, 123).colapse();
}