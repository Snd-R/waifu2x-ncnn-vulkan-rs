use std::convert::TryFrom;
use std::ffi::CString;

use image::{DynamicImage, GrayAlphaImage, GrayImage, RgbaImage, RgbImage};
use libc::{c_char, c_int, c_uchar, c_uint, c_void};

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum ModelType {
    Cunet,
    Upconv7AnimeStyleArtRgb,
    Upconv7Photo,
}

#[repr(C)]
#[derive(Debug)]
pub struct Image {
    pub data: *const c_uchar,
    pub w: c_int,
    pub h: c_int,
    pub c: c_int,
}

extern "C" {
    fn init_waifu2x(
        gpuid: c_int,
        tta_mode: bool,
        num_threads: c_int,
        noise: c_int,
        scale: c_int,
        tilesize: c_int,
        prepadding: c_int,
    ) -> *mut c_void;

    fn init_gpu_instance();

    fn get_gpu_count() -> c_int;

    fn destroy_gpu_instance();

    fn load(waifu2x: *mut c_void, param_path: *const c_char, model_path: *const c_char);

    fn process(waifu2x: *mut c_void, in_image: *const Image, out_image: *const Image) -> c_int;

    fn process_cpu(waifu2x: *mut c_void, in_image: &Image, out_image: &Image) -> c_int;

    fn get_heap_budget(gpuid: c_int) -> c_uint;

    fn free_image(image: *const Image);

    fn free_waifu2x(waifu2x: *mut c_void);
}

pub struct Waifu2x {
    waifu2x: *mut c_void,
    scale: u32,
}

unsafe impl Send for Waifu2x {}

impl Waifu2x {
    pub fn new(gpuid: i32,
               noise: i32,
               scale: u32,
               model: ModelType,
               tile_size: u32,
               tta_mode: bool,
               num_threads: i32,
               models_path: String,
    ) -> Self {
        unsafe {
            let prepadding = match model {
                ModelType::Cunet => {
                    if noise == -1 {
                        18
                    } else if scale == 1 {
                        28
                    } else { 18 }
                }
                ModelType::Upconv7AnimeStyleArtRgb => 7,
                ModelType::Upconv7Photo => 7,
            };

            let model_dir = match model {
                ModelType::Cunet => "models-cunet",
                ModelType::Upconv7AnimeStyleArtRgb => "models-upconv_7_anime_style_art_rgb",
                ModelType::Upconv7Photo => "models-upconv_7_photo"
            };


            let (model_path, param_path) = if noise == -1 {
                (format!("{}/{}/scale2.0x_model.bin", models_path, model_dir),
                 format!("{}/{}/scale2.0x_model.param", models_path, model_dir))
            } else if scale == 1 {
                (format!("{}/{}/noise{}_model.bin", models_path, model_dir, noise),
                 format!("{}/{}/noise{}_model.param", models_path, model_dir, noise))
            } else {
                (format!("{}/{}/noise{}_scale2.0x_model.bin", models_path, model_dir, noise),
                 format!("{}/{}/noise{}_scale2.0x_model.param", models_path, model_dir, noise))
            };

            init_gpu_instance();
            let gpu_count = get_gpu_count() as i32;
            if gpuid < -1 || gpuid >= gpu_count {
                destroy_gpu_instance();
                panic!("invalid gpu device")
            }
            let tile_size = if tile_size == 0 {
                if gpuid == -1 { 400 } else {
                    let calculated_tile_size;
                    let heap_budget = get_heap_budget(gpuid);

                    match model {
                        ModelType::Cunet => {
                            if heap_budget > 2600 {
                                calculated_tile_size = 400
                            } else if heap_budget > 740 {
                                calculated_tile_size = 200
                            } else if heap_budget > 250 {
                                calculated_tile_size = 100
                            } else { calculated_tile_size = 32 }
                        }
                        ModelType::Upconv7AnimeStyleArtRgb | ModelType::Upconv7Photo => {
                            if heap_budget > 1900 {
                                calculated_tile_size = 400
                            } else if heap_budget > 550 {
                                calculated_tile_size = 200
                            } else if heap_budget > 190 {
                                calculated_tile_size = 100
                            } else { calculated_tile_size = 32 }
                        }
                    }

                    calculated_tile_size
                }
            } else { tile_size };
            let waifu2x = init_waifu2x(
                gpuid,
                tta_mode,
                num_threads,
                noise,
                scale as i32,
                tile_size as i32,
                prepadding,
            );


            let param_path_cstr = CString::new(param_path).unwrap();
            let model_path_cstr = CString::new(model_path).unwrap();
            load(waifu2x, param_path_cstr.as_ptr(), model_path_cstr.as_ptr());

            Self {
                waifu2x,
                scale,
            }
        }
    }

