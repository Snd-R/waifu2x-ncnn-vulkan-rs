#if _WIN32
#include <locale>
#include <codecvt>
#include <string>
#endif

#include "waifu2x.h"

typedef struct Image {
    unsigned char *data;
    int w;
    int h;
    int c;
} Image;

extern "C" Waifu2x *init_waifu2x(
        int gpuid,
        bool tta_mode,
        int num_threads,
        int noise,
        int scale,
        int tilesize,
        int prepadding
) {
    auto waifu2x = new Waifu2x(gpuid, tta_mode, num_threads);
    waifu2x->noise = noise;
    waifu2x->scale = (scale >= 2) ? 2 : scale;
    waifu2x->tilesize = tilesize;
    waifu2x->prepadding = prepadding;
    return waifu2x;
}

extern "C" void init_gpu_instance() {
    ncnn::create_gpu_instance();
}
extern "C" int get_gpu_count() {
    return ncnn::get_gpu_count();
}

extern "C" void destroy_gpu_instance() {
    ncnn::destroy_gpu_instance();
}

extern "C" int load(Waifu2x *waifu2x, const char *param_path, const char *model_path) {
#if _WIN32
    std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> converter;
    return waifu2x->load(converter.from_bytes(param_path), converter.from_bytes(model_path));
#else
    return waifu2x->load(param_path, model_path);
#endif
}

extern "C" int process(Waifu2x *waifu2x, const Image *in_image, Image *out_image, void **mat_ptr) {
    int c = in_image->c;
    ncnn::Mat in_image_mat =
            ncnn::Mat(in_image->w, in_image->h, (void *) in_image->data, (size_t) c, c);

    auto *out_image_mat =
            new ncnn::Mat(out_image->w, out_image->h, (size_t) c, c);

    int result = waifu2x->process(in_image_mat, *out_image_mat);
    out_image->data = static_cast<unsigned char *>(out_image_mat->data);
    *mat_ptr = out_image_mat;
    return result;
}

extern "C" int process_cpu(Waifu2x *waifu2x, const Image *in_image, Image *out_image, void **mat_ptr) {
    int c = in_image->c;
    ncnn::Mat in_image_mat =
            ncnn::Mat(in_image->w, in_image->h, (void *) in_image->data, (size_t) c, c);
    auto *out_image_mat =
            new ncnn::Mat(out_image->w, out_image->h, (size_t) c, c);

    int result = waifu2x->process_cpu(in_image_mat, *out_image_mat);
    out_image->data = static_cast<unsigned char *>(out_image_mat->data);
    *mat_ptr = out_image_mat;
    return result;
}

extern "C" uint32_t get_heap_budget(int gpuid) {
    return ncnn::get_gpu_device(gpuid)->get_heap_budget();
}

extern "C" void free_image(ncnn::Mat *mat_ptr) {
    delete mat_ptr;
}

extern "C" void free_waifu2x(Waifu2x *waifu2X) {
    delete waifu2X;
    ncnn::destroy_gpu_instance();
}


