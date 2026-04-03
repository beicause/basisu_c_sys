use async_lock::OnceCell;

static BASISU_INITIALIZED: OnceCell<()> = OnceCell::new();

pub async fn basisu_transcoder_init() {
    BASISU_INITIALIZED
        .get_or_init(async || {
            basisu_c_sys::instantiate_builtin_wasm().await;
            unsafe { basisu_c_sys::transcoder::bt_init() };
        })
        .await;
}
