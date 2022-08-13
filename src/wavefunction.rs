use rand::prelude::*;

/// Stucture for holding the maching rules and assocatied data for a tile type.
/// T: assocatied data tye
/// N: the pattern size. MUST BE ODD
#[derive(Debug,Clone)]
pub struct Tile<T, const N: usize> {
    /// Additional data type, such as pixel value for image generation.
    pub additional: T,
    /// The freqency the tile should appear at, as a fraction of the sum of the weights of all
    /// tiles.
    pub weight: u32,
    /// A table of allowable ageccent tyles
    /// 0 : allowed
    /// 1 : disllowed
    pub mask: [[Vec<bool>; N]; N]
}

use std::iter;

impl<T, const N: usize> Tile<T,N> {
    pub fn allow_all(size:usize, additional: T) -> Tile<T,N> {
        let mask = vec![false; size];
        return Tile {
            additional,
            weight: 1,
            mask: iter::repeat(iter::repeat(mask).take(N).collect::<Vec<_>>().try_into().unwrap()).take(N).collect::<Vec<_>>().try_into().unwrap()
        }
    }
    pub fn disallow_all(size:usize, additional: T) -> Tile<T,N> {
        let mask = vec![true; size];
        return Tile {
            additional,
            weight: 1,
            mask: iter::repeat(iter::repeat(mask).take(N).collect::<Vec<_>>().try_into().unwrap()).take(N).collect::<Vec<_>>().try_into().unwrap()
        }
    }
    pub fn disallow(&mut self,id: usize) {
        for x in 0..N {
            for y in 0..N {
                self.mask[x][y][id] = true;
            }
        }
    }
    /// Same as dissalow, but ignores diagonals
    pub fn disallow_direct(&mut self,id: usize) {
        for x in 0..N {
            for y in 0..N {
                self.mask[x][y][id] = true;
            }
        }
        self.mask[0][0][id] = false;
        self.mask[0][N][id] = false;
        self.mask[N][1][id] = false;
        self.mask[N][N][id] = false;
    }
}

/// A Wave function collapse solver.
/// Generic over Pattern size and associated data type.
/// 
/// T: Data type for tiles.
/// N: Size of rules. (MUST BE ODD)
///
/// You should use the Wave::new() function to construct this to ensure you get a sane state.
/// 
/// The algorithm starts by assuming a state where every location is a super position of all
/// tiles. (.wave is all trues.)
/// 
/// Then until the wave is fully collapsed (one possibility per location):
///  0. Find lowest entropy tile, the one with the most information that has not been collapsed. (Least possibility's).
///  1. Collapse that tile by selecting a single allowed tile, removing other possibility's.
///  2. Use the rules to narrow down the possibility's for nearby tiles.
/// 
/// It is theoretic possible for a tile to have not possibility's, but this is very rare and not
/// handled here.
///
pub struct Wave<T: Clone, const N: usize> {
    /// A callback called on each step of the .collapse() method, I used this to make an animation
    /// of the algoritim.
    pub callback: Option<Box<dyn Fn (&Wave<T,N>, usize) -> ()>>,
    /// The pallet of tiles avalable, should not be modifyed ater creation.
    pub pallet: Vec<Tile<T,N>>,
    /// The pallet size, if this is not pallet.len(), weirdness will occur.
    pub pallet_size: usize,
    /// The actual wave function, 2D array of vec![bool; pallet_size]
    /// If true, the tile is possible at the location.
    pub wave: Vec<Vec<Vec<bool>>>,
    /// X and Y dimentions, this needs to match .wave
    pub x: usize,
    /// X and Y dimentions, this needs to match .wave
    pub y: usize,
    pub rng: rand::rngs::StdRng,
}

impl<T: Clone, const N: usize> Wave<T,N> {
    /// Create a solver, taking a tile pallet, size of image to generate and rng seed.
    /// Panics if x or y is zeor or the pallet is empty
    pub fn new(pallet: Vec<Tile<T,N>>, x: usize, y: usize, seed: u64) -> Wave<T,N> {
        // sanity check
        assert!(x > 1);
        assert!(y > 1);
        assert!(pallet.len() >= 1);
        let wave = vec![vec![vec![true; pallet.len()]; y]; x];

        Wave {
            callback: None,
            x,
            y,
            pallet_size: pallet.len(),
            pallet: pallet,
            wave,
            rng: rand::rngs::StdRng::seed_from_u64(seed)
        }
    }

