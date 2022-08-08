use rand::prelude::*;

#[derive(Debug,Clone)]
pub struct Tile {
    /// The Tile frequency weight
    pub weight: usize,
    /// A table of allowable agecent tyles
    /// 0 : allowed
    /// 1 : disllowed
    pub mask: [[Vec<bool>; 3]; 3]
}

impl Tile {
    pub fn allow_all(size:usize) -> Tile {
        let mask = vec![false; size];
        return Tile {
            weight: 1,
            mask: [[mask.clone(), mask.clone(), mask.clone()],[mask.clone(), mask.clone(), mask.clone()],[mask.clone(), mask.clone(), mask.clone()]]
        }
    }
    pub fn disallow(&mut self,id: usize) {
        for x in 0..3 {
            for y in 0..3 {
                self.mask[x][y][id] = true;
            }
        }
    }
}

#[derive(Debug,Clone)]
pub struct Wave {
    pub pallet_size: usize,
    pub pallet: Vec<Tile>,
    pub wave: Vec<Vec<Vec<bool>>>,
    pub x: usize,
    pub y: usize,
    pub rng: rand::rngs::StdRng,
}

impl Wave {
    pub fn new(pallet: Vec<Tile>, x: usize, y: usize, seed: u64) -> Wave {
        let wave = vec![vec![vec![true; pallet.len()]; y]; x];
        Wave {
            x,
            y,
            pallet_size: pallet.len(),
            pallet: pallet,
            wave,
            rng: rand::rngs::StdRng::seed_from_u64(seed)
        }
    }

    /// Get the entropy of a tile, returns f32::MAX for colapsed tiles.
    /// TODO take weight into account
    fn get_entropy(&self, x: usize, y: usize) -> f32 {
        let superposition = &self.wave[x][y];
        let mut count_allowed = 0;
        for bit in superposition {
            if *bit {
                count_allowed += 1;
            }
        }
        // Fudge entropy for colapsed tiles
        if count_allowed == 1 {
            return f32::MAX;
        }
        return 1.0 - 1.0 / count_allowed as f32;
    }

    /// Get the lowest entropy tile, excluding fully colapsed tiles
    pub fn get_lowest_entropy(&self) -> (usize, usize) {
        let mut best_x = 0;
        let mut best_y = 0;
        let mut best_e = f32::MAX;
        for x in 0..self.x {
            for y in 0..self.y {
                let e = self.get_entropy(x, y);
                println!("{} {} {}", best_x, best_y, e);
                if e < best_e {
                    best_e = e;
                    best_x = x;
                    best_y = y;
                }
            }
        }
        return (best_x, best_y);
    }

    /// Update superposition based on a ruleset mask
    fn apply_ruleset(&mut self, tileid: usize, x: usize, y: usize) {
        let rules = &self.pallet[tileid];

        for mask_x in 0..3 {
            for mask_y in 0..3 {
                let offset_x = mask_x as isize - 1;
                let offset_y = mask_y as isize - 1;
                let wave_x = offset_x + x as isize;
                let wave_y = offset_y + y as isize;
                if wave_x >= 0 && wave_x < self.x as isize && wave_y >= 0 && wave_y < self.y as isize {
                    for id in 0..self.pallet_size {
                        if rules.mask[mask_x][mask_y][id] {
                            self.wave[wave_x as usize][wave_y as usize][id] = false
                        }
                    }
                }
            }
        }
    }

    /// Single step the wave-function-colapse algoritim
    pub fn step(&mut self) {
        let (best_x, best_y) = self.get_lowest_entropy();

        println!("best {} {}", best_x, best_y);

        let superposition = &self.wave[best_x][best_y];
    
        let mut allowed = vec![];

        for (idx, bit) in superposition.iter().enumerate() {
            if *bit {
                // TODO this is not optimal
                for _ in 0..self.pallet[idx].weight {
                    allowed.push(idx);
                }
            }
        }

        let rng = self.rng.next_u32() as usize % allowed.len();
        
        let selection = allowed[rng];
        
        println!("{} ", selection);

        let mut new_position = vec![false; self.pallet_size];

        new_position[selection] = true;

        self.wave[best_x][best_y] = new_position;

        self.apply_ruleset(selection, best_x, best_y);
    }

    /// Checks if the wave function is fully colapsed
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
                if allowed != 1 {
                    return false;
                }
            }
        }
        return true;
    }
    
    /// Fully colapse a wavefunction.
    pub fn colapse(&mut self) -> u32 {
        let mut count = 0;
        while !self.is_done() {
            self.step();
            count += 1;
        }
        return count;
    }

    /// Gets the tileid for a colapsed location in the wavefunction. None if it is not colapsed
    pub fn get_colapsed_tile(&self, x: usize, y: usize) -> Option<usize> {
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
}

#[cfg(test)]
mod tests {
    use super::Tile;
    use super::Wave;
    #[test]
    fn get_lowest_entropy() {
        let pallet = vec![Tile::allow_all(3), Tile::allow_all(3), Tile::allow_all(3)];
        let mut wave = Wave::new(pallet, 3, 3);
        wave.wave[2][1][0] = false;
        assert_eq!(wave.get_lowest_entropy(), (2, 1));
    }
    #[test]
    fn single_step() {
        let pallet = vec![Tile::allow_all(2), Tile::allow_all(2)];
        let mut wave = Wave::new(pallet, 3, 3);
        wave.step();
    }
    #[test]
    fn full_colapse() {
        let pallet = vec![Tile::allow_all(2), Tile::allow_all(2)];
        let mut wave = Wave::new(pallet, 3, 3);
        wave.colapse();
        println!("{:?}", wave);
        assert!(wave.is_done())
    }

}
