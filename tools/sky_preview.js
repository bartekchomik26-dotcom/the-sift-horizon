// Draws Mei Mei Sky's previews with render.rs's own WGSL on a (software) GPU, Chromium's WebGPU:
// a scene (a sky as vanilla or a shader pack would leave it, and hills with depth) is drawn,
// then the mod's pass runs over it with the blending, depth test and uniform the mod uses.
//   node sky_preview.js <sky.rs> <out dir>
const { chromium } = require('/opt/node22/lib/node_modules/playwright');
const fs = require('fs'); const http = require('http'); const path = require('path');
const src = fs.readFileSync(process.argv[2], 'utf8');
const shader = src.split('r#"')[1].split('"#')[0];
const out = process.argv[3];

(async () => {
  const srv = http.createServer((q, s) => s.end('<html></html>')).listen(8769);
  const b = await chromium.launch({ executablePath: '/opt/pw-browsers/chromium-1243/chrome-linux64/chrome',
    args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan', '--use-webgpu-adapter=swiftshader', '--no-sandbox'] });
  const p = await b.newPage();
  p.on('console', m => console.error('page:', m.text()));
  await p.goto('http://localhost:8769/');
  const result = await p.evaluate(async ({ shader }) => {
    const W = 640, H = 360;
    const dev = await (await navigator.gpu.requestAdapter()).requestDevice();
    const errors = []; dev.onuncapturederror = e => errors.push(e.error.message);
    const mul = (A, B) => { const C = new Float32Array(16); for (let c = 0; c < 4; c++) for (let r = 0; r < 4; r++) { let s = 0; for (let k = 0; k < 4; k++) s += A[k * 4 + r] * B[c * 4 + k]; C[c * 4 + r] = s; } return C; };
    const inv = (m) => { const o = new Float32Array(16); const [a00,a01,a02,a03,a10,a11,a12,a13,a20,a21,a22,a23,a30,a31,a32,a33] = m;
      const b00=a00*a11-a01*a10,b01=a00*a12-a02*a10,b02=a00*a13-a03*a10,b03=a01*a12-a02*a11,b04=a01*a13-a03*a11,b05=a02*a13-a03*a12,b06=a20*a31-a21*a30,b07=a20*a32-a22*a30,b08=a20*a33-a23*a30,b09=a21*a32-a22*a31,b10=a21*a33-a23*a31,b11=a22*a33-a23*a32;
      const det=1/(b00*b11-b01*b10+b02*b09+b03*b08-b04*b07+b05*b06);
      o[0]=(a11*b11-a12*b10+a13*b09)*det;o[1]=(a02*b10-a01*b11-a03*b09)*det;o[2]=(a31*b05-a32*b04+a33*b03)*det;o[3]=(a22*b04-a21*b05-a23*b03)*det;
      o[4]=(a12*b08-a10*b11-a13*b07)*det;o[5]=(a00*b11-a02*b08+a03*b07)*det;o[6]=(a32*b02-a30*b05-a33*b01)*det;o[7]=(a20*b05-a22*b02+a23*b01)*det;
      o[8]=(a10*b10-a11*b08+a13*b06)*det;o[9]=(a01*b08-a00*b10-a03*b06)*det;o[10]=(a30*b04-a31*b02+a33*b00)*det;o[11]=(a21*b02-a20*b04-a23*b00)*det;
      o[12]=(a11*b07-a10*b09-a12*b06)*det;o[13]=(a00*b09-a01*b07+a02*b06)*det;o[14]=(a31*b01-a30*b03-a32*b00)*det;o[15]=(a20*b03-a21*b01+a22*b00)*det; return o; };
    const norm = v => { const l = Math.hypot(...v); return v.map(x => x / l); };
    const cross = (x, y) => [x[1]*y[2]-x[2]*y[1], x[2]*y[0]-x[0]*y[2], x[0]*y[1]-x[1]*y[0]];
    function viewProj(fwd, fov) {
      const f = norm(fwd), r = norm(cross(f, [0, 1, 0])), u = cross(r, f);
      const V = new Float32Array([r[0], u[0], -f[0], 0, r[1], u[1], -f[1], 0, r[2], u[2], -f[2], 0, 0, 0, 0, 1]);
      const n = 0.05, fy = 1 / Math.tan(fov * Math.PI / 360), asp = W / H;
      // Reversed, infinite (as Horizon's).
      const P = new Float32Array([fy / asp, 0, 0, 0, 0, fy, 0, 0, 0, 0, 0, -1, 0, 0, n, 0]);
      return mul(P, V);
    }
    const tex = (w, h, format, usage) => dev.createTexture({ size: [w, h], format, usage });
    // The scene: a sky (top and horizon colours, a sun) and hills with depth.
    const sceneCode = `
struct S { clip_to_world: mat4x4<f32>, top: vec4<f32>, horizon: vec4<f32>, sun: vec4<f32>, ground: vec4<f32> }
@group(0) @binding(0) var<uniform> s: S;
struct V { @builtin(position) position: vec4<f32> }
struct O { @location(0) color: vec4<f32>, @builtin(frag_depth) depth: f32 }
@vertex fn vs(@builtin(vertex_index) i: u32) -> V {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u)); var o: V; o.position = vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0); return o;
}
@fragment fn fs(v: V) -> O {
    let ndc = vec2<f32>(v.position.x / ${W}.0 * 2.0 - 1.0, 1.0 - v.position.y / ${H}.0 * 2.0);
    let f = s.clip_to_world * vec4<f32>(ndc, 0.5, 1.0);
    let d = normalize(f.xyz / f.w);
    let a = atan2(d.x, -d.z);
    let hill = 0.03 + 0.05 * (0.5 + 0.5 * sin(a * 3.0 + 1.0)) + 0.02 * sin(a * 11.0);
    var o: O;
    if (d.y < hill) {
        let shade = 0.75 + 0.25 * sin(a * 23.0);
        o.color = vec4<f32>(s.ground.rgb * shade, 1.0);
        o.depth = 0.001;
        return o;
    }
    var c = mix(s.horizon.rgb, s.top.rgb, pow(clamp(d.y, 0.0, 1.0), 0.5));
    let sd = normalize(s.sun.xyz);
    c += vec3<f32>(1.0, 0.9, 0.7) * pow(max(dot(d, sd), 0.0), 900.0) * s.sun.w;
    o.color = vec4<f32>(min(c, vec3<f32>(1.0)), 1.0);
    o.depth = 0.0;
    return o;
}`;
    const sceneMod = dev.createShaderModule({ code: sceneCode });
    const sceneInfo = await sceneMod.getCompilationInfo();
    for (const x of sceneInfo.messages) if (x.type === 'error') errors.push('scene ' + x.lineNum + ': ' + x.message);
    const scenePipe = dev.createRenderPipeline({ layout: 'auto', vertex: { module: sceneMod, entryPoint: 'vs' },
      fragment: { module: sceneMod, entryPoint: 'fs', targets: [{ format: 'rgba8unorm' }] },
      depthStencil: { format: 'depth32float', depthWriteEnabled: true, depthCompare: 'always' } });
    // The mod's pass, as render.rs builds it.
    const mod = dev.createShaderModule({ code: shader });
    const info = await mod.getCompilationInfo();
    for (const x of info.messages) if (x.type === 'error') errors.push('mod ' + x.lineNum + ': ' + x.message);
    const layout = dev.createBindGroupLayout({ entries: [
      { binding: 0, visibility: GPUShaderStage.FRAGMENT, buffer: {} },
      { binding: 1, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'depth' } },
    ] });
    const modPipe = dev.createRenderPipeline({ layout: dev.createPipelineLayout({ bindGroupLayouts: [layout] }),
      vertex: { module: mod, entryPoint: 'vs_sky' },
      fragment: { module: mod, entryPoint: 'fs_sky', targets: [{ format: 'rgba8unorm', blend: {
        color: { srcFactor: 'src-alpha', dstFactor: 'one-minus-src-alpha', operation: 'add' },
        alpha: { srcFactor: 'zero', dstFactor: 'one', operation: 'add' } } }] } });
    async function read(t) {
      const bpr = Math.ceil(W * 4 / 256) * 256;
      const buf = dev.createBuffer({ size: bpr * H, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
      const e = dev.createCommandEncoder(); e.copyTextureToBuffer({ texture: t }, { buffer: buf, bytesPerRow: bpr }, [W, H]); dev.queue.submit([e.finish()]);
      await buf.mapAsync(GPUMapMode.READ); const s = new Uint8Array(buf.getMappedRange()); const o = new Array(W * H * 4);
      for (let y = 0; y < H; y++) for (let x = 0; x < W * 4; x++) o[y * W * 4 + x] = s[y * bpr + x];
      buf.unmap(); return o;
    }
    async function shot(s) {
      const vp = viewProj(s.fwd, s.fov ?? 70); const c2w = inv(vp);
      const color = tex(W, H, 'rgba8unorm', GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC);
      const depth = tex(W, H, 'depth32float', GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING);
      const su = new Float32Array(32); su.set(c2w, 0); su.set([...s.top, 1], 16); su.set([...s.horizon, 1], 20); su.set([...s.sun, s.sunOn ?? 1], 24); su.set([0.30, 0.42, 0.36, 1], 28);
      const sb = dev.createBuffer({ size: 128, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST }); dev.queue.writeBuffer(sb, 0, su);
      const mu = new Float32Array(36); mu.set(c2w, 0);
      const k = s.mm;
      const body = a => { const t = a * Math.PI / 180; return [-Math.sin(t), Math.cos(t), 0]; };
      mu.set([...k.sky, 1, ...k.fog, 1, ...body(k.sun), k.day, ...body(k.moon), k.rain, W, H, W, H], 16);
      const mb = dev.createBuffer({ size: 144, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST }); dev.queue.writeBuffer(mb, 0, mu);
      const e = dev.createCommandEncoder();
      let pass = e.beginRenderPass({ colorAttachments: [{ view: color.createView(), loadOp: 'clear', storeOp: 'store', clearValue: [0, 0, 0, 1] }],
        depthStencilAttachment: { view: depth.createView(), depthLoadOp: 'clear', depthStoreOp: 'store', depthClearValue: 0 } });
      pass.setPipeline(scenePipe); pass.setBindGroup(0, dev.createBindGroup({ layout: scenePipe.getBindGroupLayout(0), entries: [{ binding: 0, resource: { buffer: sb } }] })); pass.draw(3); pass.end();
      if (k.on) {
        pass = e.beginRenderPass({ colorAttachments: [{ view: color.createView(), loadOp: 'load', storeOp: 'store' }] });
        pass.setPipeline(modPipe);
        pass.setBindGroup(0, dev.createBindGroup({ layout, entries: [{ binding: 0, resource: { buffer: mb } }, { binding: 1, resource: depth.createView() }] }));
        pass.draw(3); pass.end();
      }
      dev.queue.submit([e.finish()]);
      return await read(color);
    }
    const hex = h => [((h >> 16) & 255) / 255, ((h >> 8) & 255) / 255, (h & 255) / 255];
    const end = { top: [0.02, 0.0, 0.03], horizon: [0.04, 0.02, 0.05], sun: [0, -1, 0], sunOn: 0 };
    const dayMM = { on: true, sky: hex(0xf7cfe0), fog: hex(0xc9eff1), sun: 45, moon: 225, day: 1, rain: 0 };
    const nightMM = { on: true, sky: hex(0x176db5), fog: hex(0x0d3d78), sun: 225, moon: 45, day: 0, rain: 0 };
    const shots = {
      end_vanilla: { ...end, fwd: [-1, 0.35, 0], fov: 90, mm: { ...dayMM, on: false } },
      day_west: { ...end, fwd: [-1, 0.45, 0.2], fov: 90, mm: dayMM },
      day_east: { ...end, fwd: [1, 0.2, 0], fov: 90, mm: dayMM },
      night_west: { ...end, fwd: [-1, 0.45, 0.2], fov: 90, mm: nightMM },
      rain_west: { ...end, fwd: [-1, 0.45, 0.2], fov: 90, mm: { ...dayMM, rain: 0.8, sky: hex(0xb8a8b0), fog: hex(0x9fb5b7) } },
      up: { ...end, fwd: [-0.3, 3, 0], fov: 100, mm: dayMM },
    };
    const o = {};
    for (const [k, s] of Object.entries(shots)) o[k] = await shot(s);
    return { o, errors, W, H };
  }, { shader });
  for (const [k, v] of Object.entries(result.o)) fs.writeFileSync(path.join(out, k + '.rgba'), Buffer.from(v));
  console.log(JSON.stringify({ errors: result.errors, images: Object.keys(result.o) }));
  await b.close(); srv.close();
})();