    /// Get the entropy of a tile, returns f32::MAX for colapsed tiles, and contradictions
    // TODO take weight into account
    fn get_entropy(&self, x: usize, y: usize) -> f32 {
        let superposition = &self.wave[x][y];
        let mut count_allowed = 0;
        for bit in superposition {
            if *bit {
                count_allowed += 1;
            }
        }
        // Fudge entropy for colapsed tiles and contradictions
        if count_allowed == 1 || count_allowed == 0 {
            return f32::MAX;
        }
        return 1.0 - 1.0 / count_allowed as f32;
    }

    /// Get the lowest entropy tile, excluding fully colapsed tiles and contradictions
    pub fn get_lowest_entropy(&self) -> (usize, usize) {
        let mut best_x = 0;
        let mut best_y = 0;
        let mut best_e = f32::MAX;
        for x in 0..self.x {
            for y in 0..self.y {
                let e = self.get_entropy(x, y);
                if e < best_e {
                    best_e = e;
                    best_x = x;
                    best_y = y;
                }
            }
        }
        return (best_x, best_y);
    }

    /// Update the wavefunction of surrounding nodes
    /// This repatedy applys rules to reduce the enthropy as much as possible, and prevent
    /// contradictions.
    fn recursive_ruleset_apply(&mut self, x: usize, y:usize) {
        let mut stack = vec![(x,y)];
        let combined_mask = vec![true; self.pallet_size];
        let mut combined_mask: [[Vec<_>;N];N] = iter::repeat(iter::repeat(combined_mask).take(N).collect::<Vec<_>>().try_into().unwrap()).take(N).collect::<Vec<_>>().try_into().unwrap();
        
        while stack.len() > 0 {
            //println!("Stack size {} ", stack.len());
            let (x,y) = stack.pop().unwrap();
            // Find all allowed rulesets for current tile
            let allowed_idxs = self.wave[x][y].iter().enumerate().filter(|(_idx, v)| **v);
            let allowed_masks = allowed_idxs.map(|(idx, _v)| self.pallet[idx].mask.clone());
            // Initalizie all "true" mask.
            for i in 0..N {
                for e in 0..N {
                    for idx in 0..self.pallet_size {
                        combined_mask[i][e][idx] = true;
                    }
                }
            }
            // Combine all masks with and.
            for mask in allowed_masks {
                for x in 0..N {
                    for y in 0..N {
                        for idx in 0..self.pallet_size {
                            combined_mask[x][y][idx] &= mask[x][y][idx]
                        }
                    }
                }
            }
            // Apply combined mask
            for mask_x in 0..N {
                for mask_y in 0..N {
                    let offset_x = mask_x as isize - (N/2) as isize;
                    let offset_y = mask_y as isize - (N/2) as isize;
                    let wave_x = x as isize + offset_x;
                    let wave_y = y as isize + offset_y;
                    if wave_x >= 0 && wave_x < self.x as isize && wave_y >= 0 && wave_y < self.y as isize {
                        for id in 0..self.pallet_size {
                            let mut append_stack = false;
                            if combined_mask[mask_x][mask_y][id] {
                                if self.wave[wave_x as usize][wave_y as usize][id] {
                                    append_stack = true
                                }
                                self.wave[wave_x as usize][wave_y as usize][id] = false
                            }
                            if append_stack {
                                if !stack.contains(&(wave_x as usize, wave_y as usize)) {
                                    stack.push((wave_x as usize,wave_y as usize));
                                }
                            }
                        }
                    }
                }
            }
            
        }
    }

    /// Single step the wave-function-collapse algoritim
    /// Returns x, y, and collapsed idx of the tile
    pub fn step(&mut self) -> (usize, usize, usize) {
        let (best_x, best_y) = self.get_lowest_entropy();

        let superposition = &self.wave[best_x][best_y];
    
        let mut allowed = vec![];
        let mut weights = vec![];
        let mut total_allowed_weights = 0;

        for (idx, bit) in superposition.iter().enumerate() {
            if *bit {
                total_allowed_weights += self.pallet[idx].weight;
                allowed.push(idx);
                weights.push(self.pallet[idx].weight) 
            }
        }
      
        let rng = self.rng.next_u32();

        // weighted selection
        let rng = rng % total_allowed_weights;

        let mut current_weight_sum = 0;
        
        let mut selection = 0;

        for i in 0..allowed.len() {
            current_weight_sum += weights[i];
            if current_weight_sum > rng {
                selection = i;
                break;
            }
        }
        
        let selection = allowed[selection];
       
        let mut new_position = vec![false; self.pallet_size];

        new_position[selection] = true;

        self.wave[best_x][best_y] = new_position;

        self.recursive_ruleset_apply(best_x, best_y);

        return (best_x, best_y, selection)
    }

