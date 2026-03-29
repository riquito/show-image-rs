use crate::ImageInfo;
use crate::ImageView;
use crate::{Alpha, PixelFormat};

/// A GPU image texture ready to be used with the rendering pipeline.
pub struct GpuImage {
	name: String,
	info: ImageInfo,
	bind_group: wgpu::BindGroup,
	_texture: wgpu::Texture,
}

impl GpuImage {
	/// Create a [`GpuImage`] from an image buffer.
	///
	/// Converts the image data to RGBA8 on the CPU and uploads it as a GPU texture.
	pub fn from_data(
		name: String,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		bind_group_layout: &wgpu::BindGroupLayout,
		sampler: &wgpu::Sampler,
		image: &ImageView,
	) -> Self {
		let info = image.info();
		let width = info.size.x;
		let height = info.size.y;
		let mip_level_count = mip_levels(width, height);

		let rgba_data = convert_to_rgba8(image);

		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some(&format!("{}_texture", name)),
			size: wgpu::Extent3d {
				width,
				height,
				depth_or_array_layers: 1,
			},
			mip_level_count,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		});

		// Upload mip level 0
		queue.write_texture(
			wgpu::TexelCopyTextureInfo {
				texture: &texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			&rgba_data,
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(width * 4),
				rows_per_image: Some(height),
			},
			wgpu::Extent3d {
				width,
				height,
				depth_or_array_layers: 1,
			},
		);

		// Generate and upload remaining mip levels on CPU
		let mut prev = rgba_data;
		let mut mip_w = width;
		let mut mip_h = height;
		for level in 1..mip_level_count {
			let next_w = (mip_w / 2).max(1);
			let next_h = (mip_h / 2).max(1);
			let next = downsample_2x(&prev, mip_w, mip_h, next_w, next_h);

			queue.write_texture(
				wgpu::TexelCopyTextureInfo {
					texture: &texture,
					mip_level: level,
					origin: wgpu::Origin3d::ZERO,
					aspect: wgpu::TextureAspect::All,
				},
				&next,
				wgpu::TexelCopyBufferLayout {
					offset: 0,
					bytes_per_row: Some(next_w * 4),
					rows_per_image: Some(next_h),
				},
				wgpu::Extent3d {
					width: next_w,
					height: next_h,
					depth_or_array_layers: 1,
				},
			);

			prev = next;
			mip_w = next_w;
			mip_h = next_h;
		}

		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(&format!("{}_bind_group", name)),
			layout: bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(sampler),
				},
			],
		});

		Self {
			name,
			info,
			bind_group,
			_texture: texture,
		}
	}

	/// Get the name of the image.
	#[allow(unused)]
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the image info.
	pub fn info(&self) -> &ImageInfo {
		&self.info
	}

	/// Get the bind group that should be used to render the image with the rendering pipeline.
	pub fn bind_group(&self) -> &wgpu::BindGroup {
		&self.bind_group
	}
}

