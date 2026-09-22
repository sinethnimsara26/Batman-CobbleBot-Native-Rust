/// Owned software-rendering target.
///
/// Gameplay remains in the original 600x400 logical coordinate system.  The
/// surface dimensions describe presentation pixels only, which lets later HQ
/// phases allocate a larger target without leaking presentation scale into
/// collision, camera, animation or input code.
#[derive(Debug)]
pub struct RenderSurface {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<u32>,
}

impl RenderSurface {
    pub fn new(w: usize, h: usize) -> Self {
        assert!(w > 0 && h > 0, "render surface dimensions must be non-zero");
        Self {
            w,
            h,
            pixels: vec![0; w * h],
        }
    }

    pub fn fill(&mut self, pixel: u32) {
        self.pixels.fill(pixel);
    }

    pub fn byte_len(&self) -> usize {
        self.pixels.len() * std::mem::size_of::<u32>()
    }
}
