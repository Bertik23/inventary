use yew::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlVideoElement, MediaStream, MediaStreamTrack, MediaStreamConstraints, HtmlCanvasElement, CanvasRenderingContext2d};
use js_sys::{Reflect, Function};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_scan: Callback<String>,
}

#[function_component(BarcodeScanner)]
pub fn barcode_scanner(props: &Props) -> Html {
    let video_ref = use_node_ref();
    let scanning = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    let canvas_ref = use_node_ref();
    
    {
        let video_ref = video_ref.clone();
        let scanning = scanning.clone();
        let on_scan = props.on_scan.clone();
        let error = error.clone();
        let canvas_ref = canvas_ref.clone();
        
        use_effect_with((), move |_| {
            let video_ref_cleanup = video_ref.clone();
            let video_ref_async = video_ref.clone();
            let scanning = scanning.clone();
            let on_scan = on_scan.clone();
            let error = error.clone();
            let canvas_ref = canvas_ref.clone();
            
            spawn_local(async move {
                if let Some(video) = video_ref_async.cast::<HtmlVideoElement>() {
                    // Get user media (camera) using Navigator API
                    let window = web_sys::window().unwrap();
                    let navigator = window.navigator();
                    let media_devices_result = navigator.media_devices();
                    
                    let media_devices = match media_devices_result {
                        Ok(md) => md,
                        Err(_) => {
                            error.set(Some("MediaDevices not available. Camera access requires HTTPS or localhost.".to_string()));
                            scanning.set(false);
                            return;
                        }
                    };
                    
                    if media_devices.is_undefined() {
                        error.set(Some("MediaDevices not available. Camera access requires HTTPS or localhost.".to_string()));
                        scanning.set(false);
                        return;
                    }
                    
                    // Create video constraints
                    let video_constraints = js_sys::Object::new();
                    Reflect::set(&video_constraints, &JsValue::from_str("facingMode"), &JsValue::from_str("environment")).unwrap();
                    
                    let mut constraints = MediaStreamConstraints::new();
                    constraints.set_video(&JsValue::from(&video_constraints));
                    
                    match media_devices.get_user_media_with_constraints(&constraints) {
                        Ok(stream_promise) => {
                            match wasm_bindgen_futures::JsFuture::from(stream_promise).await {
                                Ok(stream_value) => {
                                    let stream: MediaStream = stream_value.dyn_into().unwrap();
                                    video.set_src_object(Some(&stream));
                                    let _ = video.play(); // Explicitly play the video
                                    
                                    // Wait a bit for video to initialize
                                    gloo_timers::future::TimeoutFuture::new(500).await;
                                    
                                    // Start barcode detection
                                    start_barcode_detection(video, canvas_ref, on_scan, scanning, error).await;
                                }
                                Err(e) => {
                                    let error_msg = format!("Failed to access camera: {:?}", e);
                                    log::error!("{}", error_msg);
                                    error.set(Some(error_msg));
                                    scanning.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to get user media: {:?}", e);
                            log::error!("{}", error_msg);
                            error.set(Some(error_msg));
                            scanning.set(false);
                        }
                    }
                }
            });
            
            move || {
                // Cleanup: stop all tracks
                if let Some(video) = video_ref_cleanup.cast::<HtmlVideoElement>() {
                    if let Some(stream) = video.src_object().and_then(|s| s.dyn_ref::<MediaStream>().cloned()) {
                        for track_js in stream.get_tracks().iter() {
                            if let Some(track) = track_js.dyn_ref::<MediaStreamTrack>() {
                                track.stop();
                            }
                        }
                    }
                    video.set_src_object(None);
                }
            }
        });
    }
    
    html! {
        <div class="relative w-full h-full bg-black flex items-center justify-center overflow-hidden">
            <video
                ref={video_ref}
                autoplay=true
                playsinline=true
                muted=true
                class="absolute inset-0 w-full h-full object-cover"
            />
            <canvas ref={canvas_ref} style="display: none;"></canvas>
            {if let Some(ref err) = *error {
                html! {
                    <div class="absolute inset-0 flex items-center justify-center bg-black/80 text-white p-4 text-center z-10">{err}</div>
                }
            } else {
                html! {
                    <div class="absolute bottom-4 left-0 right-0 text-center text-white/90 text-sm bg-black/40 py-2 backdrop-blur-sm z-10">{"Point camera at barcode"}</div>
                }
            }}
        </div>
    }
}

async fn start_barcode_detection(
    video: HtmlVideoElement,
    canvas_ref: NodeRef,
    on_scan: Callback<String>,
    scanning: UseStateHandle<bool>,
    error: UseStateHandle<Option<String>>,
) {
    let window = web_sys::window().unwrap();
    
    // Check if BarcodeDetector API is available
    let barcode_detector_available = Reflect::has(&window, &JsValue::from_str("BarcodeDetector")).unwrap_or(false);
    
    if barcode_detector_available {
        // Use native BarcodeDetector API
        detect_with_barcode_detector(video, canvas_ref, on_scan, scanning, error).await;
    } else {
        // Fallback: Try to use ZXing via JavaScript
        detect_with_zxing(video, canvas_ref, on_scan, scanning, error).await;
    }
}

async fn detect_with_barcode_detector(
    video: HtmlVideoElement,
    canvas_ref: NodeRef,
    on_scan: Callback<String>,
    scanning: UseStateHandle<bool>,
    error: UseStateHandle<Option<String>>,
) {
    let window = web_sys::window().unwrap();
    
    // Get BarcodeDetector constructor
    let barcode_detector_ctor = Reflect::get(&window, &JsValue::from_str("BarcodeDetector")).ok();
    
    if barcode_detector_ctor.is_none() {
        error.set(Some("BarcodeDetector API not supported".to_string()));
        scanning.set(false);
        return;
    }
    
    // Create BarcodeDetector instance with supported formats
    let formats = js_sys::Array::new();
    formats.push(&JsValue::from_str("ean_13"));
    formats.push(&JsValue::from_str("ean_8"));
    formats.push(&JsValue::from_str("upc_a"));
    formats.push(&JsValue::from_str("upc_e"));
    formats.push(&JsValue::from_str("code_128"));
    formats.push(&JsValue::from_str("code_39"));
    formats.push(&JsValue::from_str("qr_code"));
    
    let detector_ctor_fn: Function = match barcode_detector_ctor.unwrap().dyn_into() {
        Ok(f) => f,
        Err(_) => {
            error.set(Some("BarcodeDetector is not a constructor".to_string()));
            scanning.set(false);
            return;
        }
    };
    
    let detector = Reflect::construct(&detector_ctor_fn, &js_sys::Array::from_iter([JsValue::from(&formats)])).ok();
    
    if detector.is_none() {
        error.set(Some("Failed to create BarcodeDetector".to_string()));
        scanning.set(false);
        return;
    }
    
    let detector = detector.unwrap();
    let canvas = canvas_ref.cast::<HtmlCanvasElement>();
    
    if canvas.is_none() {
        error.set(Some("Canvas not available".to_string()));
        scanning.set(false);
        return;
    }
    
    let canvas = canvas.unwrap();
    let ctx = canvas.get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    
    let mut last_scan_time = 0.0;
    let scan_interval = 300.0; // Scan every 300ms
    
    loop {
        if !*scanning {
            break;
        }
        
        // Wait for video to be ready
        if video.ready_state() != 4 { // HAVE_ENOUGH_DATA
            gloo_timers::future::TimeoutFuture::new(100).await;
            continue;
        }
        
        let now = js_sys::Date::now();
        if now - last_scan_time < scan_interval {
            gloo_timers::future::TimeoutFuture::new(50).await;
            continue;
        }
        
        last_scan_time = now;
        
        // Set canvas size to match video
        let video_width = video.video_width();
        let video_height = video.video_height();
        
        if video_width == 0 || video_height == 0 {
            gloo_timers::future::TimeoutFuture::new(100).await;
            continue;
        }
        
        canvas.set_width(video_width);
        canvas.set_height(video_height);
        
        // Draw video frame to canvas
        if ctx.draw_image_with_html_video_element(&video, 0.0, 0.0).is_err() {
            gloo_timers::future::TimeoutFuture::new(100).await;
            continue;
        }
        
        // Detect barcodes using BarcodeDetector API
        let detect_fn = Reflect::get(&detector, &JsValue::from_str("detect")).ok();
        if let Some(detect_method_js) = detect_fn {
            if let Ok(detect_method) = detect_method_js.dyn_into::<Function>() {
                let detect_result = Reflect::apply(&detect_method, &detector, &js_sys::Array::from_iter([JsValue::from(&canvas)]));
            
                if let Ok(result_promise) = detect_result {
                    let result: js_sys::Promise = result_promise.dyn_into().unwrap();
                    match wasm_bindgen_futures::JsFuture::from(result).await {
                        Ok(barcodes) => {
                            let barcodes_array: js_sys::Array = barcodes.dyn_into().unwrap();
                            if barcodes_array.length() > 0 {
                                let first_barcode = barcodes_array.get(0);
                                if let Ok(raw_value) = Reflect::get(&first_barcode, &JsValue::from_str("rawValue")) {
                                    if let Some(barcode_str) = raw_value.as_string() {
                                        log::info!("Barcode detected: {}", barcode_str);
                                        on_scan.emit(barcode_str);
                                        scanning.set(false);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::debug!("Barcode detection error: {:?}", e);
                        }
                    }
                }
            }
        }
        
        gloo_timers::future::TimeoutFuture::new(50).await;
    }
}

async fn detect_with_zxing(
    video: HtmlVideoElement,
    canvas_ref: NodeRef,
    on_scan: Callback<String>,
    scanning: UseStateHandle<bool>,
    error: UseStateHandle<Option<String>>,
) {
    // Check if ZXing is loaded
    let window = web_sys::window().unwrap();
    let zxing_available = Reflect::has(&window, &JsValue::from_str("ZXing")).unwrap_or(false);
    
    if !zxing_available {
        error.set(Some("BarcodeDetector API not available. Please load ZXing library or use a browser that supports BarcodeDetector API.".to_string()));
        scanning.set(false);
        return;
    }
    
    let canvas = canvas_ref.cast::<HtmlCanvasElement>();
    if canvas.is_none() {
        error.set(Some("Canvas not available".to_string()));
        scanning.set(false);
        return;
    }
    
    let canvas = canvas.unwrap();
    let ctx = canvas.get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    
    // Initialize ZXing reader once, outside the loop
    let zxing = Reflect::get(&window, &JsValue::from_str("ZXing")).unwrap();
    let code_reader_ctor = Reflect::get(&zxing, &JsValue::from_str("BrowserMultiFormatReader")).ok();
    
    let reader = if let Some(ctor) = code_reader_ctor {
        if let Ok(reader_instance) = Reflect::construct(&ctor.dyn_into::<Function>().unwrap(), &js_sys::Array::new()) {
            reader_instance
        } else {
            error.set(Some("Failed to create ZXing reader".to_string()));
            scanning.set(false);
            return;
        }
    } else {
        error.set(Some("ZXing BrowserMultiFormatReader not available".to_string()));
        scanning.set(false);
        return;
    };

    let mut last_scan_time = 0.0;
    let scan_interval = 500.0;
    
    loop {
        if !*scanning {
            break;
        }
        
        if video.ready_state() != 4 {
            gloo_timers::future::TimeoutFuture::new(100).await;
            continue;
        }
        
        let now = js_sys::Date::now();
        if now - last_scan_time < scan_interval {
            gloo_timers::future::TimeoutFuture::new(100).await;
            continue;
        }
        
        last_scan_time = now;
        
        let video_width = video.video_width();
        let video_height = video.video_height();
        
        if video_width == 0 || video_height == 0 {
            gloo_timers::future::TimeoutFuture::new(100).await;
            continue;
        }
        
        canvas.set_width(video_width);
        canvas.set_height(video_height);
        
        if ctx.draw_image_with_html_video_element(&video, 0.0, 0.0).is_err() {
            gloo_timers::future::TimeoutFuture::new(100).await;
            continue;
        }
        
        // Use decodeFromCanvas which is standard in BrowserMultiFormatReader
        let decode_fn_js = Reflect::get(&reader, &JsValue::from_str("decodeFromCanvas")).ok();
        
        if let Some(decode_method_js) = decode_fn_js {
            if let Ok(decode_method) = decode_method_js.dyn_into::<Function>() {
                // decodeFromCanvas returns a Promise
                if let Ok(result) = Reflect::apply(&decode_method, &reader, &js_sys::Array::from_iter([JsValue::from(&canvas)])) {
                    let result_promise: js_sys::Promise = result.dyn_into().unwrap();
                    match wasm_bindgen_futures::JsFuture::from(result_promise).await {
                        Ok(decode_result) => {
                            if let Ok(text) = Reflect::get(&decode_result, &JsValue::from_str("text")) {
                                if let Some(barcode_str) = text.as_string() {
                                    log::info!("Barcode detected with ZXing: {}", barcode_str);
                                    on_scan.emit(barcode_str);
                                    scanning.set(false);
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            // No barcode found or error, continue scanning
                        }
                    }
                }
            }
        }
        
        gloo_timers::future::TimeoutFuture::new(100).await;
    }
}