    pub fn proc_image(&self, image: DynamicImage) -> DynamicImage {
        let bytes_per_pixel = image.color().bytes_per_pixel();

        let (input_image, channels) = if bytes_per_pixel == 1 {
            (DynamicImage::from(image.to_rgb8()), 3)
        } else if bytes_per_pixel == 2 {
            (DynamicImage::from(image.to_rgba8()), 4)
        } else {
            (image, bytes_per_pixel)
        };

        let in_buffer = Image {
            data: input_image.as_bytes().as_ptr() as *const c_uchar,
            w: i32::try_from(input_image.width()).unwrap(),
            h: i32::try_from(input_image.height()).unwrap(),
            c: i32::from(channels),
        };


        unsafe {
            let out_buffer =
                if self.scale == 1 {
                    let output_ptr = std::ptr::null_mut();
                    let out_buffer = Image {
                        data: output_ptr,
                        w: in_buffer.w,
                        h: in_buffer.h,
                        c: in_buffer.c,
                    };
                    process(self.waifu2x, &in_buffer as *const Image, &out_buffer as *const Image);
                    out_buffer
                } else {
                    let scale_run_count = match self.scale {
                        2 => 1,
                        4 => 2,
                        8 => 3,
                        16 => 4,
                        32 => 5,
                        _ => { panic!("unexpected scale number") }
                    };

                    let output_ptr = std::ptr::null_mut();
                    let mut out_buffer = Image {
                        data: output_ptr,
                        w: in_buffer.w * 2,
                        h: in_buffer.h * 2,
                        c: i32::from(channels),
                    };
                    process(self.waifu2x, &in_buffer as *const Image, &out_buffer as *const Image);

                    let mut tmp: Image;
                    for _ in 1..scale_run_count {
                        println!("scaling inside the loop");
                        tmp = out_buffer;

                        let output_ptr = std::ptr::null_mut();
                        out_buffer = Image {
                            data: output_ptr,
                            w: tmp.w * 2,
                            h: tmp.h * 2,
                            c: tmp.c,
                        };
                        process(self.waifu2x, &tmp as *const Image, &out_buffer as *const Image);
                        free_image(&tmp as *const Image)
                    }
                    out_buffer
                };

            let length = usize::try_from(out_buffer.h * out_buffer.w * channels as i32).unwrap();
            let copied_bytes = std::slice::from_raw_parts(out_buffer.data as *const u8, length).to_vec();
            free_image(&out_buffer as *const Image);

            Self::convert_image(out_buffer.w as u32, out_buffer.h as u32, channels, copied_bytes)
        }
    }

    fn convert_image(width: u32, height: u32, channels: u8, bytes: Vec<u8>) -> DynamicImage {
        let image = match channels {
            4 => DynamicImage::from(RgbaImage::from_raw(width, height, bytes).unwrap()),

            3 => DynamicImage::from(RgbImage::from_raw(width, height, bytes).unwrap()),

            2 => DynamicImage::from(GrayAlphaImage::from_raw(width, height, bytes).unwrap()),

            1 => DynamicImage::from(GrayImage::from_raw(width, height, bytes).unwrap()),

            _ => panic!("unexpected channel")
        };
        image
    }
}

impl Drop for Waifu2x {
    fn drop(&mut self) {
        unsafe {
            free_waifu2x(self.waifu2x);
        }
    }
}