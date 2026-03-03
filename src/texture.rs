// depth texture format used for depth testing
// 32-bit float gives sufficient precision for 3d rendering
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

// creates a depth texture matching the current window size
// this is required so the gpu can correctly determine which fragments
// are in front of others (hidden surface removal)
pub fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    label: &str,
) -> wgpu::TextureView {
    // match texture size to the swapchain surface dimensions
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1, // single 2d texture, not an array
    };

    // describe how the depth texture will be created
    let desc = wgpu::TextureDescriptor {
        label: Some(label),

        // texture dimensions
        size,

        // no mipmaps needed for depth buffer
        mip_level_count: 1,

        // no multisampling
        sample_count: 1,

        // standard 2d texture
        dimension: wgpu::TextureDimension::D2,

        // depth-only format
        format: DEPTH_FORMAT,

        // used only as a render attachment (not sampled in shader)
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

        // no alternative view formats
        view_formats: &[],
    };

    // allocate texture on gpu
    let texture = device.create_texture(&desc);

    // create a view so render pass can access it
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
