use core;

use prim;

pub struct KernelResult {
    pub color: core::Vec,
    pub direction: core::Vec,
}

pub trait Kernel {
    fn bounce(&self, dist: f64, normal: &core::Vec, prim: &Box<prim::Prim + Sync>) -> KernelResult;
}

pub struct DisplayColorKernel {
}

impl DisplayColorKernel {
    pub fn new() -> DisplayColorKernel {
        DisplayColorKernel {}
    }
}

impl Kernel for DisplayColorKernel {
    fn bounce(&self, _: f64, _: &core::Vec, prim: &Box<prim::Prim + Sync>) -> KernelResult {
        KernelResult {color: prim.display_color().clone(), direction: core::Vec::zero()}
    }
}
