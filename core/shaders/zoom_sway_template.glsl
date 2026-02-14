//!HOOK MAIN
//!BIND HOOKED
//!DESC SoloveykoAI Zoom/Sway

// Try multiple frame variables just in case
// Usually for libplacebo filter in ffmpeg, N or frame might be exposed if extra_opts configured
// But since the user says their script works with 'frame', let's use that.

// If 'frame' is not defined, try to define it via N
#ifndef frame
#define frame N
#endif

vec4 hook() {
    float FPS = {{FPS}};
    float DURATION = {{DURATION}};
    
    // Zoom/Sway Settings
    float zoom_base = {{ZOOM_BASE}};
    float zoom_amp = {{ZOOM_AMP}};
    float zoom_speed = {{ZOOM_SPEED}};
    
    float sway_speed = {{SWAY_SPEED}};
    float ups_factor = {{UPS_FACTOR}};
    
    // Time calculation
    // Casting frame to float for division
    // If 'frame' or 'N' is undefined, compilation will fail.
    // If user's script works, 'frame' MUST be available.
    
    // Use presentation timestamp (PTS) from libplacebo hook.
    // This is the most reliable way to get time in seconds if exposed.
    // Fallback: If HOOKED_pts is not defined by the implementation, try N/FPS.
    
    float time = HOOKED_pts;
    if (time < 0.001) {
         // Fallback if PTS is 0 (first frame or missing) -> Try frame index if enabled
         time = float(frame) / FPS;
    } 
    
    // Zoom Logic
    float zoom_val = zoom_base;
    if (zoom_amp > 0.001) {
        float cycle = (time / DURATION) * zoom_speed;
        // Cosine based smooth zoom
        zoom_val = zoom_base + zoom_amp * (1.0 - cos(6.28318 * cycle)) / 2.0;
    }
    
    // Sway Logic
    vec2 offset = vec2(0.0);
    if (sway_speed > 0.001) {
        // Amplitude logic from python script
        float amp_x = 0.026 * (ups_factor / 2.0);
        if (amp_x < 0.02) amp_x = 0.02;
        float amp_y = amp_x * 0.5;
        
        float t = time;
        float sa = sway_speed;
        
        offset.x = sin(t * 2.0 * sa) * amp_x + cos(t * 1.3 * sa) * (amp_x * 0.5);
        offset.y = cos(t * 1.7 * sa) * amp_y + sin(t * 0.9 * sa) * (amp_y * 0.5);
        
        // Convert pixel offset to UV standard (0..1)
        // HOOKED_size contains width/height of texture
        offset = offset / HOOKED_size;
    }
    
    // Transform (UV Space)
    vec2 pos = HOOKED_pos;
    vec2 center = vec2(0.5);
    
    // 1. Center transform origin
    // 2. Scale (Zoom) - inverse scaling of UV space means zoom IN
    pos = (pos - center) / zoom_val + center;
    
    // 3. Translate (Sway, Pan) - subtract offset to simulate camera move
    pos = pos - offset;
    
    return HOOKED_tex(pos);
}
