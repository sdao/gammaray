use core::Vec;
use prim::Prim;

pub struct KernelResult {
    pub color: Vec,
    pub direction: Vec,
}

pub trait Kernel {
    fn bounce(&self, dist: f64, normal: &Vec, prim: &Box<Prim + Sync>) -> KernelResult;
}

pub struct DisplayColorKernel {
}

impl DisplayColorKernel {
    pub fn new() -> DisplayColorKernel {
        DisplayColorKernel {}
    }
}

impl Kernel for DisplayColorKernel {
    fn bounce(&self, _: f64, _: &Vec, prim: &Box<Prim + Sync>) -> KernelResult {
        KernelResult {color: prim.display_color().clone(), direction: Vec::zero()}
    }
}
