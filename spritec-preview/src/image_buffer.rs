use euc::Target;
use vek::Rgba;

/// The number of components in an RGBA value (always 4)
const RGBA_COMPONENTS: usize = 4;

/// An image data buffer (compatible with JavaScript's ImageData)
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    data: Vec<u8>,
    width: usize,
    height: usize,
    scale: usize,
}

impl ImageBuffer {
    pub fn new(width: usize, height: usize, scale: usize) -> Self {
        Self {
            data: vec![0; RGBA_COMPONENTS * width * scale * height * scale],
            width,
            height,
            scale,
        }
    }

    /// Returns a pointer to the data buffer compatible with JavaScript's Uint8ClampedArray
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Changes the scale of the image buffer. The buffer's contents should not be relied on until
    /// it has been redrawn with this new scale.
    pub fn set_scale(&mut self, scale: usize) {
        self.scale = scale;
        self.resize_buffer();
    }

    /// Changes the width of the image buffer. The buffer's contents should not be relied on until
    /// it has been redrawn with this new width.
    pub fn set_width(&mut self, width: usize) {
        self.width = width;
        self.resize_buffer();
    }

    /// Changes the height of the image buffer. The buffer's contents should not be relied on until
    /// it has been redrawn with this new height.
    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        self.resize_buffer();
    }

    /// Sets the raw number of elements in the buffer. After calling this method, the buffer's
    /// contents should not be relied on until it has been redrawn.
    fn resize_buffer(&mut self) {
        let size = RGBA_COMPONENTS * self.width * self.scale * self.height * self.scale;
        self.data.resize_with(size, Default::default);
    }
}

impl Target for ImageBuffer {
    type Item = Rgba<f32>;

    #[inline(always)]
    fn size(&self) -> [usize; 2] {
        [self.width, self.height]
    }

    #[inline(always)]
    unsafe fn set(&mut self, [x, y]: [usize; 2], item: Self::Item) {
        let scale = self.scale;
        for i in 0..scale {
            let col = x * scale + i;
            for j in 0..scale {
                let row = y * scale + j;
                let index = row * RGBA_COMPONENTS * self.width * scale + col * RGBA_COMPONENTS;
                *self.data.get_unchecked_mut(index + 0) = (255.0 * item.r) as u8;
                *self.data.get_unchecked_mut(index + 1) = (255.0 * item.g) as u8;
                *self.data.get_unchecked_mut(index + 2) = (255.0 * item.b) as u8;
                *self.data.get_unchecked_mut(index + 3) = (255.0 * item.a) as u8;
            }
        }
    }

    #[inline(always)]
    unsafe fn get(&self, [x, y]: [usize; 2]) -> Self::Item {
        let scale = self.scale;
        let index = y * scale * RGBA_COMPONENTS * self.width + x * scale * RGBA_COMPONENTS;
        Rgba {
            r: *self.data.get_unchecked(index + 0) as f32 / 255.0,
            g: *self.data.get_unchecked(index + 1) as f32 / 255.0,
            b: *self.data.get_unchecked(index + 2) as f32 / 255.0,
            a: *self.data.get_unchecked(index + 3) as f32 / 255.0,
        }
    }

    fn clear(&mut self, fill: Self::Item) {
        for chunk in self.data.chunks_exact_mut(RGBA_COMPONENTS) {
            chunk[0] = (255.0 * fill.r) as u8;
            chunk[1] = (255.0 * fill.g) as u8;
            chunk[2] = (255.0 * fill.b) as u8;
            chunk[3] = (255.0 * fill.a) as u8;
        }
    }
}
