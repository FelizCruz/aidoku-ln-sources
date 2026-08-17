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
    sources = ["sources/wetriedtls", "sources/dreamytranslations", "sources/novelarrow"]
    for s in sources:
        if os.path.exists(s):
            s_name = os.path.basename(s)
            build_aix(s, f"{s}/{s_name}.aix")

if __name__ == "__main__":
    package_all()
