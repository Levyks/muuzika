use crate::proto::common::RoomCode;
use rand::prelude::{SliceRandom, StdRng};
use rand::{rng, SeedableRng};
use std::collections::VecDeque;

pub struct RoomCodeGeneratorImpl {
    initial_power: u32,
    rng: StdRng,
    collections: Vec<RoomCodeCollection>,
    capacity_threshold_to_remove: usize,
}

#[derive(Debug)]
pub struct RoomCodeCollection {
    power: u32,
    codes: VecDeque<u32>,
}

impl RoomCodeCollection {
    fn new(rng: &mut StdRng, min: u32, power: u32) -> Self {
        let mut codes = (min..10u32.pow(power + 1)).collect::<Vec<u32>>();
        codes.shuffle(rng);
        Self {
            power,
            codes: codes.into(),
        }
    }

    fn is_empty(&self) -> bool {
        self.codes.is_empty()
    }

    fn available(&self) -> usize {
        self.codes.len()
    }

    fn pop_front(&mut self) -> Option<u32> {
        self.codes.pop_front()
    }

    fn push_back(&mut self, code: u32) {
        self.codes.push_back(code);
    }
}

pub trait RoomCodeGenerator {
    fn new_code(&mut self) -> RoomCode;
    fn return_code(&mut self, code: RoomCode);
    fn has_available_code(&self) -> bool;
    fn create_next_collection(&mut self) -> &mut RoomCodeCollection;
}

impl RoomCodeGenerator for RoomCodeGeneratorImpl {
    fn new_code(&mut self) -> RoomCode {
        let collection =
            if let Some(collection) = self.collections.iter_mut().find(|c| !c.is_empty()) {
                collection
            } else {
                self.create_next_collection()
            };

        collection.pop_front().unwrap().into()
    }

    fn return_code(&mut self, code: RoomCode) {
        let power = code.code.ilog10().max(self.initial_power);

        let idx = if let Some((idx, _)) = self
            .collections
            .iter()
            .enumerate()
            .find(|(_, c)| c.power == power)
        {
            idx
        } else {
            return;
        };

        self.collections[idx].push_back(code.into());

        if self.collections[idx].available() > self.capacity_threshold_to_remove {
            self.collections.truncate(idx + 1);
        }
    }


    fn has_available_code(&self) -> bool {
        self.collections.iter().any(|c| !c.is_empty())
    }

    fn create_next_collection(&mut self) -> &mut RoomCodeCollection {
        let power = if let Some(last) = self.collections.last() {
            last.power + 1
        } else {
            self.initial_power
        };
        let collection = RoomCodeCollection::new(&mut self.rng, 10u32.pow(power), power);
        self.collections.push(collection);
        self.collections.last_mut().unwrap()
    }
}

impl RoomCodeGeneratorImpl {
    pub fn new(seed: Option<u64>, initial_power: u32, capacity_threshold_to_remove: usize) -> Self {
        let mut s = Self {
            initial_power,
            rng: seed
                .map(StdRng::seed_from_u64)
                .unwrap_or_else(|| StdRng::from_rng(&mut rng())),
            collections: Vec::with_capacity(1),
            capacity_threshold_to_remove,
        };

        let initial_collection = RoomCodeCollection::new(&mut s.rng, 1, initial_power);
        s.collections.push(initial_collection);

        s
    }

    fn generate_random_codes(&mut self, min: u32, max: u32) -> VecDeque<u32> {
        let mut codes = (min..max).collect::<Vec<u32>>();
        codes.shuffle(&mut self.rng);
        codes.into()
    }
}

impl From<u32> for RoomCode {
    fn from(code: u32) -> Self {
        Self { code }
    }
}

impl From<RoomCode> for u32 {
    fn from(code: RoomCode) -> Self {
        code.code
    }
}

#[cfg(test)]
mod tests {
    use super::{RoomCodeGenerator, RoomCodeGeneratorImpl};
    use crate::proto::common::RoomCode;

    static SEED: Option<u64> = Some(42);

    #[test]
    fn test_generates_a_code_on_correct_range() {
        let mut generator = RoomCodeGeneratorImpl::new(SEED, 3, 0);
        let code = generator.new_code();
        assert!(code.code < 10000);
    }

    #[test]
    fn test_code_returned_can_be_reused() {
        let mut generator = RoomCodeGeneratorImpl::new(SEED, 0, 0);
        let code = generator.new_code();
        for _ in 0..8 {
            generator.new_code();
        }
        generator.return_code(code);
        let new_code = generator.new_code();
        assert_eq!(code.code, new_code.code);
    }

    #[test]
    fn test_code_gets_bigger_when_exhausted() {
        let mut generator = RoomCodeGeneratorImpl::new(SEED, 0, 0);
        for _ in 0..9 {
            let code = generator.new_code();
            assert!(code.code < 10);
        }
        assert_eq!(generator.collections.len(), 1);
        let code = generator.new_code();
        assert_eq!(generator.collections.len(), 2);
        assert!(code.code >= 10);
        assert!(code.code < 100);
    }

    #[test]
    fn test_generates_a_code_on_the_smallest_available_collection() {
        let mut generator = RoomCodeGeneratorImpl::new(SEED, 0, 0);
        for _ in 0..10 {
            generator.new_code();
        }
        let code_1 = generator.new_code();
        let code_2 = generator.new_code();

        generator.return_code(code_1);
        generator.return_code(RoomCode { code: 7 });
        generator.return_code(code_2);

        let code = generator.new_code();
        assert_eq!(code.code, 7);
    }

    #[test]
    fn test_collection_is_removed_when_capacity_threshold_is_reached() {
        let mut generator = RoomCodeGeneratorImpl::new(SEED, 1, 20);

        // exhausting first collection (1-99)
        for _ in 1..=99 {
            generator.new_code();
        }

        assert_eq!(generator.collections.len(), 1);

        // forcing next collection to be created
        let code_2nd_collection = generator.new_code();
        assert_eq!(generator.collections.len(), 2);

        // return 20 codes of the first collection
        for i in 1..=20 {
            generator.return_code(RoomCode { code: i });
        }

        // there should still be 2 collections
        assert_eq!(generator.collections.len(), 2);

        // return 1 more code of the first collection
        generator.return_code(RoomCode { code: 21 });

        // now the second collection should be removed
        assert_eq!(generator.collections.len(), 1);

        // returning a code from the 2nd collection shouldn't break things
        generator.return_code(code_2nd_collection);
    }

    #[test]
    fn test_has_available_code() {
        let mut generator = RoomCodeGeneratorImpl::new(SEED, 0, 0);
        assert!(generator.has_available_code());
        for _ in 0..9 {
            generator.new_code();
        }
        assert!(!generator.has_available_code());
    }

    #[test]
    fn test_create_next_collection() {
        let mut generator = RoomCodeGeneratorImpl::new(SEED, 0, 0);
        for _ in 0..9 {
            generator.new_code();
        }
        assert!(!generator.has_available_code());
        assert_eq!(generator.collections.len(), 1);
        generator.create_next_collection();
        assert!(generator.has_available_code());
        assert_eq!(generator.collections.len(), 2);
    }
}