    /// Checks if the wave function is fully collapsed, returns true on contradiction.
    pub fn is_done(&self) -> bool {
        for x in 0..self.x {
            for y in 0..self.y {
                let superposition = &self.wave[x][y];
                let mut allowed = 0;
                for bit in superposition {
                    if *bit {
                        allowed += 1;
                    }
                }
                if allowed != 1 && allowed != 0 {
                    return false;
                }
            }
        }
        return true;
    }
    
    /// Checks if the function contains a contradiction.
    pub fn is_contradiction(&self) -> bool {
        for x in 0..self.x {
            for y in 0..self.y {
                let superposition = &self.wave[x][y];
                let mut allowed = 0;
                for bit in superposition {
                    if *bit {
                        allowed += 1;
                    }
                }
                if allowed == 0 {
                    return true;
                }
            }
        }
        return false;
    }
    
    /// Fully collapse a wavefunction, may produce a function with contradictions.
    /// Returns the count of steps it took to collapse.
    pub fn collapse(&mut self) -> usize {
        let mut count = 0;
        while !self.is_done() {
            self.step();
            count += 1;
            match &self.callback {
                Some(n) => n(self, count),
                None => ()
            }
        }
        return count;
    }

    /// Gets the tileid for a collapsed location in the wavefunction. None if it is not col;apsed.
    pub fn get_collapsed_tile(&self, x: usize, y: usize) -> Option<usize> {
        let superposition = &self.wave[x][y];
        let mut allowed = 0;
        let mut colapsed_idx = 0;
        for idx in 0..superposition.len() {
            let bit = superposition[idx];
            if bit {
                colapsed_idx = idx;
                allowed += 1;
            }
        }
        if allowed == 1 {
            return Some(colapsed_idx);
        } else {
            return None;
        }
    }

    /// Returns a 2dim vector containing tileids for all collapsed tiles, none if the wave is not
    /// colapsed.
    pub fn get_collapsed_vec(&self) -> Option<Vec<Vec<usize>>> {
        let mut buf = vec![];
        for x in 0..self.x {
            let mut col_buf = vec![];
            for y in 0..self.y {
                col_buf.push(self.get_collapsed_tile(x,y)?)
            }
            buf.push(col_buf);
        }
        Some(buf) 
    }
    /// Returns the assocated data for every tile in the wave, None if it is not fully collapsed.
    pub fn get_collapsed_data(&self) -> Option<Vec<Vec<&T>>> {
        let mut buf = vec![];
        for x in 0..self.x {
            let mut col_buf = vec![];
            for y in 0..self.y {
                let idx = self.get_collapsed_tile(x,y)?;
                let data = &self.pallet[idx].additional;
                col_buf.push(data);
            }
            buf.push(col_buf);
        }
        Some(buf) 
    }
}

#[cfg(test)]
mod tests {
    use super::Tile;
    use super::Wave;
    fn get_lowest_entropy() {
        let pallet = vec![Tile::<u32, 3>::allow_all(3, 0), Tile::allow_all(3, 0), Tile::allow_all(3, 0)];
        let mut wave = Wave::new(pallet, 3, 3, 123);
        wave.wave[2][1][0] = false;
        assert_eq!(wave.get_lowest_entropy(), (2, 1));
    }
    #[test]
    fn single_step() {
        let pallet = vec![Tile::<u32, 3>::allow_all(2, 0), Tile::allow_all(2, 0)];
        let mut wave = Wave::new(pallet, 3, 3, 123);
        wave.step();
    }
    #[test]
    fn full_collapse() {
        let pallet = vec![Tile::<u32, 3>::allow_all(2, 0), Tile::allow_all(2, 0)];
        let mut wave = Wave::new(pallet, 3, 3, 123);
        wave.colapse();
        println!("{:?}", wave);
        assert!(wave.is_done());
        assert!(!wave.is_contradiction());
    }

}