/// Convert any supported pixel format to tightly-packed RGBA8 bytes.
fn convert_to_rgba8(image: &ImageView) -> Vec<u8> {
	let info = image.info();
	let data = image.data();
	let width = info.size.x as usize;
	let height = info.size.y as usize;
	let stride_x = info.stride.x as usize;
	let stride_y = info.stride.y as usize;

	let mut rgba = vec![0u8; width * height * 4];

	for y in 0..height {
		for x in 0..width {
			let src = y * stride_y + x * stride_x;
			let dst = (y * width + x) * 4;

			match info.pixel_format {
				PixelFormat::Mono8 => {
					let v = data[src];
					rgba[dst] = v;
					rgba[dst + 1] = v;
					rgba[dst + 2] = v;
					rgba[dst + 3] = 255;
				}
				PixelFormat::MonoAlpha8(Alpha::Unpremultiplied) => {
					let v = data[src];
					let a = data[src + 1];
					rgba[dst] = v;
					rgba[dst + 1] = v;
					rgba[dst + 2] = v;
					rgba[dst + 3] = a;
				}
				PixelFormat::MonoAlpha8(Alpha::Premultiplied) => {
					let a = data[src + 1];
					let v = if a == 0 { 0 } else { ((data[src] as u16 * 255) / a as u16) as u8 };
					rgba[dst] = v;
					rgba[dst + 1] = v;
					rgba[dst + 2] = v;
					rgba[dst + 3] = a;
				}
				PixelFormat::Bgr8 => {
					rgba[dst] = data[src + 2];
					rgba[dst + 1] = data[src + 1];
					rgba[dst + 2] = data[src];
					rgba[dst + 3] = 255;
				}
				PixelFormat::Bgra8(Alpha::Unpremultiplied) => {
					rgba[dst] = data[src + 2];
					rgba[dst + 1] = data[src + 1];
					rgba[dst + 2] = data[src];
					rgba[dst + 3] = data[src + 3];
				}
				PixelFormat::Bgra8(Alpha::Premultiplied) => {
					let a = data[src + 3];
					if a == 0 {
						rgba[dst] = 0;
						rgba[dst + 1] = 0;
						rgba[dst + 2] = 0;
					} else {
						rgba[dst] = ((data[src + 2] as u16 * 255) / a as u16) as u8;
						rgba[dst + 1] = ((data[src + 1] as u16 * 255) / a as u16) as u8;
						rgba[dst + 2] = ((data[src] as u16 * 255) / a as u16) as u8;
					}
					rgba[dst + 3] = a;
				}
				PixelFormat::Rgb8 => {
					rgba[dst] = data[src];
					rgba[dst + 1] = data[src + 1];
					rgba[dst + 2] = data[src + 2];
					rgba[dst + 3] = 255;
				}
				PixelFormat::Rgba8(Alpha::Unpremultiplied) => {
					rgba[dst] = data[src];
					rgba[dst + 1] = data[src + 1];
					rgba[dst + 2] = data[src + 2];
					rgba[dst + 3] = data[src + 3];
				}
				PixelFormat::Rgba8(Alpha::Premultiplied) => {
					let a = data[src + 3];
					if a == 0 {
						rgba[dst] = 0;
						rgba[dst + 1] = 0;
						rgba[dst + 2] = 0;
					} else {
						rgba[dst] = ((data[src] as u16 * 255) / a as u16) as u8;
						rgba[dst + 1] = ((data[src + 1] as u16 * 255) / a as u16) as u8;
						rgba[dst + 2] = ((data[src + 2] as u16 * 255) / a as u16) as u8;
					}
					rgba[dst + 3] = a;
				}
			}
		}
	}

	rgba
}

/// Calculate the number of mip levels for a given image size.
fn mip_levels(width: u32, height: u32) -> u32 {
	(width.max(height) as f32).log2().floor() as u32 + 1
}

/// Downsample RGBA8 data by averaging 2x2 blocks.
fn downsample_2x(src: &[u8], src_w: u32, _src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
	let src_w = src_w as usize;
	let dst_w_usize = dst_w as usize;
	let dst_h_usize = dst_h as usize;
	let mut dst = vec![0u8; dst_w_usize * dst_h_usize * 4];

	for y in 0..dst_h_usize {
		for x in 0..dst_w_usize {
			let sx = x * 2;
			let sy = y * 2;

			// Average the 2x2 block, handling edge cases for odd dimensions
			let s00 = (sy * src_w + sx) * 4;
			let s10 = (sy * src_w + (sx + 1).min(src_w - 1)) * 4;
			let s01 = ((sy + 1).min(src.len() / (src_w * 4) - 1) * src_w + sx) * 4;
			let s11 = ((sy + 1).min(src.len() / (src_w * 4) - 1) * src_w + (sx + 1).min(src_w - 1)) * 4;

			let d = (y * dst_w_usize + x) * 4;
			for c in 0..4 {
				let avg = (src[s00 + c] as u16 + src[s10 + c] as u16 + src[s01 + c] as u16 + src[s11 + c] as u16) / 4;
				dst[d + c] = avg as u8;
			}
		}
	}

	dst
}
