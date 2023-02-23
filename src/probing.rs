use std::ops::Add;

pub trait BuildProbing {
    type Probing: Probing;
    fn build_probing(&self, cap: usize) -> Self::Probing;
}

pub trait Probing {
    type ProbingIter: Iterator<Item=usize>;
    fn probe(&self, hash: u64) -> Self::ProbingIter;
}

#[derive(Default)]
pub struct BuildLinearProbing;

pub struct LinearProbing {
    cap: usize,
    mask: usize,
}

#[derive(Default)]
pub struct BuildTriangularProbing;

pub struct TriangularProbing {
    cap: usize,
    mask: usize,
}

impl BuildProbing for BuildLinearProbing {
    type Probing = LinearProbing;
    fn build_probing(&self, cap: usize) -> Self::Probing {
        assert!(cap.is_power_of_two());
        LinearProbing {
            cap,
            mask: cap.saturating_sub(1),
        }
    }
}

impl Probing for LinearProbing {
    type ProbingIter = impl Iterator<Item=usize>;
    fn probe(&self, hash: u64) -> Self::ProbingIter {
        let mask = self.mask;
        (0..self.cap).map(move |x| (x + hash as usize) & mask)
    }
}

impl BuildProbing for BuildTriangularProbing {
    type Probing = TriangularProbing;
    fn build_probing(&self, cap: usize) -> Self::Probing {
        TriangularProbing { cap, mask: cap.saturating_sub(1) }
    }
}

impl Probing for TriangularProbing {
    type ProbingIter = impl Iterator<Item=usize>;
    fn probe(&self, hash: u64) -> Self::ProbingIter {
        let mask = self.mask;
        (0..self.cap).scan(0, |x, y| Some(*x + y)).map(move |x| (x + hash as usize) & mask)
    }
}