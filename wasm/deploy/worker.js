importScripts("wasm.js");

//const { step_to_triangle_buf, init_log } = wasm_bindgen;

const { step_to_gltf, init_log } = wasm_bindgen;
async function run() {
    await wasm_bindgen("worker_bg.wasm");
    init_log();

    onmessage = function(e) {
        //var triangles = step_to_triangle_buf(e.data);
        //postMessage(triangles);

        var glb = step_to_gltf(e.data);
        postMessage(glb);

    }
}
run();
