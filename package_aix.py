import os
import zipfile

def build_aix(source_dir, output_aix):
    source_name = os.path.basename(source_dir.rstrip("/\\"))
    res_dir = os.path.join(source_dir, "res")
    wasm_file = os.path.join("target", "wasm32-unknown-unknown", "release", f"{source_name}.wasm")
    
    if not os.path.exists(wasm_file):
        raise FileNotFoundError(f"WASM file not found at {wasm_file}. Please run cargo build first.")
    
    if not os.path.exists(res_dir):
        raise FileNotFoundError(f"res directory not found at {res_dir}")

    os.makedirs(os.path.dirname(output_aix) if os.path.dirname(output_aix) else ".", exist_ok=True)
    
    with zipfile.ZipFile(output_aix, "w", zipfile.ZIP_DEFLATED) as zf:
        # Add main.wasm
        zf.write(wasm_file, arcname="Payload/main.wasm")
        print(f"[{source_name}] Added Payload/main.wasm ({os.path.getsize(wasm_file)} bytes)")
        
        # Add res directory files
        for fn in sorted(os.listdir(res_dir)):
            full_p = os.path.join(res_dir, fn)
            if os.path.isfile(full_p):
                zf.write(full_p, arcname=f"Payload/{fn}")
                print(f"[{source_name}] Added Payload/{fn} ({os.path.getsize(full_p)} bytes)")

    print(f"[{source_name}] Successfully packaged -> {output_aix} ({os.path.getsize(output_aix)} bytes)")

def package_all():
    sources_dir = "sources"
    for d in sorted(os.listdir(sources_dir)):
        full_d = os.path.join(sources_dir, d)
        if os.path.isdir(full_d) and os.path.exists(os.path.join(full_d, "res", "source.json")):
            build_aix(full_d, os.path.join(full_d, f"{d}.aix"))

if __name__ == "__main__":
    package_all()
